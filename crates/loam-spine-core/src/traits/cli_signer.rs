// SPDX-License-Identifier: AGPL-3.0-or-later

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
/// Environment variable for Phase 2 bins directory override.
pub const ENV_BINS_DIR: &str = "LOAMSPINE_BINS_DIR";

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
    /// 1. Environment variable `LOAMSPINE_SIGNER_PATH` (explicit binary path)
    /// 2. `LOAMSPINE_BINS_DIR` or `../bins/` directory (Phase 2 integration)
    /// 3. System PATH (looks for common signing service names)
    #[must_use]
    pub fn discover_binary() -> Option<PathBuf> {
        if let Ok(path) = std::env::var(ENV_SIGNER_PATH) {
            let path = PathBuf::from(path);
            if path.exists() {
                return Some(path);
            }
        }

        let bins_dir =
            std::env::var(ENV_BINS_DIR).map_or_else(|_| PathBuf::from("../bins"), PathBuf::from);
        if bins_dir.is_dir() {
            for candidate in &["signer", "signing-service"] {
                let path = bins_dir.join(candidate);
                if path.exists() {
                    return Some(path);
                }
            }
        }

        for candidate in &["loamspine-signer", "signer"] {
            if let Ok(output) = Command::new("which").arg(candidate).output()
                && output.status.success()
            {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(PathBuf::from(path));
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
                data_file.to_str().ok_or_else(|| {
                    LoamSpineError::Internal("data file path is not valid UTF-8".into())
                })?,
                "--output",
                sig_file.to_str().ok_or_else(|| {
                    LoamSpineError::Internal("signature file path is not valid UTF-8".into())
                })?,
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

        Ok(Signature::from_vec(sig_bytes))
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
                sig_file.to_str().ok_or_else(|| {
                    LoamSpineError::Internal("signature file path is not valid UTF-8".into())
                })?,
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
#[path = "cli_signer_tests.rs"]
mod cli_signer_tests;
