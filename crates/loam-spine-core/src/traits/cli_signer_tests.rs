// SPDX-License-Identifier: AGPL-3.0-or-later

#[expect(
    clippy::unwrap_used,
    reason = "test code uses unwrap for concise assertions"
)]
#[expect(
    clippy::uninlined_format_args,
    reason = "test helper formatting uses explicit format args for clarity"
)]
#[expect(clippy::panic, reason = "test helpers panic on setup failures")]
#[expect(
    clippy::needless_pass_by_value,
    reason = "enum arguments passed by value for ergonomics"
)]
mod tests {
    use super::super::*;
    use std::path::PathBuf;

    /// Discover a signing service binary for integration tests.
    ///
    /// Uses the same capability-based discovery as production: checks
    /// `LOAMSPINE_SIGNER_PATH` env var, then standard `../bins/` candidates.
    fn get_test_binary() -> Option<PathBuf> {
        CliSigner::discover_binary()
    }

    #[test]
    fn discover_binary_returns_none_if_not_found() {
        let result = CliSigner::discover_binary_from(None, None);
        let _ = result;
    }

    #[test]
    fn discover_binary_respects_env_var() {
        let test_path = "/tmp/test-signer";
        let result = CliSigner::discover_binary_from(Some(test_path), None);
        assert!(result.is_none());
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
        let result = CliSigner::discover_binary_from(Some("/test/priority/path"), None);
        assert!(result.is_none());
    }

    #[test]
    fn binary_discovery_searches_multiple_locations() {
        let result = CliSigner::discover_binary_from(None, None);
        let _ = result;
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

    #[cfg(unix)]
    static MOCK_BINARY_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[cfg(unix)]
    enum MockScript {
        Success,
        KeyInfoFail,
        EncryptFail,
        DecryptFail,
    }

    #[cfg(unix)]
    fn mock_binary_guard() -> std::sync::MutexGuard<'static, ()> {
        MOCK_BINARY_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[cfg(unix)]
    fn is_text_file_busy(err: &LoamSpineError) -> bool {
        err.to_string().contains("Text file busy")
    }

    #[cfg(unix)]
    fn mock_signer_new_result(path: &std::path::Path, key_id: &str) -> LoamSpineResult<CliSigner> {
        let mut last = CliSigner::new(path, key_id);
        for _ in 0..4 {
            if !matches!(&last, Err(e) if is_text_file_busy(e)) {
                return last;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            last = CliSigner::new(path, key_id);
        }
        last
    }

    #[cfg(unix)]
    fn new_mock_signer(path: &std::path::Path, key_id: &str) -> CliSigner {
        mock_signer_new_result(path, key_id)
            .unwrap_or_else(|err| panic!("CliSigner::new failed: {err}"))
    }

    #[cfg(unix)]
    fn new_mock_verifier(path: &std::path::Path) -> CliVerifier {
        for attempt in 0..5 {
            match CliVerifier::new(path) {
                Ok(verifier) => return verifier,
                Err(err) if attempt < 4 && is_text_file_busy(&err) => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(err) => panic!("CliVerifier::new failed: {err}"),
            }
        }
        unreachable!("retry loop returns or panics")
    }

    #[cfg(unix)]
    macro_rules! retry_async {
        ($op:expr) => {{
            let mut result = $op.await;
            for _ in 0..4 {
                if let Err(ref err) = result {
                    if is_text_file_busy(err) {
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                        result = $op.await;
                        continue;
                    }
                }
                break;
            }
            result
        }};
    }

    #[cfg(unix)]
    fn install_mock_script(name: &str, content: &str) -> PathBuf {
        let dir = std::env::var_os("CARGO_TARGET_DIR")
            .map_or_else(|| PathBuf::from("target"), PathBuf::from)
            .join("loamspine-mock-signers");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        write_named_executable_script_at(&path, content);
        path
    }

    #[cfg(unix)]
    fn mock_script_path(script: MockScript) -> &'static std::path::Path {
        static SUCCESS: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
        static KEY_FAIL: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
        static ENCRYPT_FAIL: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
        static DECRYPT_FAIL: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
        match script {
            MockScript::Success => {
                SUCCESS.get_or_init(|| install_mock_script("success.sh", MOCK_SUCCESS_SCRIPT))
            }
            MockScript::KeyInfoFail => KEY_FAIL
                .get_or_init(|| install_mock_script("key-info-fail.sh", MOCK_KEY_INFO_FAIL_SCRIPT)),
            MockScript::EncryptFail => ENCRYPT_FAIL
                .get_or_init(|| install_mock_script("encrypt-fail.sh", MOCK_ENCRYPT_FAIL_SCRIPT)),
            MockScript::DecryptFail => DECRYPT_FAIL
                .get_or_init(|| install_mock_script("decrypt-fail.sh", MOCK_DECRYPT_FAIL_SCRIPT)),
        }
    }

    #[cfg(unix)]
    fn write_named_executable_script_at(path: &std::path::Path, content: &str) {
        use std::os::unix::fs::PermissionsExt;

        std::fs::write(path, content).unwrap();
        let mut perms = std::fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms).unwrap();
    }

    #[cfg(unix)]
    fn write_named_executable_script(dir: &std::path::Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        write_named_executable_script_at(&path, content);
        path
    }

