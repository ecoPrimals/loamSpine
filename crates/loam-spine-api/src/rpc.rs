// SPDX-License-Identifier: AGPL-3.0-or-later

//! RPC trait definition for `LoamSpine`.
//!
//! This module defines the `LoamSpineRpc` trait using tarpc macros
//! for pure Rust, high-performance RPC.

use crate::error::ApiError;
use crate::types::{
    AnchorPublishRequest, AnchorPublishResponse, AnchorSliceRequest, AnchorSliceResponse,
    AnchorVerifyRequest, AnchorVerifyResponse, AppendEntryRequest, AppendEntryResponse,
    CheckoutSliceRequest, CheckoutSliceResponse, CommitBraidRequest, CommitBraidResponse,
    CommitSessionRequest, CommitSessionResponse, CreateSpineRequest, CreateSpineResponse,
    GenerateInclusionProofRequest, GenerateInclusionProofResponse, GetCertificateRequest,
    GetCertificateResponse, GetEntryRequest, GetEntryResponse, GetSpineRequest, GetSpineResponse,
    GetTipRequest, GetTipResponse, HealthCheckRequest, HealthCheckResponse, LoanCertificateRequest,
    LoanCertificateResponse, MintCertificateRequest, MintCertificateResponse,
    ReturnCertificateRequest, ReturnCertificateResponse, SealSpineRequest, SealSpineResponse,
    TransferCertificateRequest, TransferCertificateResponse, VerifyInclusionProofRequest,
    VerifyInclusionProofResponse,
};

/// Pure Rust RPC service trait for `LoamSpine`.
///
/// This trait is implemented using tarpc macros for high-performance
/// primal-to-primal communication, and also exposed via JSON-RPC
/// for external clients.
///
/// ## Design Philosophy
///
/// - **Pure Rust RPC**: Uses tarpc (JSON-over-TCP) for structured primal-to-primal calls, pure JSON-RPC for external clients
/// - **No gRPC/protobuf**: Maintains Rust-native toolchain sovereignty
/// - **Capability-based**: Methods map to primal capabilities
///
/// ## Method Categories
///
/// - **Spine Operations**: Create, get, seal spines
/// - **Entry Operations**: Append, query entries
/// - **Certificate Operations**: Mint, transfer, loan, return certificates
/// - **Slice Operations**: Waypoint anchoring and checkout
/// - **Proof Operations**: Inclusion proof generation and verification
/// - **Integration**: Session commits, braid commits (from other primals)
#[tarpc::service]
pub trait LoamSpineRpc {
    // ========================================================================
    // Spine Operations
    // ========================================================================

    /// Create a new spine.
    ///
    /// Creates a sovereign append-only ledger owned by a DID.
    async fn create_spine(request: CreateSpineRequest) -> Result<CreateSpineResponse, ApiError>;

    /// Get a spine by ID.
    ///
    /// Returns spine metadata including height, tip hash, and owner.
    async fn get_spine(request: GetSpineRequest) -> Result<GetSpineResponse, ApiError>;

    /// Seal a spine (make immutable).
    ///
    /// Once sealed, no more entries can be appended.
    async fn seal_spine(request: SealSpineRequest) -> Result<SealSpineResponse, ApiError>;

    // ========================================================================
    // Entry Operations
    // ========================================================================

    /// Append an entry to a spine.
    ///
    /// Entries are cryptographically linked to form an immutable chain.
    async fn append_entry(request: AppendEntryRequest) -> Result<AppendEntryResponse, ApiError>;

    /// Get an entry by hash.
    ///
    /// Returns the full entry data if found.
    async fn get_entry(request: GetEntryRequest) -> Result<GetEntryResponse, ApiError>;

    /// Get the tip entry of a spine.
    ///
    /// Returns the most recent entry in the chain.
    async fn get_tip(request: GetTipRequest) -> Result<GetTipResponse, ApiError>;

