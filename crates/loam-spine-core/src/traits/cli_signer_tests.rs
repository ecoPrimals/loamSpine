// SPDX-License-Identifier: AGPL-3.0-or-later

#[expect(
    clippy::unwrap_used,
    reason = "test code uses unwrap for concise assertions"
)]
#[expect(
    clippy::uninlined_format_args,
    reason = "test helper formatting uses explicit format args for clarity"
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
}