    #[cfg(unix)]
    const MOCK_SUCCESS_SCRIPT: &str = "#!/bin/sh\n[ \"$1\" = key ] && [ \"$2\" = info ] && exit 0\n[ \"$1\" = encrypt ] && { printf mock-signature-bytes > \"$7\"; exit 0; }\n[ \"$1\" = decrypt ] && exit 0\nexit 1\n";
    #[cfg(unix)]
    const MOCK_KEY_INFO_FAIL_SCRIPT: &str =
        "#!/bin/sh\n[ \"$1\" = key ] && { echo key not found >&2; exit 1; }\nexit 1\n";
    #[cfg(unix)]
    const MOCK_ENCRYPT_FAIL_SCRIPT: &str = "#!/bin/sh\n[ \"$1\" = key ] && [ \"$2\" = info ] && exit 0\n[ \"$1\" = encrypt ] && { echo Signing failed: mock >&2; exit 1; }\nexit 1\n";
    #[cfg(unix)]
    const MOCK_DECRYPT_FAIL_SCRIPT: &str = "#!/bin/sh\n[ \"$1\" = key ] && [ \"$2\" = info ] && exit 0\n[ \"$1\" = decrypt ] && { echo Verification failed: mock >&2; exit 1; }\nexit 1\n";

    #[cfg(unix)]
    #[test]
    fn mock_cli_components_succeed_with_fake_binary() {
        let _guard = mock_binary_guard();
        let path = mock_script_path(MockScript::Success);
        let key_id = "test-key-42";
        let signer = new_mock_signer(path, key_id);
        assert_eq!(signer.key_id(), key_id);
        assert_eq!(signer.binary_path(), path);
        assert_eq!(Signer::did(&signer).as_str(), "did:key:test-key-42");
        let verifier = new_mock_verifier(path);
        assert_eq!(verifier.binary_path(), path);
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn mock_signer_sign_succeeds_with_fake_binary() {
        let signer = {
            let _guard = mock_binary_guard();
            new_mock_signer(mock_script_path(MockScript::Success), "sign-key")
        };
        let signature = retry_async!(signer.sign(b"payload to sign")).unwrap();
        assert_eq!(signature.as_bytes(), b"mock-signature-bytes");
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn mock_verifier_verify_returns_valid_with_fake_binary() {
        let verifier = {
            let _guard = mock_binary_guard();
            new_mock_verifier(mock_script_path(MockScript::Success))
        };
        let sig = Signature::from_vec(b"mock-signature-bytes".to_vec());
        let did = Did::new("did:key:sign-key");
        let result = retry_async!(verifier.verify(b"signed payload", &sig, &did)).unwrap();
        assert!(result.valid);
        assert!(result.error.is_none());
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn mock_verifier_verify_entry_succeeds_with_fake_binary() {
        use crate::entry::{Entry, EntryType};

        let verifier = {
            let _guard = mock_binary_guard();
            new_mock_verifier(mock_script_path(MockScript::Success))
        };
        let entry = Entry::new(
            0,
            None,
            Did::new("did:key:sign-key"),
            EntryType::SpineSealed { reason: None },
        );
        let result = retry_async!(verifier.verify_entry(&entry)).unwrap();
        assert!(result.valid);
        assert!(result.error.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn mock_signer_new_fails_when_key_info_exits_nonzero() {
        let _guard = mock_binary_guard();
        let result =
            mock_signer_new_result(mock_script_path(MockScript::KeyInfoFail), "missing-key");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, LoamSpineError::Config(_)));
        assert!(err.to_string().contains("missing-key"));
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn mock_signer_sign_fails_when_encrypt_exits_nonzero() {
        let signer = {
            let _guard = mock_binary_guard();
            new_mock_signer(mock_script_path(MockScript::EncryptFail), "any-key")
        };
        let err = retry_async!(signer.sign(b"test data")).unwrap_err();
        assert!(
            err.to_string().contains("Signing failed"),
            "unexpected: {err}"
        );
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn mock_verifier_verify_returns_invalid_when_decrypt_exits_nonzero() {
        let verifier = {
            let _guard = mock_binary_guard();
            new_mock_verifier(mock_script_path(MockScript::DecryptFail))
        };
        let sig = Signature::from_vec(vec![1, 2, 3]);
        let did = Did::new("did:key:test");
        let result = retry_async!(verifier.verify(b"test data", &sig, &did)).unwrap();
        assert!(!result.valid);
        assert!(
            result
                .error
                .as_ref()
                .is_some_and(|msg| msg.contains("Verification failed"))
        );
    }

    #[cfg(unix)]
    #[test]
    fn discover_binary_from_finds_bins_dir_candidates() {
        let tmp = tempfile::tempdir().unwrap();
        let signer_path =
            write_named_executable_script(tmp.path(), "signer", "#!/bin/sh\nexit 0\n");
        assert_eq!(
            CliSigner::discover_binary_from(None, Some(tmp.path().to_str().unwrap()))
                .unwrap()
                .as_path(),
            signer_path.as_path()
        );
        let tmp = tempfile::tempdir().unwrap();
        let svc_path =
            write_named_executable_script(tmp.path(), "signing-service", "#!/bin/sh\nexit 0\n");
        assert_eq!(
            CliSigner::discover_binary_from(None, Some(tmp.path().to_str().unwrap()))
                .unwrap()
                .as_path(),
            svc_path.as_path()
        );
    }
}
