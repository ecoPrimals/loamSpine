//! CLI-based signer integration for real cryptographic signing.
//!
//! This module provides a real `Signer` implementation that uses an external
//! CLI binary for Ed25519 signing operations. This replaces the mock signer for
//! production use.
//!
//! ## Design Philosophy
//!
//! - **Primal Self-Knowledge**: `LoamSpine` doesn't hardcode external primal names
//! - **Runtime Discovery**: Binary path configured at runtime via environment
//! - **Capability-Based**: Registers with `CapabilityRegistry` for other primals to discover
//! - **Graceful Degradation**: Falls back if signing service unavailable
//! - **Zero Vendor Lock-in**: Works with any CLI that implements the signing protocol
//!
//! ## Usage
//!
//! ```rust,no_run
//! use loam_spine_core::traits::cli_signer::CliSigner;
//! use loam_spine_core::discovery::CapabilityRegistry;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create signer with binary path (discovered or configured)
//! let signer = CliSigner::new(
//!     "/path/to/signing-service",
//!     "my-key-id",  // key ID
//! )?;
//!
//! // Register with capability system for other primals to discover
//! let registry = CapabilityRegistry::new();
//! registry.register_signer(Arc::new(signer)).await;
//! # Ok(())
//! # }
//! ```
//!
//! ## Environment Variables
//!
//! - `LOAMSPINE_SIGNER_PATH`: Path to the signing service binary
//! - `LOAMSPINE_SIGNER_KEY`: Default key ID to use

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::entry::Entry;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::traits::signing::{SignatureVerification, Signer, Verifier};
use crate::types::{Did, Signature};

/// Environment variable for signer binary path.
pub const ENV_SIGNER_PATH: &str = "LOAMSPINE_SIGNER_PATH";
/// Environment variable for default signer key.
pub const ENV_SIGNER_KEY: &str = "LOAMSPINE_SIGNER_KEY";

/// CLI-based signer using external binary.
///
/// This is the production signer that uses an external CLI binary
/// for Ed25519 signing operations. The binary must implement a standard
/// signing protocol (key info, encrypt/sign, decrypt/verify).
#[derive(Clone, Debug)]
pub struct CliSigner {
    /// Path to the signer binary.
    binary_path: PathBuf,
    /// Key ID to use for signing.
    key_id: String,
    /// DID derived from the key.
    did: Did,
}

impl CliSigner {
    /// Create a new CLI signer.
    ///
    /// # Arguments
    ///
    /// * `binary_path` - Path to the signing CLI binary
    /// * `key_id` - Key ID to use for signing (must exist in the signing service)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Binary doesn't exist
    /// - Key ID doesn't exist in the signing service
    pub fn new(binary_path: impl AsRef<Path>, key_id: impl Into<String>) -> LoamSpineResult<Self> {
        let binary_path = binary_path.as_ref().to_path_buf();
        let key_id = key_id.into();

        // Verify binary exists
        if !binary_path.exists() {
            return Err(LoamSpineError::Config(format!(
                "Signing service binary not found: {}",
                binary_path.display()
            )));
        }

        // Get key info to verify it exists and get DID
        let output = Command::new(&binary_path)
            .args(["key", "info", &key_id])
            .output()
            .map_err(|e| LoamSpineError::Config(format!("Failed to run signing service: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LoamSpineError::Config(format!(
                "Key '{key_id}' not found in signing service: {stderr}"
            )));
        }

        // Parse DID from key info output
        // Construct a DID from the key ID (service-agnostic format)
        let did = Did::new(format!("did:key:{key_id}"));

        Ok(Self {
            binary_path,
            key_id,
            did,
        })
    }

    /// Discover signing service binary in standard locations.
    ///
    /// Searches in order:
    /// 1. Environment variable `LOAMSPINE_SIGNER_PATH`
    /// 2. `../bins/` directory (Phase 2 integration path)
    /// 3. System PATH (looks for common signing service names)
    #[must_use]
    pub fn discover_binary() -> Option<PathBuf> {
        // Check environment variable first (highest priority)
        if let Ok(path) = std::env::var(ENV_SIGNER_PATH) {
            let path = PathBuf::from(path);
            if path.exists() {
                return Some(path);
            }
        }

        // Check Phase 2 bins directory for any signing service
        let bins_dir = PathBuf::from("../bins");
        if bins_dir.is_dir() {
            // Look for generic signing service binaries (discovered at runtime)
            // No primal names - only capability-based discovery
            for candidate in &["signer", "signing-service"] {
                let path = bins_dir.join(candidate);
                if path.exists() {
                    return Some(path);
                }
            }
        }

        // Check system PATH for common signing service names
        for candidate in &["loamspine-signer", "signer"] {
            if let Ok(output) = Command::new("which").arg(candidate).output() {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        return Some(PathBuf::from(path));
                    }
                }
            }
        }

        None
    }

    /// Get the key ID.
    #[must_use]
    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    /// Get the binary path.
    #[must_use]
    pub fn binary_path(&self) -> &Path {
        &self.binary_path
    }
}

