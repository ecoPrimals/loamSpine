// SPDX-License-Identifier: AGPL-3.0-or-later

//! RPC trait definition for `LoamSpine`.
//!
//! This module defines the `LoamSpineRpc` trait using tarpc macros
//! for pure Rust, high-performance RPC.

use crate::error::ApiError;
use crate::types::{
    AnchorPublishBatchRequest, AnchorPublishBatchResponse, AnchorPublishRequest,
    AnchorPublishResponse, AnchorSliceRequest, AnchorSliceResponse, AnchorVerifyRequest,
    AnchorVerifyResponse, AppendEntryBatchRequest, AppendEntryBatchResponse, AppendEntryRequest,
    AppendEntryResponse, BondLedgerListRequest, BondLedgerListResponse, BondLedgerRetrieveRequest,
    BondLedgerRetrieveResponse, BondLedgerStoreRequest, BondLedgerStoreResponse,
    BtspNegotiateRequest, BtspNegotiateResponse, CertificateHistoryRequest,
    CertificateHistoryResponse, CertificateLifecycleRequest, CertificateLifecycleResponse,
    CheckoutSliceRequest, CheckoutSliceResponse, CommitBraidRequest, CommitBraidResponse,
    CommitSessionRequest, CommitSessionResponse, CreateSpineRequest, CreateSpineResponse,
    DehydrateSessionRequest, DehydrateSessionResponse, GenerateInclusionProofRequest,
    GenerateInclusionProofResponse, GetCertificateRequest, GetCertificateResponse, GetEntryRequest,
    GetEntryResponse, GetSpineRequest, GetSpineResponse, GetTipRequest, GetTipResponse,
    HealthCheckRequest, HealthCheckResponse, ListEntriesRequest, ListEntriesResponse,
    ListSpinesRequest, ListSpinesResponse, LoanCertificateRequest, LoanCertificateResponse,
    MintCertificateBatchRequest, MintCertificateBatchResponse, MintCertificateRequest,
    MintCertificateResponse, ReturnCertificateRequest, ReturnCertificateResponse, SealSpineRequest,
    SealSpineResponse, SpineStatusRequest, SpineStatusResponse, TransferCertificateRequest,
    TransferCertificateResponse, TrustAnchorRequest, TrustAnchorResponse, TrustEventCountRequest,
    TrustEventCountResponse, TrustQueryRequest, TrustQueryResponse, VerifyCertificateRequest,
    VerifyCertificateResponse, VerifyInclusionProofRequest, VerifyInclusionProofResponse,
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

    /// List all spine IDs in the store.
    async fn list_spines(request: ListSpinesRequest) -> Result<ListSpinesResponse, ApiError>;

    /// Get comprehensive spine status for observability.
    ///
    /// Reports entry count, tip/genesis hashes, state, timestamps, and
    /// all associated sessions with Merkle roots.
    async fn spine_status(request: SpineStatusRequest) -> Result<SpineStatusResponse, ApiError>;

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

    /// Append multiple entries to a spine in a single batch (amortized I/O).
    async fn append_entry_batch(
        request: AppendEntryBatchRequest,
    ) -> Result<AppendEntryBatchResponse, ApiError>;

    /// Get an entry by hash.
    ///
    /// Returns the full entry data if found.
    async fn get_entry(request: GetEntryRequest) -> Result<GetEntryResponse, ApiError>;

    /// Get the tip entry of a spine.
    ///
    /// Returns the most recent entry in the chain.
    async fn get_tip(request: GetTipRequest) -> Result<GetTipResponse, ApiError>;

    /// List entries in a spine (paginated).
    async fn list_entries(request: ListEntriesRequest) -> Result<ListEntriesResponse, ApiError>;

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

    /// Mint multiple certificates in a single batch (amortized I/O).
    async fn mint_certificate_batch(
        request: MintCertificateBatchRequest,
    ) -> Result<MintCertificateBatchResponse, ApiError>;

    /// Get a certificate by ID.
    ///
    /// Returns certificate state and history.
    async fn get_certificate(
        request: GetCertificateRequest,
    ) -> Result<GetCertificateResponse, ApiError>;

    /// Verify a certificate's integrity and provenance.
    async fn verify_certificate(
        request: VerifyCertificateRequest,
    ) -> Result<VerifyCertificateResponse, ApiError>;

    /// Get ordered lifecycle entries for a certificate.
    async fn certificate_lifecycle(
        request: CertificateLifecycleRequest,
    ) -> Result<CertificateLifecycleResponse, ApiError>;

    /// Get structured certificate history with ownership and loan records.
    async fn certificate_history(
        request: CertificateHistoryRequest,
    ) -> Result<CertificateHistoryResponse, ApiError>;

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

    /// Record an aggregate batch anchor across multiple spines.
    async fn publish_anchor_batch(
        request: AnchorPublishBatchRequest,
    ) -> Result<AnchorPublishBatchResponse, ApiError>;

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

    /// Dehydrate a session — compute content-addressed summary without committing.
    async fn dehydrate_session(
        request: DehydrateSessionRequest,
    ) -> Result<DehydrateSessionResponse, ApiError>;

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

    // ========================================================================
    // Bond Ledger Operations
    // ========================================================================

    /// Store a bond record in the ledger.
    ///
    /// Persists ionic bond state from the `crypto.sign_contract` flow.
    async fn bond_ledger_store(
        request: BondLedgerStoreRequest,
    ) -> Result<BondLedgerStoreResponse, ApiError>;

    /// Retrieve a bond record by ID.
    ///
    /// Returns the most recent data for the given bond identifier.
    async fn bond_ledger_retrieve(
        request: BondLedgerRetrieveRequest,
    ) -> Result<BondLedgerRetrieveResponse, ApiError>;

    /// List all stored bond identifiers.
    async fn bond_ledger_list(
        request: BondLedgerListRequest,
    ) -> Result<BondLedgerListResponse, ApiError>;

    // ========================================================================
    // Cross-Gate Trust Operations
    // ========================================================================

    /// Anchor a cross-gate trust event as a permanent ledger entry.
    async fn trust_anchor(request: TrustAnchorRequest) -> Result<TrustAnchorResponse, ApiError>;

    /// Query trust events involving a specific gate DID.
    async fn trust_query(request: TrustQueryRequest) -> Result<TrustQueryResponse, ApiError>;

    /// Return the number of trust events in the ledger.
    async fn trust_event_count(
        request: TrustEventCountRequest,
    ) -> Result<TrustEventCountResponse, ApiError>;

    // ========================================================================
    // BTSP Phase 3 Negotiation
    // ========================================================================

    /// Negotiate BTSP Phase 3 cipher suite for encrypted post-handshake channel.
    async fn negotiate_btsp(
        request: BtspNegotiateRequest,
    ) -> Result<BtspNegotiateResponse, ApiError>;
}
