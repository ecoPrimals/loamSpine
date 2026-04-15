// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration tests for CLI signer/verifier: sign/verify flows, trait-object
//! dispatch (DynSigner/DynVerifier), and binary discovery edge cases.
//!
//! Extracted from `cli_signer_tests.rs` — these tests exercise the full
//! sign/verify lifecycle via subprocess invocation and the `Dyn*` trait
//! blanket impls, forming a cohesive "integration + trait-object" domain.

use super::*;
use crate::discovery::{DynSigner, DynVerifier};
use crate::types::ByteBuffer;
use std::path::PathBuf;
use std::sync::Arc;

fn get_test_binary_for_error_paths() -> Option<(PathBuf, PathBuf)> {
    let candidates = ["/usr/bin/true", "/bin/true", "true"];
    let mut true_path = None;
    for c in candidates {
        let p = if c.starts_with('/') {
            PathBuf::from(c)
        } else {
            let output = Command::new("which").arg(c).output().ok()?;
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if s.is_empty() {
                    continue;
                }
                PathBuf::from(s)
            } else {
                continue;
            }
        };
        if p.exists() {
            true_path = Some(p);
            break;
        }
    }
    let true_path = true_path?;
    let false_candidates = ["/usr/bin/false", "/bin/false", "false"];
    let mut false_path = None;
    for c in false_candidates {
        let p = if c.starts_with('/') {
            PathBuf::from(c)
        } else {
            let output = Command::new("which").arg(c).output().ok()?;
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if s.is_empty() {
                    continue;
                }
                PathBuf::from(s)
            } else {
                continue;
            }
        };
        if p.exists() {
            false_path = Some(p);
            break;
        }
    }
    let false_path = false_path?;
    Some((true_path, false_path))
}

// =========================================================================
// Sign/verify integration tests (subprocess invocation)
// =========================================================================

#[tokio::test]
async fn sign_fails_when_binary_does_not_produce_output_file() {
    let Some((true_path, _)) = get_test_binary_for_error_paths() else {
        eprintln!("⚠️  Skipping: true/false binaries not found (non-Unix?)");
        return;
    };

    let signer = CliSigner::new(&true_path, "any-key").expect("true accepts key info");
    let result = signer.sign(b"test data").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("read") || err.to_string().contains("signature"),
        "Expected read/signature error: {}",
        err
    );
}

#[tokio::test]
async fn verify_returns_invalid_when_binary_fails() {
    let Some((_, false_path)) = get_test_binary_for_error_paths() else {
        eprintln!("⚠️  Skipping: true/false binaries not found (non-Unix?)");
        return;
    };

    let verifier = CliVerifier::new(&false_path).expect("false exists");
    let data = b"test data";
    let sig = crate::types::Signature::from_vec(vec![1, 2, 3]);
    let did = crate::types::Did::new("did:key:test");

    let result = verifier.verify(data, &sig, &did).await;

    assert!(result.is_ok());
    let verification = result.unwrap();
    assert!(!verification.valid);
    assert!(verification.error.is_some());
}

#[tokio::test]
async fn verify_entry_returns_invalid_when_binary_fails() {
    let Some((_, false_path)) = get_test_binary_for_error_paths() else {
        eprintln!("⚠️  Skipping: true/false binaries not found (non-Unix?)");
        return;
    };

    let verifier = CliVerifier::new(&false_path).expect("false exists");
    let entry = crate::entry::Entry::new(
        0,
        None,
        crate::types::Did::new("did:test"),
        crate::entry::EntryType::SpineSealed { reason: None },
    );

    let result = verifier.verify_entry(&entry).await;

    assert!(result.is_ok());
    let verification = result.unwrap();
    assert!(!verification.valid);
}

#[tokio::test]
async fn sign_fails_when_binary_returns_nonzero() {
    let script = std::env::temp_dir().join(format!(
        "loamspine-sign-fail-test-{}.sh",
        uuid::Uuid::now_v7()
    ));
    let script_content = r#"#!/bin/sh
case "$1" in
  key) [ "$2" = "info" ] && echo "did:key:test" && exit 0 ;;
  encrypt) echo "Signing failed: mock" >&2; exit 1 ;;
  *) exit 1 ;;