    // ========================================================================
    // Certificate Operations
    // ========================================================================

    /// Mint a new certificate.
    ///
    /// Creates a new digital ownership certificate on the spine.
    async fn mint_certificate(
        request: MintCertificateRequest,
    ) -> Result<MintCertificateResponse, ApiError>;

    /// Transfer a certificate.
    ///
    /// Transfers ownership to a new DID.
    async fn transfer_certificate(
        request: TransferCertificateRequest,
    ) -> Result<TransferCertificateResponse, ApiError>;

    /// Loan a certificate.
    ///
    /// Temporarily grants access to another party.
    async fn loan_certificate(
        request: LoanCertificateRequest,
    ) -> Result<LoanCertificateResponse, ApiError>;

    /// Return a loaned certificate.
    ///
    /// Returns the certificate to the owner.
    async fn return_certificate(
        request: ReturnCertificateRequest,
    ) -> Result<ReturnCertificateResponse, ApiError>;

    /// Get a certificate by ID.
    ///
    /// Returns certificate state and history.
    async fn get_certificate(
        request: GetCertificateRequest,
    ) -> Result<GetCertificateResponse, ApiError>;

    // ========================================================================
    // Slice/Waypoint Operations
    // ========================================================================

    /// Anchor a slice on a waypoint spine.
    ///
    /// Creates a reference to borrowed state from another spine.
    async fn anchor_slice(request: AnchorSliceRequest) -> Result<AnchorSliceResponse, ApiError>;

    /// Checkout a slice from a waypoint.
    ///
    /// Initiates a borrow operation with provenance tracking.
    async fn checkout_slice(
        request: CheckoutSliceRequest,
    ) -> Result<CheckoutSliceResponse, ApiError>;

    // ========================================================================
    // Proof Operations
    // ========================================================================

    /// Generate an inclusion proof.
    ///
    /// Creates a cryptographic proof that an entry exists in a spine.
    async fn generate_inclusion_proof(
        request: GenerateInclusionProofRequest,
    ) -> Result<GenerateInclusionProofResponse, ApiError>;

    /// Verify an inclusion proof.
    ///
    /// Validates a previously generated proof.
    async fn verify_inclusion_proof(
        request: VerifyInclusionProofRequest,
    ) -> Result<VerifyInclusionProofResponse, ApiError>;

    // ========================================================================
    // Public Chain Anchor Operations
    // ========================================================================

    /// Record a public chain anchor on a spine.
    ///
    /// Stores the receipt of anchoring a spine's state hash to an external
    /// append-only ledger. The actual chain submission is performed by a
    /// capability-discovered `"chain-anchor"` primal.
    async fn publish_anchor(
        request: AnchorPublishRequest,
    ) -> Result<AnchorPublishResponse, ApiError>;

    /// Verify a spine's state against a recorded public chain anchor.
    ///
    /// Checks that the recorded state hash matches the spine's actual state.
    async fn verify_anchor(request: AnchorVerifyRequest) -> Result<AnchorVerifyResponse, ApiError>;

    // ========================================================================
    // Health Operations
    // ========================================================================

    /// Health check.
    ///
    /// Returns service health status and component states.
    async fn health_check(request: HealthCheckRequest) -> Result<HealthCheckResponse, ApiError>;

    // ========================================================================
    // Ephemeral Storage Integration
    // ========================================================================

    /// Commit a session from an ephemeral storage primal.
    ///
    /// Permanently stores a dehydrated DAG session summary.
    async fn commit_session(
        request: CommitSessionRequest,
    ) -> Result<CommitSessionResponse, ApiError>;

    // ========================================================================
    // Semantic Attribution Integration
    // ========================================================================

    /// Commit a braid from a semantic attribution primal.
    ///
    /// Permanently stores semantic attribution information.
    async fn commit_braid(request: CommitBraidRequest) -> Result<CommitBraidResponse, ApiError>;
}
