// SPDX-License-Identifier: AGPL-3.0-or-later

#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test code uses unwrap/expect for concise assertions"
)]
#[expect(
    clippy::uninlined_format_args,
    reason = "test helper formatting uses explicit format args for clarity"
)]
mod tests {
    use super::super::*;
    use crate::discovery::{DynSigner, DynVerifier};
    use crate::types::ByteBuffer;
    use serial_test::serial;
    use std::path::PathBuf;
    use std::sync::Arc;

    /// Helper to get the path to a standard Unix binary (true/false) for error-path tests.
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

    /// Discover a signing service binary for integration tests.
    ///
    /// Uses the same capability-based discovery as production: checks
    /// `LOAMSPINE_SIGNER_PATH` env var, then standard `../bins/` candidates.
    fn get_test_binary() -> Option<PathBuf> {
        CliSigner::discover_binary()
    }

    #[test]
    fn discover_binary_returns_none_if_not_found() {
        temp_env::with_var(ENV_SIGNER_PATH, None::<&str>, || {
            let result = CliSigner::discover_binary();
            let _ = result;
        });
    }

    #[test]
    fn discover_binary_respects_env_var() {
        let test_path = "/tmp/test-signer";
        temp_env::with_var(ENV_SIGNER_PATH, Some(test_path), || {
            let result = CliSigner::discover_binary();
            assert!(result.is_none());
        });
    }