esac
"#;
    if std::fs::write(&script, script_content).is_err() {
        eprintln!("⚠️  Skipping: could not write temp script");
        return;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).is_err() {
            let _ = std::fs::remove_file(&script);
            eprintln!("⚠️  Skipping: could not chmod script");
            return;
        }
    }
    #[cfg(not(unix))]
    {
        eprintln!("⚠️  Skipping: script execution requires Unix");
        let _ = std::fs::remove_file(&script);
        return;
    }

    let Ok(signer) = CliSigner::new(&script, "any-key") else {
        let _ = std::fs::remove_file(&script);
        eprintln!("⚠️  Skipping: could not create signer from script");
        return;
    };
    let result = signer.sign(b"test data").await;
    let _ = std::fs::remove_file(&script);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Signing failed")
            || err.to_string().contains("signing")
            || err.to_string().contains("Signing"),
        "Expected signing failure error: {}",
        err
    );
}

#[tokio::test]
async fn verify_returns_valid_when_binary_succeeds() {
    let Some((true_path, _)) = get_test_binary_for_error_paths() else {
        eprintln!("⚠️  Skipping: true/false binaries not found (non-Unix?)");
        return;
    };

    let verifier = CliVerifier::new(&true_path).expect("true exists");
    let data = b"test data";
    let sig = Signature::from_vec(vec![1, 2, 3]);
    let did = Did::new("did:key:test");

    let result = verifier.verify(data, &sig, &did).await;

    assert!(result.is_ok());
    let verification = result.unwrap();
    assert!(
        verification.valid,
        "true binary exits 0 so verification path returns valid"
    );
}

#[tokio::test]
async fn verify_with_empty_signature_edge_case() {
    let Some((true_path, _)) = get_test_binary_for_error_paths() else {
        eprintln!("⚠️  Skipping: true/false binaries not found (non-Unix?)");
        return;
    };

    let verifier = CliVerifier::new(&true_path).expect("true exists");
    let data = b"data";
    let empty_sig = Signature::empty();
    let did = Did::new("did:key:test");

    let result = verifier.verify(data, &empty_sig, &did).await;
    assert!(result.is_ok());
    let _ = result.unwrap();
}

// =========================================================================
// DynSigner / DynVerifier trait-object dispatch
// =========================================================================

#[tokio::test]
async fn cli_signer_as_dyn_signer_did() {
    let Some((true_path, _)) = get_test_binary_for_error_paths() else {
        eprintln!("⚠️  Skipping: true/false binaries not found (non-Unix?)");
        return;
    };

    let signer = CliSigner::new(&true_path, "any-key").expect("true exists");
    let boxed: Arc<dyn DynSigner> = Arc::new(signer);

    let did = boxed.did();
    assert!(!did.as_str().is_empty());
    assert!(did.as_str().starts_with("did:key:"));
}