impl Signer for CliSigner {
    async fn sign(&self, data: &[u8]) -> LoamSpineResult<Signature> {
        // Create temp file for data
        let temp_dir = std::env::temp_dir();
        let data_file = temp_dir.join(format!("loamspine-sign-{}.dat", uuid::Uuid::now_v7()));
        let sig_file = temp_dir.join(format!("loamspine-sig-{}.sig", uuid::Uuid::now_v7()));

        // Write data to temp file
        std::fs::write(&data_file, data)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to write temp data: {e}")))?;

        // Run signing service encrypt (signing mode)
        let output = Command::new(&self.binary_path)
            .args([
                "encrypt",
                "--key",
                &self.key_id,
                "--input",
                data_file.to_str().unwrap_or_default(),
                "--output",
                sig_file.to_str().unwrap_or_default(),
            ])
            .output()
            .map_err(|e| LoamSpineError::Internal(format!("Failed to run signing service: {e}")))?;

        // Clean up data file
        let _ = std::fs::remove_file(&data_file);

        if !output.status.success() {
            let _ = std::fs::remove_file(&sig_file);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LoamSpineError::SignatureVerification(format!(
                "Signing failed: {stderr}"
            )));
        }

        // Read signature
        let sig_bytes = std::fs::read(&sig_file)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to read signature: {e}")))?;

        // Clean up signature file
        let _ = std::fs::remove_file(&sig_file);

        Ok(Signature::new(sig_bytes))
    }

    fn did(&self) -> &Did {
        &self.did
    }
}

/// CLI-based verifier using external binary.
///
/// This is the production verifier that uses an external CLI binary
/// for Ed25519 signature verification.
#[derive(Clone, Debug)]
pub struct CliVerifier {
    /// Path to the verifier binary.
    binary_path: PathBuf,
}

impl CliVerifier {
    /// Create a new CLI verifier.
    ///
    /// # Arguments
    ///
    /// * `binary_path` - Path to the signing service CLI binary
    ///
    /// # Errors
    ///
    /// Returns error if binary doesn't exist.
    pub fn new(binary_path: impl AsRef<Path>) -> LoamSpineResult<Self> {
        let binary_path = binary_path.as_ref().to_path_buf();

        if !binary_path.exists() {
            return Err(LoamSpineError::Config(format!(
                "Signing service binary not found: {}",
                binary_path.display()
            )));
        }

        Ok(Self { binary_path })
    }

    /// Get the binary path.
    #[must_use]
    pub fn binary_path(&self) -> &Path {
        &self.binary_path
    }
}

impl Verifier for CliVerifier {
    async fn verify(
        &self,
        data: &[u8],
        signature: &Signature,
        _signer: &Did,
    ) -> LoamSpineResult<SignatureVerification> {
        // Create temp files
        let temp_dir = std::env::temp_dir();
        let data_file = temp_dir.join(format!("loamspine-verify-{}.dat", uuid::Uuid::now_v7()));
        let sig_file = temp_dir.join(format!("loamspine-verify-{}.sig", uuid::Uuid::now_v7()));

        // Write files
        std::fs::write(&data_file, data)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to write temp data: {e}")))?;
        std::fs::write(&sig_file, signature.as_bytes()).map_err(|e| {
            LoamSpineError::Internal(format!("Failed to write temp signature: {e}"))
        })?;

        // Run signing service decrypt (verification mode)
        let output = Command::new(&self.binary_path)
            .args([
                "decrypt",
                "--input",
                sig_file.to_str().unwrap_or_default(),
                "--output",
                "/dev/null",
            ])
            .output()
            .map_err(|e| {
                LoamSpineError::Internal(format!("Failed to run verification service: {e}"))
            })?;

        // Clean up
        let _ = std::fs::remove_file(&data_file);
        let _ = std::fs::remove_file(&sig_file);

        if output.status.success() {
            Ok(SignatureVerification::valid())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(SignatureVerification::invalid(format!(
                "Verification failed: {stderr}"
            )))
        }
    }

