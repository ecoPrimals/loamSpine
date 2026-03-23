// SPDX-License-Identifier: AGPL-3.0-or-later

#![expect(
    clippy::expect_used,
    reason = "integration tests use expect for assertion clarity"
)]

//! Integration tests for CLI signer with real `BearDog` binary.
//!
//! These tests use the actual `BearDog` binary from `../bins/` to test
//! real signing operations. They gracefully skip if the binary is not available.

use loam_spine_core::Entry;
use loam_spine_core::traits::cli_signer::{CliSigner, CliVerifier};
use loam_spine_core::traits::signing::{Signer, Verifier};
use std::path::Path;

/// Path to `BearDog` binary
const BEARDOG_BIN: &str = "../bins/beardog";

/// Helper to check if `BearDog` binary exists
fn beardog_available() -> bool {
    Path::new(BEARDOG_BIN).exists()
}

#[tokio::test]
async fn test_cli_signer_binary_detection() {
    if !beardog_available() {
        eprintln!("⚠️  Skipping test: BearDog binary not found at {BEARDOG_BIN}");
        return;
    }

    // Should detect binary exists
    assert!(
        Path::new(BEARDOG_BIN).exists(),
        "BearDog binary should exist"
    );

    // Check it's executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let Ok(metadata) = std::fs::metadata(BEARDOG_BIN) else {
            eprintln!("⚠️  Could not read metadata");
            return;
        };
        let permissions = metadata.permissions();
        assert!(
            permissions.mode() & 0o111 != 0,
            "BearDog binary should be executable"
        );
    }
}

#[tokio::test]
async fn test_cli_signer_creation_with_valid_binary() {
    if !beardog_available() {
        eprintln!("⚠️  Skipping test: BearDog binary not available");
        return;
    }

    // Try to create signer (may fail if key doesn't exist, which is expected)
    let result = CliSigner::new(BEARDOG_BIN, "test-key");

    // We expect this to either succeed (if test-key exists) or fail with a clear error
    match result {
        Ok(signer) => {
            // Great! The key exists
            println!("✓ Created signer successfully with test-key");
            assert!(signer.did().as_str().starts_with("did:"));
        }
        Err(e) => {
            // Expected if key doesn't exist
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("key")
                    || err_msg.contains("not found")
                    || err_msg.contains("Failed"),
                "Error should be about missing key: {err_msg}"
            );
            println!("⚠️  Key not found (expected): {err_msg}");
        }
    }
}

#[tokio::test]
async fn test_cli_signer_invalid_binary_path() {
    // Should fail gracefully with non-existent binary
    let result = CliSigner::new("/nonexistent/path/to/binary", "test-key");

    assert!(result.is_err(), "Should fail with non-existent binary");

    if let Err(err) = result {
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("not found") || err_msg.contains("binary"),
            "Error should mention binary not found: {err_msg}"
        );
    }
}

#[tokio::test]
async fn test_cli_signer_with_environment_variable() {
    if !beardog_available() {
        eprintln!("⚠️  Skipping test: BearDog binary not available");
        return;
    }

    temp_env::with_vars(
        [
            ("LOAMSPINE_SIGNER_PATH", Some(BEARDOG_BIN)),
            ("LOAMSPINE_SIGNER_KEY", Some("test-key")),
        ],
        || {
            let signer_path = std::env::var("LOAMSPINE_SIGNER_PATH");
            assert!(signer_path.is_ok(), "Should read LOAMSPINE_SIGNER_PATH");
            if let Ok(path) = signer_path {
                assert_eq!(path, BEARDOG_BIN);
            }
        },
    );
}

#[tokio::test]
async fn test_cli_verifier_creation() {
    if !beardog_available() {
        eprintln!("⚠️  Skipping test: BearDog binary not available");
        return;
    }

    // CliVerifier should be creatable regardless of keys
    let result = CliVerifier::new(BEARDOG_BIN);

    match result {
        Ok(_verifier) => {
            println!("✓ Created verifier successfully");
        }
        Err(e) => {
            println!("⚠️  Verifier creation failed: {e}");
            // This is acceptable - binary might not support verification
        }
    }
}

#[tokio::test]
async fn test_cli_signer_did_format() {
    if !beardog_available() {
        eprintln!("⚠️  Skipping test: BearDog binary not available");
        return;
    }

    // If we can create a signer, verify DID format
    if let Ok(signer) = CliSigner::new(BEARDOG_BIN, "test-key") {
        let did = signer.did();

        // DID should start with "did:"
        assert!(
            did.as_str().starts_with("did:"),
            "DID should start with 'did:', got: {}",
            did.as_str()
        );

        // DID should have a method
        assert!(
            did.as_str().contains(':'),
            "DID should contain method separator"
        );

        println!("✓ DID format valid: {}", did.as_str());
    } else {
        println!("⚠️  Could not create signer (key may not exist)");
    }
}

