// SPDX-License-Identifier: AGPL-3.0-or-later

//! Signing and verification traits with runtime discovery.
//!
//! These traits define the interface for cryptographic signing and verification.
//! The actual signing primal is discovered at runtime through the capability system.
//!
//! ## Design Philosophy
//!
//! Rather than hardcoding a dependency on a specific signing primal,
//! we define capability traits that any signing primal can implement. The actual
//! primal is discovered at runtime through the discovery system.
//!
//! ## Capability-Based Discovery
//!
//! A primal that wants to use signing services:
//! 1. Requests the `Signer` capability from the discovery system
//! 2. Receives a boxed trait object implementing `Signer`
//! 3. Uses it without knowing which primal provides it
//!
//! This enables:
//! - Loose coupling between primals
//! - Runtime primal substitution (e.g., for testing)
//! - Multiple signing primals coexisting
//! - Graceful degradation when signing is unavailable

use crate::entry::Entry;
use crate::error::LoamSpineResult;
use crate::types::{Did, Signature, Timestamp};

/// Signature verification result.
#[derive(Clone, Debug)]
pub struct SignatureVerification {
    /// Whether verification passed.
    pub valid: bool,
    /// Verification timestamp.
    pub verified_at: Timestamp,
    /// Error message (if failed).
    pub error: Option<String>,
}

impl SignatureVerification {
    /// Create a successful verification result.
    #[must_use]
    pub fn valid() -> Self {
        Self {
            valid: true,
            verified_at: Timestamp::now(),
            error: None,
        }
    }

    /// Create a failed verification result.
    #[must_use]
    pub fn invalid(reason: impl Into<String>) -> Self {
        Self {
            valid: false,
            verified_at: Timestamp::now(),
            error: Some(reason.into()),
        }
    }
}

/// Signer capability trait.
///
/// This trait is implemented by primals that provide signing services.
/// The actual implementation is discovered at runtime.
///
/// # Example
///
/// ```rust,no_run
/// use loam_spine_core::discovery::CapabilityRegistry;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Request a signer from discovery
/// let registry = CapabilityRegistry::new();
/// if let Some(signer) = registry.get_signer().await {
///     // Use it without knowing which primal provides it
///     let data = loam_spine_core::types::ByteBuffer::from_static(b"data");
///     let signature = signer.sign_boxed(data).await?;
/// }
/// # Ok(())
/// # }
/// ```
pub trait Signer: Send + Sync {
    /// Sign data.
    fn sign(
        &self,
        data: &[u8],
    ) -> impl std::future::Future<Output = LoamSpineResult<Signature>> + Send;

    /// Get the signer's DID.
    fn did(&self) -> &Did;
}

/// Verifier capability trait.
///
/// This trait is implemented by primals that provide signature verification.
/// The actual implementation is discovered at runtime.
pub trait Verifier: Send + Sync {
    /// Verify a signature.
    fn verify(
        &self,
        data: &[u8],
        signature: &Signature,
        signer: &Did,
    ) -> impl std::future::Future<Output = LoamSpineResult<SignatureVerification>> + Send;

    /// Verify an entry signature.
    fn verify_entry(
        &self,
        entry: &Entry,
    ) -> impl std::future::Future<Output = LoamSpineResult<SignatureVerification>> + Send;
}

// ============================================================================
// Test utilities - only available in test builds
// ============================================================================

/// Test utilities for signing traits.
///
/// These are isolated to test builds only and should never be used in production.
#[cfg(any(test, feature = "testing"))]
pub mod testing {
    use super::{Did, Entry, LoamSpineResult, Signature, SignatureVerification, Signer, Verifier};

    /// Mock signer for testing.
    ///
    /// This should ONLY be used in tests. Production code should use
    /// runtime discovery to obtain a real signer.
    #[derive(Clone, Debug)]
    pub struct MockSigner {
        did: Did,
    }

    impl MockSigner {
        /// Create a new mock signer.
        #[must_use]
        pub const fn new(did: Did) -> Self {
            Self { did }
        }
    }

    impl Signer for MockSigner {
        async fn sign(&self, data: &[u8]) -> LoamSpineResult<Signature> {
            // Simple mock: hash the data as signature
            let hash = crate::types::hash_bytes(data);
            Ok(Signature::from_vec(hash.to_vec()))
        }

        fn did(&self) -> &Did {
            &self.did
        }
    }

    /// Mock verifier for testing.
    ///
    /// This should ONLY be used in tests. Production code should use
    /// runtime discovery to obtain a real verifier.
    #[derive(Clone, Debug, Default)]
    pub struct MockVerifier {
        /// Always return valid.
        pub always_valid: bool,
    }

