// SPDX-License-Identifier: AGPL-3.0-or-later

//! Object-safe wrappers for `Signer`, `Verifier`, and `AttestationProvider` traits.
//!
//! These wrappers use dynamic dispatch (`Pin<Box<dyn Future>>`) to allow
//! storing heterogeneous implementations behind a single `Arc<dyn Dyn*>`.
//!
//! Blanket implementations convert any concrete `Signer`/`Verifier` into
//! the corresponding `Dyn*` trait automatically.

use std::sync::Arc;

use crate::error::LoamSpineResult;
use crate::traits::{Signer, Verifier};
use crate::waypoint::{AttestationContext, AttestationResult};

/// A boxed signer that can be stored and shared.
pub type BoxedSigner = Arc<dyn DynSigner>;

/// A boxed verifier that can be stored and shared.
pub type BoxedVerifier = Arc<dyn DynVerifier>;

/// Object-safe version of Signer for dynamic dispatch.
///
/// Uses `bytes::Bytes` for zero-copy data passing across async boundaries.
/// Methods return `Pin<Box<dyn Future>>` rather than `async fn` for object safety.
pub trait DynSigner: Send + Sync {
    /// Sign data (takes `Bytes` for zero-copy object safety).
    fn sign_boxed(
        &self,
        data: crate::types::ByteBuffer,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = LoamSpineResult<crate::types::Signature>> + Send + '_>,
    >;

    /// Get the signer's DID.
    fn did(&self) -> &crate::types::Did;
}

/// Blanket implementation for any Signer.
impl<T: Signer> DynSigner for T {
    fn sign_boxed(
        &self,
        data: crate::types::ByteBuffer,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = LoamSpineResult<crate::types::Signature>> + Send + '_>,
    > {
        Box::pin(async move { self.sign(&data).await })
    }

    fn did(&self) -> &crate::types::Did {
        Signer::did(self)
    }
}

/// Object-safe version of Verifier for dynamic dispatch.
///
/// Uses `bytes::Bytes` for zero-copy data passing across async boundaries.
/// Methods return `Pin<Box<dyn Future>>` rather than `async fn` for object safety.
pub trait DynVerifier: Send + Sync {
    /// Verify a signature (takes `Bytes` for zero-copy object safety).
    fn verify_boxed(
        &self,
        data: crate::types::ByteBuffer,
        signature: crate::types::Signature,
        signer: crate::types::Did,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = LoamSpineResult<crate::traits::SignatureVerification>>
                + Send
                + '_,
        >,
    >;

    /// Verify an entry signature (takes owned entry for object safety).
    fn verify_entry_boxed(
        &self,
        entry: crate::entry::Entry,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = LoamSpineResult<crate::traits::SignatureVerification>>
                + Send
                + '_,
        >,
    >;
}

/// Blanket implementation for any Verifier.
impl<T: Verifier> DynVerifier for T {
    fn verify_boxed(
        &self,
        data: crate::types::ByteBuffer,
        signature: crate::types::Signature,
        signer: crate::types::Did,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = LoamSpineResult<crate::traits::SignatureVerification>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move { self.verify(&data, &signature, &signer).await })
    }

    fn verify_entry_boxed(
        &self,
        entry: crate::entry::Entry,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = LoamSpineResult<crate::traits::SignatureVerification>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move { self.verify_entry(&entry).await })
    }
}

/// A boxed attestation provider that can be stored and shared.
pub type BoxedAttestationProvider = Arc<dyn DynAttestationProvider>;

/// Object-safe attestation provider for waypoint operation attestation.
///
/// Discovered at runtime via capability identifier `ATTESTATION`.
/// Sends JSON-RPC `attestation.request` to capability-discovered endpoint.
pub trait DynAttestationProvider: Send + Sync {
    /// Request attestation for a waypoint operation.
    fn request_attestation(
        &self,
        context: AttestationContext,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = LoamSpineResult<AttestationResult>> + Send + '_>,
    >;
}