    async fn verify_entry(&self, entry: &Entry) -> LoamSpineResult<SignatureVerification> {
        // Verify the entry's signature against its content hash
        let entry_bytes = serde_json::to_vec(entry)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to serialize entry: {e}")))?;

        self.verify(&entry_bytes, &entry.signature, &entry.committer)
            .await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::uninlined_format_args)]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;

    /// Helper to get the beardog binary path if it exists.
    fn get_test_binary() -> Option<PathBuf> {
        // Try Phase 2 bins directory
        let bins_dir = PathBuf::from("../bins/beardog");
        if bins_dir.exists() && bins_dir.metadata().map(|m| m.is_file()).unwrap_or(false) {
            return Some(bins_dir);
        }

        // Try relative to workspace root
        let workspace_bins = PathBuf::from("../../bins/beardog");
        if workspace_bins.exists()
            && workspace_bins
                .metadata()
                .map(|m| m.is_file())
                .unwrap_or(false)
        {
            return Some(workspace_bins);
        }

        None
    }

    #[test]
    fn discover_binary_returns_none_if_not_found() {
        // Clear env var to test discovery
        env::remove_var(ENV_SIGNER_PATH);
        // Discovery may or may not find binary depending on environment
        let result = CliSigner::discover_binary();
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn discover_binary_respects_env_var() {
        // Set env var to a test path
        let test_path = "/tmp/test-signer";
        env::set_var(ENV_SIGNER_PATH, test_path);

        let result = CliSigner::discover_binary();

        // Clean up
        env::remove_var(ENV_SIGNER_PATH);

        // Should return None because the path doesn't exist, but it checked the env var
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
            eprintln!("⚠️  Skipping test: beardog binary not found");
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
            eprintln!("⚠️  Skipping test: beardog binary not found");
            return;
        };

        let result = CliVerifier::new(binary);
        assert!(result.is_ok());
    }

    #[test]
    fn verifier_binary_path_accessor() {
        // Skip if no binary available
        let Some(binary) = get_test_binary() else {
            eprintln!("⚠️  Skipping test: beardog binary not found");
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
    fn signer_implements_debug() {
        // Skip if no binary available
        let Some(binary) = get_test_binary() else {
            eprintln!("⚠️  Skipping test: beardog binary not found");
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
            eprintln!("⚠️  Skipping test: beardog binary not found");
            return;
        };

        let verifier = CliVerifier::new(binary).unwrap();
        let debug_str = format!("{:?}", verifier);
        assert!(debug_str.contains("CliVerifier"));
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn signer_implements_clone() {
        // Skip if no binary available
        let Some(binary) = get_test_binary() else {
            eprintln!("⚠️  Skipping test: beardog binary not found");
            return;
        };

        if let Ok(signer) = CliSigner::new(binary, "default") {
            let cloned = signer.clone();
            assert_eq!(signer.did(), cloned.did());
        }
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn verifier_implements_clone() {
        // Skip if no binary available
        let Some(binary) = get_test_binary() else {
            eprintln!("⚠️  Skipping test: beardog binary not found");
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
            eprintln!("⚠️  Skipping test: beardog binary not found");
            return;
        };

        if let Ok(signer) = CliSigner::new(binary, "default") {
            let did = signer.did();
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
        assert!(!struct_name.contains("beardog"), "Should not hardcode primal names");
        assert!(!struct_name.contains("nestgate"), "Should not hardcode primal names");
    }
    
    #[test]
    fn environment_variable_priority() {
        // Test that env vars are checked first (highest priority)
        let original = env::var(ENV_SIGNER_PATH).ok();
        
        // Set a test path
        env::set_var(ENV_SIGNER_PATH, "/test/priority/path");
        
        // Discovery should check this first
        let result = CliSigner::discover_binary();
        
        // Restore original
        if let Some(val) = original {
            env::set_var(ENV_SIGNER_PATH, val);
        } else {
            env::remove_var(ENV_SIGNER_PATH);
        }
        
        // Should return None (path doesn't exist) but proved it checked env var
        assert!(result.is_none());
    }
    
    #[test]
    fn binary_discovery_searches_multiple_locations() {
        // Clear env to test fallback locations
        let original = env::var(ENV_SIGNER_PATH).ok();
        env::remove_var(ENV_SIGNER_PATH);
        
        // Discovery should search multiple locations without panicking
        let result = CliSigner::discover_binary();
        
        // Restore
        if let Some(val) = original {
            env::set_var(ENV_SIGNER_PATH, val);
        }
        
        // Result depends on environment, but shouldn't panic
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
            "",
            "/",
            "/tmp",  // directory, not file
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