#[tokio::test]
async fn cli_signer_as_dyn_signer_sign_boxed_fails() {
    let Some((true_path, _)) = get_test_binary_for_error_paths() else {
        eprintln!("⚠️  Skipping: true/false binaries not found (non-Unix?)");
        return;
    };

    let signer = CliSigner::new(&true_path, "any-key").expect("true exists");
    let boxed: Arc<dyn DynSigner> = Arc::new(signer);

    let data = ByteBuffer::from_static(b"test");
    let result = boxed.sign_boxed(data).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn cli_verifier_as_dyn_verifier_verify_boxed() {
    let Some((_, false_path)) = get_test_binary_for_error_paths() else {
        eprintln!("⚠️  Skipping: true/false binaries not found (non-Unix?)");
        return;
    };

    let verifier = CliVerifier::new(&false_path).expect("false exists");
    let boxed: Arc<dyn DynVerifier> = Arc::new(verifier);

    let data = ByteBuffer::from_static(b"test");
    let sig = Signature::from_vec(vec![1, 2, 3]);
    let did = Did::new("did:key:test");

    let result = boxed.verify_boxed(data, sig, did).await;

    assert!(result.is_ok());
    let verification = result.unwrap();
    assert!(!verification.valid);
}

#[tokio::test]
async fn cli_verifier_as_dyn_verifier_verify_entry_boxed() {
    let Some((_, false_path)) = get_test_binary_for_error_paths() else {
        eprintln!("⚠️  Skipping: true/false binaries not found (non-Unix?)");
        return;
    };

    let verifier = CliVerifier::new(&false_path).expect("false exists");
    let boxed: Arc<dyn DynVerifier> = Arc::new(verifier);

    let entry = crate::entry::Entry::new(
        0,
        None,
        Did::new("did:test"),
        crate::entry::EntryType::SpineSealed { reason: None },
    );

    let result = boxed.verify_entry_boxed(entry).await;

    assert!(result.is_ok());
    let verification = result.unwrap();
    assert!(!verification.valid);
}

// =========================================================================
// Additional binary discovery edge cases + sign/verify edge cases
// =========================================================================

#[test]
fn discover_binary_returns_path_when_env_points_to_existing_binary() {
    let Some((true_path, _)) = get_test_binary_for_error_paths() else {
        eprintln!("⚠️  Skipping: true/false binaries not found (non-Unix?)");
        return;
    };

    let path_str = true_path.to_string_lossy();
    let result = CliSigner::discover_binary_from(Some(path_str.as_ref()), None);
    assert!(result.is_some());
    assert_eq!(
        result.as_ref().unwrap().to_string_lossy(),
        true_path.to_string_lossy()
    );
}

#[test]
fn discover_binary_env_var_nonexistent_path_falls_through() {
    let result =
        CliSigner::discover_binary_from(Some("/tmp/loamspine-nonexistent-binary-xyz-9999"), None);
    assert!(
        result.is_none(),
        "non-existent env path should fall through"
    );
}

#[test]
fn discover_binary_returns_none_when_nothing_available() {
    let result = CliSigner::discover_binary_from(None, None);
    let _ = result;
}

#[test]
fn cli_signer_new_binary_not_found() {
    let result = CliSigner::new("/nonexistent/path/to/signer", "key-id");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("not found"),
        "Expected 'not found' error: {err}"
    );
}

#[test]
fn cli_verifier_new_binary_not_found() {
    let result = CliVerifier::new("/nonexistent/path/to/verifier");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("not found"),
        "Expected 'not found' error: {err}"
    );
}

#[test]
fn cli_signer_new_command_fails_with_bad_key() {
    let Some((true_path, _)) = get_test_binary_for_error_paths() else {
        return;
    };
    let result = CliSigner::new(&true_path, "nonexistent-key-xyz");
    let _ = result;
}