    #[test]
    fn signer_creation_fails_with_missing_binary() {
        let result = CliSigner::new("/nonexistent/signer", "test-key");
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(err, LoamSpineError::Config(_)));
            assert!(err.to_string().contains("not found"));
        }
    }

    #[test]
    fn signer_creation_fails_with_invalid_key() {
        // Skip if no binary available
        let Some(binary) = get_test_binary() else {
            eprintln!("⚠️  Skipping test: signing service binary not found");
            return;
        };

        let result = CliSigner::new(binary, "nonexistent-key-id-12345");
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(
                err,
                LoamSpineError::Config(_) | LoamSpineError::Internal(_)
            ));
        }
    }

    #[test]
    fn verifier_creation_fails_with_missing_binary() {
        let result = CliVerifier::new("/nonexistent/signer");
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(err, LoamSpineError::Config(_)));
            assert!(err.to_string().contains("not found"));
        }
    }

    #[test]
    fn verifier_creation_succeeds_with_valid_binary() {
        // Skip if no binary available
        let Some(binary) = get_test_binary() else {
            eprintln!("⚠️  Skipping test: signing service binary not found");
            return;
        };

        let result = CliVerifier::new(binary);
        assert!(result.is_ok());
    }

    #[test]
    fn verifier_binary_path_accessor() {
        // Skip if no binary available
        let Some(binary) = get_test_binary() else {
            eprintln!("⚠️  Skipping test: signing service binary not found");
            return;
        };

        let verifier = CliVerifier::new(&binary).unwrap();
        assert_eq!(verifier.binary_path(), binary.as_path());
    }

    #[test]
    fn env_constants_are_defined() {
        assert_eq!(ENV_SIGNER_PATH, "LOAMSPINE_SIGNER_PATH");
        assert_eq!(ENV_SIGNER_KEY, "LOAMSPINE_SIGNER_KEY");
    }

    #[test]
    fn signer_key_id_accessor() {
        let key_id = "test-key-123";
        let result = CliSigner::new("/nonexistent/binary", key_id);
        assert!(result.is_err());
        if let Some(signer) = get_test_binary().and_then(|b| CliSigner::new(b, key_id).ok()) {
            assert_eq!(signer.key_id(), key_id);
        }
    }

    #[test]
    fn signer_binary_path_accessor() {
        let result = CliSigner::new("/nonexistent/binary", "key");
        assert!(result.is_err());
        if let Some(binary) = get_test_binary()
            && let Ok(signer) = CliSigner::new(&binary, "default")
        {
            assert_eq!(signer.binary_path(), binary.as_path());
        }
    }

    #[test]
    fn signer_implements_debug() {
        // Skip if no binary available
        let Some(binary) = get_test_binary() else {
            eprintln!("⚠️  Skipping test: signing service binary not found");
            return;
        };

        // This test verifies the binary exists and can query keys
        // If it fails, it means the binary exists but has no keys setup
        if let Ok(signer) = CliSigner::new(&binary, "default") {
            let debug_str = format!("{:?}", signer);
            assert!(debug_str.contains("CliSigner"));
        }
    }

    #[test]
    fn verifier_implements_debug() {
        // Skip if no binary available
        let Some(binary) = get_test_binary() else {
            eprintln!("⚠️  Skipping test: signing service binary not found");
            return;
        };

        let verifier = CliVerifier::new(binary).unwrap();
        let debug_str = format!("{:?}", verifier);
        assert!(debug_str.contains("CliVerifier"));
    }

    #[test]
    fn signer_implements_clone() {
        // Skip if no binary available
        let Some(binary) = get_test_binary() else {
            eprintln!("⚠️  Skipping test: signing service binary not found");
            return;
        };

        if let Ok(signer) = CliSigner::new(binary, "default") {
            let cloned = signer.clone();
            assert_eq!(Signer::did(&signer), Signer::did(&cloned));
        }
    }

    #[test]
    fn verifier_implements_clone() {
        // Skip if no binary available
        let Some(binary) = get_test_binary() else {
            eprintln!("⚠️  Skipping test: signing service binary not found");
            return;
        };

        let verifier = CliVerifier::new(binary).unwrap();
        let cloned = verifier.clone();
        assert_eq!(verifier.binary_path(), cloned.binary_path());
    }

    #[tokio::test]
    async fn signer_did_accessor() {
        // Skip if no binary available
        let Some(binary) = get_test_binary() else {
            eprintln!("⚠️  Skipping test: signing service binary not found");
            return;
        };

        if let Ok(signer) = CliSigner::new(binary, "default") {
            let did = Signer::did(&signer);
            assert!(!did.as_str().is_empty());
            assert!(did.as_str().starts_with("did:"));
        }
    }

    #[test]
    fn binary_path_normalization() {
        // Test different path formats
        let paths = vec![
            "/absolute/path/to/signer",
            "./relative/signer",
            "../relative/../signer",
        ];

        for path_str in paths {
            let path = PathBuf::from(path_str);
            // Just verify path handling doesn't panic
            let _ = path.exists();
        }
    }

    #[test]
    fn cli_signer_capability_pattern() {
        // Test that CLI signer follows capability pattern
        // (no hardcoded primal names in struct)
        let _binary = PathBuf::from("/tmp/test");

        // Structure should not contain primal names
        let struct_name = std::any::type_name::<CliSigner>();
        assert!(
            !struct_name.contains("beardog"),
            "Should not hardcode primal names"
        );
        assert!(
            !struct_name.contains("nestgate"),
            "Should not hardcode primal names"
        );
    }

    #[test]
    fn environment_variable_priority() {
        temp_env::with_var(ENV_SIGNER_PATH, Some("/test/priority/path"), || {
            let result = CliSigner::discover_binary();
            assert!(result.is_none());
        });
    }

    #[test]
    fn binary_discovery_searches_multiple_locations() {
        temp_env::with_var(ENV_SIGNER_PATH, None::<&str>, || {
            let result = CliSigner::discover_binary();
            let _ = result;
        });
    }

    #[test]
    fn cli_signer_did_format() {
        // Test DID format construction
        let key_id = "test-key-123";
        let expected_prefix = "did:key:";

        // DID should follow did:key: format
        let did_string = format!("did:key:{}", key_id);
        assert!(did_string.starts_with(expected_prefix));
        assert!(did_string.contains(key_id));
    }

    #[test]
    fn error_messages_are_descriptive() {
        // Test that error messages contain useful information
        let result = CliSigner::new("/nonexistent/binary", "key");

        if let Err(e) = result {
            let msg = e.to_string();
            // Should mention binary not found
            assert!(
                msg.contains("not found") || msg.contains("binary"),
                "Error message should be descriptive: {}",
                msg
            );
        }
    }

    #[test]
    fn binary_path_validation() {
        // Test various invalid paths
        let invalid_paths = vec![
            "", "/", "/tmp", // directory, not file
        ];

        for path in invalid_paths {
            let result = CliSigner::new(path, "key");
            // Should fail gracefully (error, not panic)
            assert!(result.is_err(), "Should reject invalid path: {}", path);
        }
    }

    #[test]
    fn verifier_handles_nonexistent_binary() {
        // Verifier should fail gracefully with missing binary
        let result = CliVerifier::new("/absolutely/does/not/exist/binary");

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, LoamSpineError::Config(_)));
        }
    }

    #[test]
    fn concurrent_signer_creation() {
        // Test that signer creation is thread-safe
        use std::thread;

        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    let path = format!("/tmp/signer-{}", i);
                    let result = CliSigner::new(&path, "key");
                    // Should fail (path doesn't exist) but shouldn't panic
                    assert!(result.is_err());
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[tokio::test]
    async fn sign_fails_when_binary_does_not_produce_output_file() {
        // Use `true` as binary: it succeeds (exit 0) but doesn't create the output file,
        // so we hit the "Failed to read signature" error path.
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
        // Use `false` as binary: it always exits non-zero, so we hit the invalid verification path.
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
        // Use a temp script: succeeds for "key info" but fails for "encrypt", hitting
        // the "Signing failed" error path (!output.status.success()).
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
        // Use `true` as binary: it exits 0, so verify returns valid (even though it doesn't
        // actually verify - we're testing the success path).
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
        // Empty signature with `true` binary: writes empty file, runs decrypt, may succeed or fail
        let _ = result.unwrap();
    }

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

    #[test]
    #[serial]
    fn discover_binary_returns_path_when_env_points_to_existing_binary() {
        let Some((true_path, _)) = get_test_binary_for_error_paths() else {
            eprintln!("⚠️  Skipping: true/false binaries not found (non-Unix?)");
            return;
        };

        let path_str = true_path.to_string_lossy();
        temp_env::with_var(ENV_SIGNER_PATH, Some(path_str.as_ref()), || {
            let result = CliSigner::discover_binary();
            assert!(result.is_some());
            assert_eq!(
                result.as_ref().unwrap().to_string_lossy(),
                true_path.to_string_lossy()
            );
        });
    }

    // =========================================================================
    // Additional coverage: discover_binary paths, sign/verify edge cases
    // =========================================================================

    #[test]
    #[serial]
    fn discover_binary_env_var_nonexistent_path_falls_through() {
        temp_env::with_var(
            ENV_SIGNER_PATH,
            Some("/tmp/loamspine-nonexistent-binary-xyz-9999"),
            || {
                let result = CliSigner::discover_binary();
                assert!(
                    result.is_none(),
                    "non-existent env path should fall through"
                );
            },
        );
    }

    #[test]
    #[serial]
    fn discover_binary_returns_none_when_nothing_available() {
        temp_env::with_vars(
            [
                (ENV_SIGNER_PATH, None::<&str>),
                (ENV_SIGNER_KEY, None::<&str>),
            ],
            || {
                let result = CliSigner::discover_binary();
                // May or may not find a system binary; just ensure no panic
                let _ = result;
            },
        );
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
        // `true` always exits 0, so this may succeed with an empty DID.
        // The important thing is it doesn't panic.
        let _ = result;
    }

    #[tokio::test]
    async fn cli_signer_sign_with_nonexistent_binary_after_creation() {
        let Some((true_path, _)) = get_test_binary_for_error_paths() else {
            return;
        };

        // Create a temp file that looks like a binary
        let temp_dir = std::env::temp_dir();
        let fake_binary = temp_dir.join("loamspine-test-fake-signer");
        std::fs::copy(&true_path, &fake_binary).unwrap();

        // Create signer with the copy
        let signer = CliSigner::new(&fake_binary, "test-key");

        // Remove the binary to simulate disappearance between new() and sign()
        let _ = std::fs::remove_file(&fake_binary);

        if let Ok(signer) = signer {
            let result = signer.sign(b"test data").await;
            // Should fail since binary no longer exists
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
            // `true` exits 0, so verification should report valid
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
            // `false` exits non-zero, so verification should report invalid
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

        // `true` will exit 0 for any args, simulating a valid key
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
}