    impl MockVerifier {
        /// Create a permissive mock verifier (always passes).
        #[must_use]
        pub const fn permissive() -> Self {
            Self { always_valid: true }
        }

        /// Create a strict mock verifier (always fails).
        #[must_use]
        pub const fn strict() -> Self {
            Self {
                always_valid: false,
            }
        }
    }

    impl Verifier for MockVerifier {
        async fn verify(
            &self,
            _data: &[u8],
            _signature: &Signature,
            _signer: &Did,
        ) -> LoamSpineResult<SignatureVerification> {
            Ok(if self.always_valid {
                SignatureVerification::valid()
            } else {
                SignatureVerification::invalid("mock verification failed")
            })
        }

        async fn verify_entry(&self, _entry: &Entry) -> LoamSpineResult<SignatureVerification> {
            Ok(if self.always_valid {
                SignatureVerification::valid()
            } else {
                SignatureVerification::invalid("mock verification failed")
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_verification_valid() {
        let result = SignatureVerification::valid();
        assert!(result.valid);
        assert!(result.error.is_none());
    }

    #[test]
    fn signature_verification_invalid() {
        let result = SignatureVerification::invalid("test failure");
        assert!(!result.valid);
        assert_eq!(result.error, Some("test failure".to_string()));
    }

    #[test]
    fn signature_verification_debug_clone() {
        let result = SignatureVerification::valid();
        let debug_str = format!("{result:?}");
        assert!(debug_str.contains("SignatureVerification"));

        #[allow(clippy::redundant_clone)]
        let cloned = result.clone();
        assert_eq!(result.valid, cloned.valid);
    }

    #[tokio::test]
    async fn mock_signer_signs_data() {
        use testing::MockSigner;

        let did = Did::new("did:key:z6MkTest");
        let signer = MockSigner::new(did.clone());

        let data = b"test data";
        let signature = signer.sign(data).await.unwrap_or_else(|_| unreachable!());

        // Signature should be non-empty
        assert!(!signature.0.is_empty());

        // Same data should produce same signature
        let sig2 = signer.sign(data).await.unwrap_or_else(|_| unreachable!());
        assert_eq!(signature, sig2);

        // Different data should produce different signature
        let sig3 = signer
            .sign(b"different")
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_ne!(signature, sig3);

        // Check DID
        assert_eq!(signer.did(), &did);
    }

    #[tokio::test]
    async fn mock_verifier_permissive() {
        use testing::MockVerifier;

        let verifier = MockVerifier::permissive();
        let data = b"test";
        let sig = Signature::from_vec(vec![1, 2, 3]);
        let did = Did::new("did:key:test");

        let result = verifier
            .verify(data, &sig, &did)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(result.valid);
    }

    #[tokio::test]
    async fn mock_verifier_strict() {
        use testing::MockVerifier;

        let verifier = MockVerifier::strict();
        let data = b"test";
        let sig = Signature::from_vec(vec![1, 2, 3]);
        let did = Did::new("did:key:test");

        let result = verifier
            .verify(data, &sig, &did)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(!result.valid);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn mock_verifier_entry_permissive() {
        use crate::entry::{Entry, EntryType};
        use testing::MockVerifier;

        let verifier = MockVerifier::permissive();
        let entry = Entry::new(
            0,
            None,
            Did::new("did:test"),
            EntryType::SpineSealed { reason: None },
        );

        let result = verifier
            .verify_entry(&entry)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(result.valid);
    }

    #[tokio::test]
    async fn mock_verifier_entry_strict() {
        use crate::entry::{Entry, EntryType};
        use testing::MockVerifier;

        let verifier = MockVerifier::strict();
        let entry = Entry::new(
            0,
            None,
            Did::new("did:test"),
            EntryType::SpineSealed { reason: None },
        );

        let result = verifier
            .verify_entry(&entry)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(!result.valid);
    }

    #[test]
    fn mock_verifier_default() {
        use testing::MockVerifier;

        let verifier = MockVerifier::default();
        assert!(!verifier.always_valid);
    }

    #[test]
    fn mock_signer_debug_clone() {
        use testing::MockSigner;

        let signer = MockSigner::new(Did::new("did:test"));
        let debug_str = format!("{signer:?}");
        assert!(debug_str.contains("MockSigner"));

        #[allow(clippy::redundant_clone)]
        let cloned = signer.clone();
        assert_eq!(signer.did(), cloned.did());
    }

    #[test]
    fn mock_verifier_debug_clone() {
        use testing::MockVerifier;

        let verifier = MockVerifier::permissive();
        let debug_str = format!("{verifier:?}");
        assert!(debug_str.contains("MockVerifier"));

        #[allow(clippy::redundant_clone)]
        let cloned = verifier.clone();
        assert_eq!(verifier.always_valid, cloned.always_valid);
    }
}