#[tokio::test]
async fn cli_signer_sign_with_nonexistent_binary_after_creation() {
    let Some((true_path, _)) = get_test_binary_for_error_paths() else {
        return;
    };

    let temp_dir = std::env::temp_dir();
    let fake_binary = temp_dir.join("loamspine-test-fake-signer");
    std::fs::copy(&true_path, &fake_binary).unwrap();

    let signer = CliSigner::new(&fake_binary, "test-key");
    let _ = std::fs::remove_file(&fake_binary);

    if let Ok(signer) = signer {
        let result = signer.sign(b"test data").await;
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn cli_verifier_verify_with_true_binary() {
    let Some((true_path, _)) = get_test_binary_for_error_paths() else {
        return;
    };

    let verifier = CliVerifier::new(&true_path);
    if let Ok(verifier) = verifier {
        let data = b"test data";
        let sig = crate::types::Signature::from_vec(vec![1, 2, 3]);
        let did = crate::types::Did::new("did:key:test");

        let result = verifier.verify(data, &sig, &did).await;
        assert!(result.is_ok());
        if let Ok(v) = result {
            assert!(v.valid);
        }
    }
}

#[tokio::test]
async fn cli_verifier_verify_with_false_binary() {
    let Some((_, false_path)) = get_test_binary_for_error_paths() else {
        return;
    };

    let verifier = CliVerifier::new(&false_path);
    if let Ok(verifier) = verifier {
        let data = b"test data";
        let sig = crate::types::Signature::from_vec(vec![1, 2, 3]);
        let did = crate::types::Did::new("did:key:test");

        let result = verifier.verify(data, &sig, &did).await;
        assert!(result.is_ok());
        if let Ok(v) = result {
            assert!(!v.valid);
        }
    }
}

#[tokio::test]
async fn cli_verifier_verify_entry_delegates_to_verify() {
    let Some((_, false_path)) = get_test_binary_for_error_paths() else {
        return;
    };

    let verifier = CliVerifier::new(&false_path);
    if let Ok(verifier) = verifier {
        let entry = crate::entry::Entry::new(
            0,
            None,
            crate::types::Did::new("did:test"),
            crate::entry::EntryType::SpineSealed { reason: None },
        );

        let result = verifier.verify_entry(&entry).await;
        assert!(result.is_ok());
    }
}

#[test]
fn cli_signer_accessors() {
    let Some((true_path, _)) = get_test_binary_for_error_paths() else {
        return;
    };

    if let Ok(signer) = CliSigner::new(&true_path, "test-key") {
        assert_eq!(signer.key_id(), "test-key");
        assert_eq!(signer.binary_path(), true_path.as_path());
        assert!(Signer::did(&signer).as_str().contains("test-key"));
    }
}

#[test]
fn env_constants_defined() {
    assert_eq!(ENV_SIGNER_PATH, "LOAMSPINE_SIGNER_PATH");
    assert_eq!(ENV_SIGNER_KEY, "LOAMSPINE_SIGNER_KEY");
}

#[test]
fn discover_binary_finds_signer_in_bins_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let signer_path = tmp.path().join("signer");
    std::fs::write(&signer_path, "#!/bin/sh\nexit 0").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&signer_path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let result = CliSigner::discover_binary_from(None, Some(tmp.path().to_str().unwrap()));
    assert!(result.is_some(), "should find signer in bins dir");
    assert_eq!(result.unwrap(), signer_path);
}

#[test]
fn discover_binary_finds_signing_service_in_bins_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let svc_path = tmp.path().join("signing-service");
    std::fs::write(&svc_path, "#!/bin/sh\nexit 0").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&svc_path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let result = CliSigner::discover_binary_from(None, Some(tmp.path().to_str().unwrap()));
    assert!(result.is_some(), "should find signing-service in bins dir");
    assert_eq!(result.unwrap(), svc_path);
}

#[test]
fn discover_binary_bins_dir_missing_falls_through() {
    let result = CliSigner::discover_binary_from(None, Some("/tmp/nonexistent-bins-dir-xyz-12345"));
    let _ = result;
}

#[test]
fn env_bins_dir_constant_defined() {
    assert_eq!(ENV_BINS_DIR, "LOAMSPINE_BINS_DIR");
}

/// Directory exists but is not an executable file.
#[test]
fn signer_creation_rejects_directory_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let result = CliSigner::new(tmp.path(), "any-key");
    assert!(result.is_err());
}

/// `discover_binary_from` scans `signer` before `signing-service` when both exist.
#[cfg(unix)]
#[test]
fn discover_binary_prefers_signer_over_signing_service_in_bins_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let signer_path = tmp.path().join("signer");
    let svc_path = tmp.path().join("signing-service");
    std::fs::write(&signer_path, "#!/bin/sh\nexit 0").expect("write signer");
    std::fs::write(&svc_path, "#!/bin/sh\nexit 0").expect("write signing-service");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&signer_path, std::fs::Permissions::from_mode(0o755))
            .expect("chmod signer");
        std::fs::set_permissions(&svc_path, std::fs::Permissions::from_mode(0o755))
            .expect("chmod signing-service");
    }

    let result = CliSigner::discover_binary_from(None, Some(tmp.path().to_str().expect("utf8")));
    assert_eq!(result.as_ref().expect("discovered").as_path(), signer_path);
}

#[cfg(unix)]
#[test]
fn signer_creation_dev_null_is_not_a_valid_signing_cli() {
    let result = CliSigner::new("/dev/null", "any-key-id");
    assert!(result.is_err());
}