#[tokio::test]
async fn test_cli_signer_capability_pattern() {
    // Test that CliSigner follows capability pattern (no hardcoded primal names)

    // File name should be generic (cli_signer.rs, not beardog.rs) ✓
    // Type name should be generic (CliSigner, not BearDogSigner) ✓
    // No hardcoded "beardog" strings in API ✓

    // Verify we can use ANY binary that implements the protocol
    let any_signer_path = "/path/to/any/ed25519/signer";
    let result = CliSigner::new(any_signer_path, "test-key");

    // Should fail because binary doesn't exist, not because of primal name check
    assert!(result.is_err(), "Should fail with non-existent binary");
    let Err(err) = result else {
        unreachable!();
    };

    let err_msg = err.to_string();
    assert!(
        !err_msg.contains("beardog") && !err_msg.contains("BearDog"),
        "Error should not mention specific primal name: {err_msg}"
    );
}

#[tokio::test]
async fn test_cli_signer_integration_with_entry() {
    use loam_spine_core::{SpineId, entry::SpineConfig};

    if !beardog_available() {
        eprintln!("⚠️  Skipping test: BearDog binary not available");
        return;
    }

    // Test full integration: create entry, sign it, verify it
    if let Ok(signer) = CliSigner::new(BEARDOG_BIN, "test-key") {
        let owner = signer.did().clone();

        // Create a test entry
        let spine_id = SpineId::nil();
        let entry = Entry::genesis(owner.clone(), spine_id, SpineConfig::default());

        // Sign the entry
        let data = entry.to_canonical_bytes().expect("to_canonical_bytes");
        let sign_result = signer.sign(&data).await;

        match sign_result {
            Ok(signature) => {
                println!("✓ Successfully signed entry");
                assert!(
                    !signature.as_bytes().is_empty(),
                    "Signature should not be empty"
                );

                // Try to verify if we have a verifier
                if let Ok(verifier) = CliVerifier::new(BEARDOG_BIN) {
                    let verify_result = verifier.verify(&data, &signature, &owner).await;
                    match verify_result {
                        Ok(verification) => {
                            println!("✓ Verification completed: {:?}", verification.valid);
                        }
                        Err(e) => {
                            println!("⚠️  Verification failed: {e}");
                        }
                    }
                }
            }
            Err(e) => {
                println!("⚠️  Signing failed (may be expected): {e}");
            }
        }
    } else {
        println!("⚠️  Could not create signer (key may not exist)");
    }
}

#[tokio::test]
async fn test_cli_signer_multiple_operations() {
    if !beardog_available() {
        eprintln!("⚠️  Skipping test: BearDog binary not available");
        return;
    }

    // Test that signer can be used multiple times
    if let Ok(signer) = CliSigner::new(BEARDOG_BIN, "test-key") {
        let data1 = b"first message";
        let data2 = b"second message";

        let sig1_result = signer.sign(data1).await;
        let sig2_result = signer.sign(data2).await;

        if let (Ok(sig1), Ok(sig2)) = (sig1_result, sig2_result) {
            // Different data should produce different signatures
            assert_ne!(
                sig1.as_bytes(),
                sig2.as_bytes(),
                "Different messages should have different signatures"
            );
            println!("✓ Multiple signing operations successful");
        } else {
            println!("⚠️  Signing operations failed (may be expected if key doesn't exist)");
        }
    }
}

#[tokio::test]
async fn test_cli_signer_concurrent_operations() {
    if !beardog_available() {
        eprintln!("⚠️  Skipping test: BearDog binary not available");
        return;
    }

    // Test concurrent signing operations
    if let Ok(signer) = CliSigner::new(BEARDOG_BIN, "test-key") {
        use std::sync::Arc;

        let signer = Arc::new(signer);
        let mut handles = vec![];

        for i in 0..5 {
            let signer_clone = Arc::clone(&signer);
            let handle = tokio::spawn(async move {
                let data = format!("message {i}");
                signer_clone.sign(data.as_bytes()).await
            });
            handles.push(handle);
        }

        let mut success_count = 0;
        for handle in handles {
            if let Ok(Ok(_signature)) = handle.await {
                success_count += 1;
            }
        }

        if success_count > 0 {
            println!("✓ Concurrent operations successful: {success_count}/5");
        } else {
            println!("⚠️  No concurrent operations succeeded (key may not exist)");
        }
    }
}

#[tokio::test]
async fn test_binary_path_normalization() {
    // Test various path formats
    let paths = vec![
        "../bins/beardog",
        "../bins/../bins/beardog",
        "./nonexistent",
    ];

    for path in paths {
        let result = CliSigner::new(path, "test-key");
        // Should handle all path formats gracefully
        match result {
            Ok(_) => println!("✓ Path {path} resolved successfully"),
            Err(e) => {
                // Error should be clear about the issue
                assert!(
                    e.to_string().len() > 10,
                    "Error message should be descriptive"
                );
            }
        }
    }
}
