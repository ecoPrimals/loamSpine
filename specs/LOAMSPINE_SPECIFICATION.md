<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# LoamSpine — Permanent Ledger Specification

**Version:** 1.1.0  
**Status:** Active  
**Author:** ecoPrimals Project  
**Date:** December 2025  
**License:** AGPL-3.0

> **Implementation Note**: The LoamSpine codebase uses capability-based discovery.
> External primals are discovered at runtime via environment variables and the
> `CapabilityRegistry`. No primal names are hardcoded in the source code.
> This spec describes the *ecosystem architecture* for documentation purposes.  

---

## Abstract

LoamSpine is the immutable, permanent ledger of the ecoPrimals ecosystem. Named after **loam**—the slow, anaerobic soil layer where organic matter is compressed into permanent geological record—LoamSpine serves as the "fossil record" and canonical source of truth for all events, discoveries, and artifacts that matter.

Unlike RhizoCrypt's ephemeral branching (the rhizome above), LoamSpine provides **selective permanence**: committing to LoamSpine is a deliberate, meaningful act that canonizes data into an unalterable linear history. The two layers work together—rhizomes explore and branch above, while the loam below slowly accumulates what matters.

LoamSpine is not a blockchain in the traditional sense. It is a **sovereign, federated ledger** designed for:
- Individual sovereignty (your own LoamSpine, your own history)
- Federated verification (others can verify your claims)
- Recursive stacking (spines can reference other spines)
- Efficient scaling (summaries, not raw data)
- **Slice anchoring** (temporary waypoints for borrowed state)

---

## 0. Biological Model: The Loam Layer

LoamSpine's architecture is modeled on soil loam—the deep, anaerobic layer where:

- **Organic matter compresses** — Only what matters becomes permanent
- **Time moves slowly** — Changes are deliberate, not reactive
- **Fossils form** — The permanent record of what once lived above
- **Nutrients cycle** — Value can be extracted and returned to the surface

```
    RhizoCrypt (Rhizome Layer)
    ══════════════════════════
         ○──○──○    ○──○
        /        \  /    \
       ○    ○──○──○──○    ○     ← Ephemeral branching
        \  /          \  /
         ○─────────────○
              │
              │ Dehydration (selective commitment)
              ▼
    ══════════════════════════
    LoamSpine (Anaerobic Layer)
    
    [Genesis]──[Entry]──[Entry]──[Entry]──[Tip]
                                     ↑
                            Linear, permanent
```

This biological metaphor informs the architecture:

| Loam Property | LoamSpine Implementation |
|---------------|--------------------------|
| Slow accumulation | Deliberate, signed commits only |
| Compression | Dehydration summaries, not raw DAG data |
| Fossil record | Immutable entry chain with Merkle proofs |
| Anaerobic stability | No modification, only append |
| Nutrient cycling | Slice lending with waypoint anchoring |
| Geological layers | Recursive spine stacking |

---

## 1. Core Principles

### 1.1 Selective Permanence

LoamSpine embodies the **Philosophy of Forgetting's** complement: **selective remembering**. Most data should be ephemeral (RhizoCrypt). Only what is deliberately committed to LoamSpine becomes permanent.

Committing to LoamSpine is:
- **Deliberate** — Requires explicit action
- **Meaningful** — Represents a significant event
- **Expensive** — Computationally and semantically
- **Irreversible** — Cannot be modified or deleted

### 1.2 The Museum Analogy

If RhizoCrypt is the workshop where creative chaos happens, LoamSpine is the museum where finished works are preserved:

| RhizoCrypt | LoamSpine |
|------------|-----------|
| Every sketch, draft, iteration | The final masterpiece |
| Every shot fired in a raid | The validated extraction |
| Every experimental result | The published finding |
| Working memory | Permanent record |

### 1.3 Sovereign Spines

Each user, organization, or community maintains their own LoamSpine(s):

- **Personal Spine** — Your private history
- **Professional Spine** — Your work contributions
- **Community Spine** — Shared group history
- **Public Spine** — Globally verifiable claims

This sovereignty means no central authority controls what gets recorded. You own your history.

### 1.4 Recursive Stacking

LoamSpines can reference other LoamSpines, creating a hierarchical structure:

```
┌─────────────────────────────────────────────────┐
│              Global Commons Spine               │
│         (Aggregated community hashes)           │
└───────────────────┬─────────────────────────────┘
                    │
      ┌─────────────┼─────────────┐
      │             │             │
      ▼             ▼             ▼
┌──────────┐  ┌──────────┐  ┌──────────┐
│Community │  │Community │  │Community │
│ Spine A  │  │ Spine B  │  │ Spine C  │
└────┬─────┘  └────┬─────┘  └────┬─────┘
     │             │             │
   ┌─┴──┐        ┌─┴──┐        ┌─┴──┐
   │    │        │    │        │    │
   ▼    ▼        ▼    ▼        ▼    ▼
┌────┐┌────┐  ┌────┐┌────┐  ┌────┐┌────┐
│User││User│  │User││User│  │User││User│
│ 1  ││ 2  │  │ 3  ││ 4  │  │ 5  ││ 6  │
└────┘└────┘  └────┘└────┘  └────┘└────┘
```

A local spine can be finalized, hashed, and that single hash committed to a higher-level spine. This allows communities to share history without forcing everyone to carry the full burden.

---

## 2. Waypoint Spines & Slice Anchoring

A key innovation of the LoamSpine/RhizoCrypt layering is the **waypoint spine**—a localized linear anchor that can temporarily hold slices from other spines, enabling asynchronous operations while maintaining provenance.

### 2.1 The Waypoint Concept

When you lend a digital asset (slice) to a friend:

1. The slice originates from your LoamSpine
2. It travels through a RhizoCrypt DAG (the transit layer)
3. It can **anchor** to your friend's local LoamSpine (the waypoint)
4. Your friend operates on it locally with full lineage
5. A return DAG brings it back to your original spine

**Crucially**: The waypoint spine provides local permanence for the borrower's operations, but **cannot propagate upward** to the lender's parent spines or the global commons. This is "consignment without ownership transfer."

```
                    ┌────────────────┐
                    │   gAIa/Global  │  ← Cannot be affected by waypoint
                    │    Commons     │
                    └───────┬────────┘
                            │
                    ┌───────┴────────┐
                    │ ALICE's Spine  │  ← Owner's canonical spine
                    │  (Canonical)   │
                    └───────┬────────┘
                            │
                      [Slice lent]
                            │
                            ▼
                    ┌───────────────┐
                    │  Transit DAG  │  ← RhizoCrypt handles async
                    │  (RhizoCrypt) │
                    └───────┬───────┘
                            │
                      [Waypoint anchor]
                            │
                            ▼
                    ┌───────────────┐
                    │  BOB's Spine  │  ← Waypoint: local permanence
                    │  (Waypoint)   │     for Bob's operations
                    └───────┬───────┘
                            │
                      [Return DAG]
                            │
                            ▼
                    ┌───────────────┐
                    │ ALICE's Spine │  ← Slice returns, possibly
                    │  (Updated)    │     with usage record
                    └───────────────┘
```

### 2.2 Waypoint Entry Types

Waypoint spines use special entry types to track borrowed state:

```rust
/// Waypoint-specific entry types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WaypointEntryType {
    /// Slice arrives at waypoint
    SliceAnchor {
        /// The incoming slice
        slice: SliceRef,
        /// Origin spine
        origin_spine: SpineId,
        /// Origin entry
        origin_entry: EntryHash,
        /// Lending terms
        terms: LoanTerms,
        /// Arrival timestamp
        anchored_at: Timestamp,
    },
    
    /// Operations performed on anchored slice
    SliceOperation {
        /// The slice being operated on
        slice: SliceRef,
        /// Operation type
        operation: SliceOperationType,
        /// Operation payload
        payload: PayloadRef,
    },
    
    /// Slice departs waypoint (returning or transferring)
    SliceDeparture {
        /// The departing slice
        slice: SliceRef,
        /// Departure reason
        reason: DepartureReason,
        /// Return route
        destination: ResolutionRoute,
        /// Summary of operations performed
        operation_summary: OperationSummary,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DepartureReason {
    /// Loan term expired
    TermExpired,
    /// Borrower initiated return
    VoluntaryReturn,
    /// Owner recalled slice
    OwnerRecall,
    /// Transfer to new owner (e.g., auction sold)
    Transfer { new_owner: Did },
    /// Slice conditions triggered departure
    ConditionTriggered { condition: String },
}
```

### 2.3 Waypoint Hierarchy & Propagation Rules

Waypoint spines have **limited propagation rights**:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaypointConfig {
    /// This spine can serve as a waypoint
    pub is_waypoint: bool,
    
    /// Maximum depth of re-anchoring (sub-lending)
    pub max_anchor_depth: Option<u32>,
    
    /// Spines this waypoint can anchor to (for sub-lending)
    pub allowed_sub_waypoints: Vec<SpineId>,
    
    /// Whether operations here can propagate to origin
    pub propagate_to_origin: PropagationPolicy,
    
    /// Whether operations here can propagate upward (to parent spines)
    pub propagate_upward: PropagationPolicy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PropagationPolicy {
    /// Never propagate
    Never,
    
    /// Propagate summary only (e.g., "used for 10 hours")
    SummaryOnly,
    
    /// Propagate specific event types only
    Selective { allowed_types: Vec<String> },
    
    /// Full propagation (rare, requires explicit consent)
    Full,
}
```

**Key rule**: By default, waypoint operations **do not propagate upward**. A friend's game saves on their waypoint spine cannot modify your canonical spine or global commons. This prevents "pollution" of the ownership chain while still providing local provenance.

### 2.4 Reversible Transactions via Layering

The DAG/Linear layering enables **reversible transactions**:

```
═══════════════════════════════════════════════════════════════
                    STATE MACHINE
═══════════════════════════════════════════════════════════════

  CANONICAL SPINE                           CANONICAL SPINE
  (Before)                                  (After)
       │                                         │
       │                                         │
       ▼                                         ▼
  ┌─────────┐                              ┌─────────┐
  │ Entry N │                              │ Entry N │
  └────┬────┘                              └────┬────┘
       │                                        │
       │     ┌──────────────────┐               │
       └────►│   RhizoCrypt     │───────────────┘
             │      DAG         │          (if COMMIT)
             │                  │
             │  [Operations]    │          OR
             │  [Tentative]     │
             │  [Reversible]    │               │
             └────────┬─────────┘               │
                      │                         ▼
                      │                    (unchanged)
                      └────────────────────(if ROLLBACK)

═══════════════════════════════════════════════════════════════
```

The DAG is the **tentative layer** where:
- Multiple outcomes can be explored
- Operations are reversible until resolution
- Async coordination can occur
- Time-limited holds are possible

Resolution collapses the DAG into either:
- **COMMIT**: New entry appended to canonical spine
- **ROLLBACK**: Spine unchanged, DAG discarded

### 2.5 External Anchors (Optional)

While LoamSpine is sovereign by default, it can optionally anchor to external systems for additional verification:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExternalAnchor {
    /// Anchor to gAIa global commons
    GaiaCommons { 
        shard_id: GaiaShardId,
    },
    
    /// Anchor to another federated LoamSpine
    FederatedSpine { 
        spine_id: SpineId,
        peer_id: PeerId,
    },
    
    /// Anchor to external blockchain (optional, not required)
    ExternalChain {
        chain: ChainType,
        tx_hash: TxHash,
    },
    
    /// Anchor to data commons (preferred over currency chains)
    DataCommons {
        commons_id: CommonsId,
        anchor_hash: ContentHash,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChainType {
    /// Bitcoin (high security, slow, expensive)
    Bitcoin,
    /// Ethereum (programmable, moderate cost)
    Ethereum,
    /// Other chains as needed
    Other { chain_name: String },
}
```

**Philosophy**: We prefer anchoring to **data commons** (gAIa, federated spines) rather than currency chains (BTC, ETH). This provides:
- Self-sovereign verification (no external dependency)
- Zero transaction costs for network participants
- Alignment with ecoPrimals values (data > currency)
- Federated trust rather than economic consensus

Currency chains remain an **option** for use cases requiring external witnesses, but are not the default.

---

## 3. Data Model

### 2.1 Entry Structure

```rust
/// A single entry in a LoamSpine
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoamEntry {
    /// Sequential index within this spine
    pub index: u64,
    
    /// Hash of the previous entry (empty for genesis)
    pub previous: Option<EntryHash>,
    
    /// Timestamp of commitment
    pub timestamp: Timestamp,
    
    /// The agent committing this entry
    pub committer: Did,
    
    /// Entry type
    pub entry_type: EntryType,
    
    /// Entry payload (type-specific)
    pub payload: EntryPayload,
    
    /// Cryptographic signature from committer
    pub signature: Signature,
    
    /// Additional attestations (witnesses, validators)
    pub attestations: Vec<Attestation>,
    
    /// Hash of this entry (computed)
    pub hash: EntryHash,
}

/// Entry hash (Blake3)
pub type EntryHash = [u8; 32];

/// Canonical entry for hashing (excludes computed fields)
impl LoamEntry {
    pub fn compute_hash(&self) -> EntryHash {
        let canonical = CanonicalEntry {
            index: self.index,
            previous: self.previous,
            timestamp: self.timestamp,
            committer: self.committer.clone(),
            entry_type: self.entry_type.clone(),
            payload: self.payload.clone(),
            signature: self.signature.clone(),
            attestations: self.attestations.clone(),
        };
        blake3::hash(&canonical.to_canonical_bytes()).into()
    }
}
```

### 2.2 Entry Types

```rust
/// Types of entries that can be committed to LoamSpine
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EntryType {
    // === Spine Lifecycle ===
    /// Genesis entry (first in spine)
    Genesis { 
        spine_id: SpineId,
        owner: Did,
        config: SpineConfig,
    },
    
    /// Spine metadata update
    MetadataUpdate { 
        field: MetadataField,
        value: Value,
    },
    
    // === RhizoCrypt Commits ===
    /// Dehydrated RhizoCrypt session
    SessionCommit {
        session_id: SessionId,
        session_type: SessionType,
        merkle_root: MerkleRoot,
        summary: DehydrationSummary,
    },
    
    // === Data Anchoring ===
    /// Anchor a content hash
    DataAnchor {
        data_hash: ContentHash,
        mime_type: String,
        size: u64,
        metadata: HashMap<String, Value>,
    },
    
    /// SweetGrass Braid commitment
    BraidCommit {
        braid_id: BraidId,
        braid_hash: ContentHash,
        subject_hash: ContentHash,
    },
    
    // === Ownership & Transfer ===
    /// Mint a new Loam certificate
    CertificateMint {
        cert_id: CertificateId,
        cert_type: CertificateType,
        initial_owner: Did,
        metadata: CertificateMetadata,
    },
    
    /// Transfer certificate ownership
    CertificateTransfer {
        cert_id: CertificateId,
        from: Did,
        to: Did,
        conditions: Option<TransferConditions>,
    },
    
    /// Loan/lending of a certificate
    CertificateLoan {
        cert_id: CertificateId,
        lender: Did,
        borrower: Did,
        terms: LoanTerms,
    },
    
    /// Return of loaned certificate
    CertificateReturn {
        cert_id: CertificateId,
        loan_entry: EntryHash,
    },
    
    // === Recursive Stacking ===
    /// Reference to another spine's entry
    SpineReference {
        referenced_spine: SpineId,
        referenced_entry: EntryHash,
        reference_type: ReferenceType,
    },
    
    /// Roll-up of multiple entries into single hash
    Rollup {
        start_index: u64,
        end_index: u64,
        rollup_hash: ContentHash,
        summary: RollupSummary,
    },
    
    // === Attestations ===
    /// Third-party attestation about an entry
    Attestation {
        subject_entry: EntryHash,
        attestation_type: AttestationType,
        claim: Claim,
    },
    
    /// Revocation of previous entry
    Revocation {
        revoked_entry: EntryHash,
        reason: RevocationReason,
    },
    
    // === Slice Operations (Waypoint Support) ===
    /// Slice arrives at this spine (waypoint anchor)
    SliceAnchor {
        /// Unique slice identifier
        slice_id: SliceId,
        /// Origin spine and entry
        origin: SliceOrigin,
        /// Slice mode
        mode: SliceMode,
        /// Terms of the slice
        terms: SliceTerms,
    },
    
    /// Operation performed on an anchored slice
    SliceOperation {
        /// The slice being operated on
        slice_id: SliceId,
        /// Operation type
        operation: SliceOperationType,
        /// Operation payload hash
        payload: PayloadRef,
        /// RhizoCrypt session (if applicable)
        session: Option<SessionId>,
    },
    
    /// Slice departs this spine
    SliceDeparture {
        /// The departing slice
        slice_id: SliceId,
        /// Reason for departure
        reason: DepartureReason,
        /// Summary of operations performed while here
        operation_summary: OperationSummary,
        /// Where the slice is going
        destination: SliceDestination,
    },
    
    /// Slice returns from waypoint to origin
    SliceReturn {
        /// The returning slice
        slice_id: SliceId,
        /// Original loan/consignment entry
        original_entry: EntryHash,
        /// Waypoint where slice was anchored
        waypoint_spine: SpineId,
        /// Summary of waypoint operations (if allowed to propagate)
        waypoint_summary: Option<WaypointSummary>,
    },
    
    /// External anchor (optional cross-chain/cross-commons reference)
    ExternalAnchor {
        /// The entry being anchored
        entry: EntryHash,
        /// External anchor reference
        anchor: ExternalAnchorRef,
        /// Proof of anchor
        proof: AnchorProof,
    },
    
    // === Custom ===
    Custom {
        type_uri: String,
        payload: Bytes,
    },
}

/// Origin information for a slice
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SliceOrigin {
    /// Source spine
    pub spine_id: SpineId,
    /// Source entry
    pub entry: EntryHash,
    /// Certificate (if slice is of a certificate)
    pub certificate: Option<CertificateId>,
    /// Owner DID
    pub owner: Did,
}

/// Slice operation types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SliceOperationType {
    /// Read/view the slice
    View,
    /// Use the slice (e.g., play a game)
    Use { context: String },
    /// Modify metadata
    ModifyMetadata { field: String },
    /// Sublend to another party
    Sublend { to: Did, terms: SliceTerms },
    /// Custom operation
    Custom { operation: String },
}

/// Where a slice goes when it departs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SliceDestination {
    /// Return to origin spine
    ReturnToOrigin,
    /// Transfer to new owner's spine
    TransferToOwner { new_owner: Did, new_spine: SpineId },
    /// Move to another waypoint
    ToWaypoint { waypoint: SpineId },
    /// Consumed (e.g., one-time use license)
    Consumed,
}

/// Summary of operations at a waypoint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaypointSummary {
    /// Duration of stay
    pub duration: Duration,
    /// Number of operations
    pub operation_count: u64,
    /// Operation types performed
    pub operation_types: Vec<SliceOperationType>,
    /// Hash of full operation log (for verification)
    pub operations_hash: ContentHash,
}
```

### 2.3 Spine Structure

```rust
/// A LoamSpine (the complete ledger)
#[derive(Clone, Debug)]
pub struct LoamSpine {
    /// Unique spine identifier
    pub spine_id: SpineId,
    
    /// Spine owner (DID)
    pub owner: Did,
    
    /// Spine configuration
    pub config: SpineConfig,
    
    /// Genesis entry hash
    pub genesis: EntryHash,
    
    /// Latest entry hash (tip of the spine)
    pub tip: EntryHash,
    
    /// Current entry count
    pub height: u64,
    
    /// Spine metadata
    pub metadata: SpineMetadata,
    
    /// State (active, sealed, archived)
    pub state: SpineState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpineConfig {
    /// Spine type (personal, community, public)
    pub spine_type: SpineType,
    
    /// Replication policy
    pub replication: ReplicationPolicy,
    
    /// Access control policy
    pub access: AccessPolicy,
    
    /// Required attestations for certain entry types
    pub attestation_requirements: AttestationRequirements,
    
    /// Maximum entries before mandatory rollup
    pub max_entries_before_rollup: Option<u64>,
    
    /// Retention policy
    pub retention: RetentionPolicy,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SpineState {
    /// Actively accepting entries
    Active,
    
    /// Temporarily frozen (e.g., dispute resolution)
    Frozen { reason: String, until: Option<Timestamp> },
    
    /// Permanently sealed (no new entries)
    Sealed { final_entry: EntryHash },
    
    /// Archived (read-only, may be moved to cold storage)
    Archived,
}
```

### 2.4 Certificate Model (Loam Certificates)

The Loam Certificate Layer provides "memory-bound objects":

```rust
/// A Loam Certificate (memory-bound object)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoamCertificate {
    /// Unique certificate ID
    pub cert_id: CertificateId,
    
    /// Certificate type
    pub cert_type: CertificateType,
    
    /// Current owner
    pub owner: Did,
    
    /// Mint entry (proof of origin)
    pub mint_entry: EntryHash,
    
    /// Current entry (latest state)
    pub current_entry: EntryHash,
    
    /// Full ownership history
    pub history: Vec<OwnershipRecord>,
    
    /// Associated metadata
    pub metadata: CertificateMetadata,
    
    /// Linked data (e.g., RhizoCrypt sessions, external references)
    pub links: Vec<CertificateLink>,
    
    /// Current loan status
    pub loan_status: Option<LoanStatus>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CertificateType {
    // Digital ownership
    DigitalGameKey { platform: String, game_id: String },
    DigitalCollectible { collection: String, item_id: String },
    DigitalLicense { software: String, license_type: String },
    
    // Physical ownership
    VehicleTitle { vin: String },
    PropertyDeed { parcel_id: String },
    
    // Credentials
    AcademicDegree { institution: String, degree: String },
    ProfessionalLicense { authority: String, license_type: String },
    Certification { issuer: String, cert_name: String },
    
    // Provenance
    ArtworkProvenance { artist: String, title: String },
    BiologicalSample { sample_type: String, origin: String },
    
    // Custom
    Custom { type_uri: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoanTerms {
    /// Loan duration
    pub duration: Option<Duration>,
    
    /// Use restrictions
    pub restrictions: Vec<UseRestriction>,
    
    /// Return conditions
    pub return_conditions: ReturnConditions,
    
    /// Automatic return on expiry
    pub auto_return: bool,
}
```

---

## 3. Architecture

### 3.1 Component Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      LoamSpine Service                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │    Spine    │  │    Entry    │  │      Certificate        │ │
│  │   Manager   │  │   Writer    │  │        Manager          │ │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘ │
│         │                │                      │               │
│         ▼                ▼                      ▼               │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                    Entry Store                             │ │
│  │  (Append-only, indexed by hash and index)                  │ │
│  └───────────────────────────────────────────────────────────┘ │
│                            │                                    │
│                            ▼                                    │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                 Replication Engine                         │ │
│  │      (Sync with peers, federated verification)             │ │
│  └───────────────────────────────────────────────────────────┘ │
│                            │                                    │
└────────────────────────────┼────────────────────────────────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
        ▼                    ▼                    ▼
   ┌─────────┐         ┌──────────┐        ┌───────────┐
   │ BearDog │         │RhizoCrypt│        │ SweetGrass│
   │   🐻    │         │   🔐     │        │    🌾     │
   │ Signing │         │ Sessions │        │  Braids   │
   └─────────┘         └──────────┘        └───────────┘
```

### 3.2 Spine Manager

```rust
/// Spine Manager API
pub trait SpineManager {
    /// Create a new spine
    async fn create_spine(
        &self,
        owner: Did,
        config: SpineConfig,
    ) -> Result<LoamSpine, LoamError>;
    
    /// Get a spine by ID
    async fn get_spine(&self, id: SpineId) -> Result<Option<LoamSpine>, LoamError>;
    
    /// List spines (with filters)
    async fn list_spines(&self, filter: SpineFilter) -> Result<Vec<SpineSummary>, LoamError>;
    
    /// Seal a spine (no more entries)
    async fn seal_spine(&self, id: SpineId) -> Result<EntryHash, LoamError>;
    
    /// Archive a spine
    async fn archive_spine(&self, id: SpineId) -> Result<(), LoamError>;
}
```

### 3.3 Entry Writer

```rust
/// Entry Writer API
pub trait EntryWriter {
    /// Append an entry to a spine
    async fn append(
        &self,
        spine: SpineId,
        entry_type: EntryType,
        payload: EntryPayload,
        signer: &impl Signer,
    ) -> Result<LoamEntry, LoamError>;
    
    /// Append with additional attestations
    async fn append_with_attestations(
        &self,
        spine: SpineId,
        entry_type: EntryType,
        payload: EntryPayload,
        signer: &impl Signer,
        attestations: Vec<Attestation>,
    ) -> Result<LoamEntry, LoamError>;
    
    /// Request attestation from another party
    async fn request_attestation(
        &self,
        entry: EntryHash,
        attester: Did,
    ) -> Result<AttestationRequest, LoamError>;
}
```

### 3.4 Certificate Manager

```rust
/// Certificate Manager API
pub trait CertificateManager {
    /// Mint a new certificate
    async fn mint(
        &self,
        spine: SpineId,
        cert_type: CertificateType,
        initial_owner: Did,
        metadata: CertificateMetadata,
        signer: &impl Signer,
    ) -> Result<LoamCertificate, LoamError>;
    
    /// Transfer certificate ownership
    async fn transfer(
        &self,
        cert_id: CertificateId,
        to: Did,
        conditions: Option<TransferConditions>,
        signer: &impl Signer,
    ) -> Result<LoamEntry, LoamError>;
    
    /// Loan a certificate
    async fn loan(
        &self,
        cert_id: CertificateId,
        borrower: Did,
        terms: LoanTerms,
        signer: &impl Signer,
    ) -> Result<LoamEntry, LoamError>;
    
    /// Return a loaned certificate
    async fn return_loan(
        &self,
        cert_id: CertificateId,
        signer: &impl Signer,
    ) -> Result<LoamEntry, LoamError>;
    
    /// Get certificate by ID
    async fn get_certificate(&self, id: CertificateId) -> Result<Option<LoamCertificate>, LoamError>;
    
    /// Get certificate history
    async fn get_history(&self, id: CertificateId) -> Result<Vec<OwnershipRecord>, LoamError>;
    
    /// Verify certificate authenticity
    async fn verify(&self, cert: &LoamCertificate) -> Result<VerificationResult, LoamError>;
}
```

---

## 4. Storage Model

### 4.1 Entry Store

LoamSpine entries are stored in an append-only, durable store:

```rust
/// Entry Store trait
pub trait EntryStore: Send + Sync {
    /// Append an entry (must be at tip)
    async fn append(&self, spine: SpineId, entry: LoamEntry) -> Result<(), StoreError>;
    
    /// Get entry by hash
    async fn get_by_hash(&self, hash: EntryHash) -> Result<Option<LoamEntry>, StoreError>;
    
    /// Get entry by spine and index
    async fn get_by_index(&self, spine: SpineId, index: u64) -> Result<Option<LoamEntry>, StoreError>;
    
    /// Get entries in range
    async fn get_range(
        &self,
        spine: SpineId,
        start: u64,
        end: u64,
    ) -> Result<Vec<LoamEntry>, StoreError>;
    
    /// Get the tip entry
    async fn get_tip(&self, spine: SpineId) -> Result<Option<LoamEntry>, StoreError>;
    
    /// Iterate all entries in a spine
    fn iter_spine(&self, spine: SpineId) -> impl Stream<Item = LoamEntry>;
    
    /// Verify chain integrity
    async fn verify_chain(&self, spine: SpineId) -> Result<ChainVerification, StoreError>;
}
```

**Recommended backends:**
- **SQLite** — For personal spines, portable
- **PostgreSQL** — For community/shared spines
- **RocksDB** — For high-performance local storage
- **S3-compatible** — For archived spines (cold storage)

### 4.2 Index Store

Secondary indexes for efficient querying:

```rust
/// Index Store trait
pub trait IndexStore: Send + Sync {
    /// Index entry by certificate
    async fn index_certificate(&self, cert_id: CertificateId, entry: EntryHash) -> Result<(), StoreError>;
    
    /// Get all entries for a certificate
    async fn get_certificate_entries(&self, cert_id: CertificateId) -> Result<Vec<EntryHash>, StoreError>;
    
    /// Index entry by committer
    async fn index_committer(&self, committer: Did, entry: EntryHash) -> Result<(), StoreError>;
    
    /// Get all entries by committer
    async fn get_committer_entries(&self, committer: Did) -> Result<Vec<EntryHash>, StoreError>;
    
    /// Full-text search in metadata
    async fn search(&self, query: SearchQuery) -> Result<Vec<EntryHash>, StoreError>;
}
```

---

## 5. Replication & Federation

### 5.1 Replication Model

LoamSpine supports multiple replication strategies:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ReplicationPolicy {
    /// No replication (single node)
    None,
    
    /// Replicate to specific peers
    Peers { peers: Vec<PeerId>, min_copies: usize },
    
    /// Replicate to any N nodes in federation
    Federation { min_copies: usize, prefer_geographic_distribution: bool },
    
    /// Full replication to all federation members
    Full,
    
    /// Archive to cold storage after N days
    ArchiveAfter { days: u32, storage: ArchiveStorage },
}
```

### 5.2 Sync Protocol

```rust
/// Sync Protocol for spine replication
pub trait SyncProtocol {
    /// Get spine summary (for comparison)
    async fn get_summary(&self, spine: SpineId) -> Result<SpineSummary, SyncError>;
    
    /// Get entries since index
    async fn get_entries_since(
        &self,
        spine: SpineId,
        since_index: u64,
        limit: usize,
    ) -> Result<Vec<LoamEntry>, SyncError>;
    
    /// Push entries to peer
    async fn push_entries(
        &self,
        peer: PeerId,
        spine: SpineId,
        entries: Vec<LoamEntry>,
    ) -> Result<SyncReceipt, SyncError>;
    
    /// Request missing entries from peer
    async fn request_entries(
        &self,
        peer: PeerId,
        spine: SpineId,
        hashes: Vec<EntryHash>,
    ) -> Result<Vec<LoamEntry>, SyncError>;
}

#[derive(Clone, Debug)]
pub struct SpineSummary {
    pub spine_id: SpineId,
    pub height: u64,
    pub tip: EntryHash,
    pub genesis: EntryHash,
    pub last_sync: Timestamp,
}
```

### 5.3 Conflict Resolution

LoamSpine is append-only, so conflicts are structural impossibilities in a well-behaved system. However, forks can occur:

```rust
#[derive(Clone, Debug)]
pub enum ForkResolution {
    /// Accept the longer chain
    LongestChain,
    
    /// Accept the chain with more attestations
    MostAttested,
    
    /// Accept the chain from the spine owner
    OwnerAuthority,
    
    /// Manual resolution required
    ManualReview,
}
```

---

## 6. Verification & Proofs

### 6.1 Chain Verification

```rust
/// Chain verification result
#[derive(Clone, Debug)]
pub struct ChainVerification {
    pub spine_id: SpineId,
    pub verified_entries: u64,
    pub valid: bool,
    pub errors: Vec<VerificationError>,
}

#[derive(Clone, Debug)]
pub enum VerificationError {
    /// Previous hash mismatch
    HashMismatch { index: u64, expected: EntryHash, actual: EntryHash },
    
    /// Invalid signature
    InvalidSignature { index: u64, signer: Did },
    
    /// Missing required attestation
    MissingAttestation { index: u64, required: AttestationType },
    
    /// Invalid entry type for spine
    InvalidEntryType { index: u64, entry_type: EntryType },
    
    /// Timestamp regression
    TimestampRegression { index: u64, previous: Timestamp, current: Timestamp },
}
```

### 6.2 Inclusion Proofs

Prove an entry exists in a spine without revealing the full chain:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InclusionProof {
    /// The entry being proven
    pub entry: LoamEntry,
    
    /// Path from entry to tip
    pub path: Vec<EntryHash>,
    
    /// Current tip
    pub tip: EntryHash,
    
    /// Spine ID
    pub spine_id: SpineId,
    
    /// Proof timestamp
    pub timestamp: Timestamp,
    
    /// Optional: signature from spine owner
    pub owner_attestation: Option<Signature>,
}

impl InclusionProof {
    /// Verify this proof
    pub fn verify(&self) -> bool {
        // Verify hash chain from entry to tip
        let mut current = self.entry.hash;
        for next in &self.path {
            // Each entry in path must reference current as previous
            // (simplified - actual implementation verifies full entry)
            current = *next;
        }
        current == self.tip
    }
}
```

### 6.3 Certificate Proofs

Prove certificate ownership and history:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificateProof {
    /// Certificate ID
    pub cert_id: CertificateId,
    
    /// Current owner
    pub owner: Did,
    
    /// Mint entry with inclusion proof
    pub mint_proof: InclusionProof,
    
    /// Transfer chain (if any)
    pub transfer_proofs: Vec<InclusionProof>,
    
    /// Current state entry
    pub current_proof: InclusionProof,
    
    /// Proof timestamp
    pub timestamp: Timestamp,
}
```

---

## 7. Integration Points

### 7.1 BearDog Integration

```rust
/// BearDog client for LoamSpine
pub trait BearDogClient {
    /// Sign an entry
    async fn sign_entry(&self, entry: &LoamEntry, key_id: KeyId) -> Result<Signature, BearDogError>;
    
    /// Verify entry signature
    async fn verify_entry(&self, entry: &LoamEntry) -> Result<bool, BearDogError>;
    
    /// Resolve DID to verify ownership
    async fn resolve_did(&self, did: &Did) -> Result<DidDocument, BearDogError>;
    
    /// Create attestation
    async fn create_attestation(
        &self,
        subject: EntryHash,
        claim: Claim,
        key_id: KeyId,
    ) -> Result<Attestation, BearDogError>;
}
```

### 7.2 RhizoCrypt Integration

```rust
/// RhizoCrypt client for LoamSpine
pub trait RhizoCryptClient {
    /// Commit a dehydrated session
    async fn commit_session(
        &self,
        spine: SpineId,
        summary: DehydrationSummary,
    ) -> Result<EntryHash, RhizoError>;
    
    /// Verify session Merkle root
    async fn verify_session(&self, merkle_root: MerkleRoot) -> Result<bool, RhizoError>;
    
    /// Get Merkle proof for item in session
    async fn get_item_proof(
        &self,
        session_id: SessionId,
        item_id: ItemId,
    ) -> Result<MerkleProof, RhizoError>;
}
```

### 7.3 SweetGrass Integration

```rust
/// SweetGrass client for LoamSpine
pub trait SweetGrassClient {
    /// Commit a Braid
    async fn commit_braid(
        &self,
        spine: SpineId,
        braid: Braid,
    ) -> Result<EntryHash, SweetGrassError>;
    
    /// Query Braids for a data hash
    async fn get_braids_for_data(&self, data_hash: ContentHash) -> Result<Vec<BraidId>, SweetGrassError>;
}
```

---

## 8. API Specification

### 8.1 Pure Rust RPC Philosophy

LoamSpine uses **pure Rust RPC**—no gRPC, no protobuf, no C++ tooling.

| ❌ What We Don't Use | ✅ What We Use |
|---------------------|----------------|
| gRPC | tarpc (pure Rust) |
| protobuf/proto files | serde (native Rust) |
| protoc (C++ compiler) | cargo build only |
| tonic | pure Rust JSON-RPC 2.0 |

This aligns with the **Primal Sovereignty** principle: no external tooling, no vendor lock-in, no C++ dependencies.

See [PURE_RUST_RPC.md](./PURE_RUST_RPC.md) for the full philosophy.

### 8.2 tarpc Service Trait

```rust
#[tarpc::service]
pub trait LoamSpineRpc {
    // Spine management
    async fn create_spine(request: CreateSpineRequest) -> Result<CreateSpineResponse, ApiError>;
    async fn get_spine(request: GetSpineRequest) -> Result<GetSpineResponse, ApiError>;
    async fn seal_spine(request: SealSpineRequest) -> Result<SealSpineResponse, ApiError>;
    
    // Entry operations
    async fn append_entry(request: AppendEntryRequest) -> Result<AppendEntryResponse, ApiError>;
    async fn get_entry(request: GetEntryRequest) -> Result<GetEntryResponse, ApiError>;
    async fn get_tip(request: GetTipRequest) -> Result<GetTipResponse, ApiError>;
    
    // Certificate operations
    async fn mint_certificate(request: MintCertificateRequest) -> Result<MintCertificateResponse, ApiError>;
    async fn transfer_certificate(request: TransferCertificateRequest) -> Result<TransferCertificateResponse, ApiError>;
    async fn loan_certificate(request: LoanCertificateRequest) -> Result<LoanCertificateResponse, ApiError>;
    async fn return_certificate(request: ReturnCertificateRequest) -> Result<ReturnCertificateResponse, ApiError>;
    async fn get_certificate(request: GetCertificateRequest) -> Result<GetCertificateResponse, ApiError>;
    
    // Waypoint operations
    async fn anchor_slice(request: AnchorSliceRequest) -> Result<AnchorSliceResponse, ApiError>;
    async fn checkout_slice(request: CheckoutSliceRequest) -> Result<CheckoutSliceResponse, ApiError>;
    
    // Integration
    async fn commit_session(request: CommitSessionRequest) -> Result<CommitSessionResponse, ApiError>;
    async fn commit_braid(request: CommitBraidRequest) -> Result<CommitBraidResponse, ApiError>;
    
    // Health
    async fn health_check(request: HealthCheckRequest) -> Result<HealthCheckResponse, ApiError>;
}
```

### 8.3 JSON-RPC 2.0 Endpoint

External clients use JSON-RPC 2.0 for universal access:

```bash
# Health check
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "loamspine.healthCheck", "params": {}, "id": 1}'

# Create spine
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "loamspine.createSpine", "params": {"name": "my-spine", "owner": {"value": "did:key:z6Mk..."}}, "id": 2}'
```

See [API_SPECIFICATION.md](./API_SPECIFICATION.md) for the full API reference.

---

## 9. Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Entry append latency | < 10ms | p99, with signature |
| Entry read latency | < 1ms | By hash |
| Chain verification | < 1s | 100k entries |
| Inclusion proof generation | < 10ms | Single entry |
| Certificate lookup | < 5ms | By ID |
| Sync throughput | > 1000 entries/sec | Between peers |
| Storage efficiency | < 1KB | Per entry (excluding payload) |

---

## 10. Security Considerations

### 10.1 Immutability Guarantees

- Entries cannot be modified after commit
- Hash chain ensures tamper detection
- Signatures provide non-repudiation
- Replication prevents single-point deletion

### 10.2 Access Control

- Spine owners control append access
- Read access configurable per spine
- Certificates have ownership-based access
- BearDog policies enforce all access

### 10.3 Privacy

- Spine contents can be encrypted
- Certificate metadata supports selective disclosure
- Proofs reveal minimal information
- Zero-knowledge proofs for sensitive claims (future)

### 10.4 Availability

- Replication ensures durability
- Federation prevents central control
- Archive policy preserves historical data
- Cold storage for long-term retention

---

## 11. Implementation Roadmap

### Phase 1: Core Engine (4 weeks)
- [ ] Entry and Spine data structures
- [ ] Append-only entry store (SQLite)
- [ ] Basic chain verification
- [ ] BearDog signing integration

### Phase 2: Certificates (3 weeks)
- [ ] Certificate mint/transfer/loan
- [ ] Certificate history tracking
- [ ] Ownership verification

### Phase 3: Proofs (2 weeks)
- [ ] Inclusion proof generation
- [ ] Certificate proof generation
- [ ] Proof verification

### Phase 4: Replication (3 weeks)
- [ ] Sync protocol implementation
- [ ] Peer discovery (via Songbird)
- [ ] Conflict detection

### Phase 5: Integration (2 weeks)
- [ ] RhizoCrypt session commits
- [ ] SweetGrass Braid commits
- [ ] Songbird UPA registration

### Phase 6: Performance & Hardening (2 weeks)
- [ ] Benchmarking and optimization
- [ ] PostgreSQL backend
- [ ] Security audit

---

## 12. References

### ecoPrimals Specifications
- [PURE_RUST_RPC.md](./PURE_RUST_RPC.md) — Pure Rust RPC philosophy (no gRPC)
- [API_SPECIFICATION.md](./API_SPECIFICATION.md) — tarpc + JSON-RPC 2.0 API
- [RhizoCrypt Specification](../../rhizoCrypt/specs/) — Ephemeral DAG
- [SweetGrass Specification](../../sweetGrass/specs/) — Semantic attribution
- [BearDog Specification](../../../phase1/bearDog/specs/) — Identity and signing

### External Resources
- [Merkle Trees](https://en.wikipedia.org/wiki/Merkle_tree) — Cryptographic verification
- [Certificate Transparency](https://certificate.transparency.dev/) — Append-only log inspiration
- [Git Object Model](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects) — Content-addressed storage
- [tarpc Documentation](https://docs.rs/tarpc/) — Pure Rust RPC framework
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification) — Hand-rolled pure Rust implementation (no jsonrpsee dependency)

---

## Appendix A: Example Certificate Lifecycle (Game Key)

```
1. Game publisher mints certificate
   → MintCertificate(type: DigitalGameKey {
       platform: "steam",
       game_id: "half-life-3"
     }, owner: "did:key:publisher")
   → CertificateId: "cert-hl3-001"

2. Publisher sells to retailer
   → TransferCertificate(
       cert_id: "cert-hl3-001",
       to: "did:key:gamestop"
     )

3. Retailer sells to customer
   → TransferCertificate(
       cert_id: "cert-hl3-001",
       to: "did:key:player1"
     )

4. Customer loans to friend for weekend
   → LoanCertificate(
       cert_id: "cert-hl3-001",
       borrower: "did:key:friend",
       terms: { duration: 48h, auto_return: true }
     )

5. Friend plays game (Steam integration)
   → RhizoCrypt captures play session
   → Session committed to LoamSpine
   → Certificate now has play history

6. Loan expires, certificate returns
   → ReturnCertificate (automatic)
   → Owner is player1 again
   → Friend can no longer play

7. Years later: verification
   → GetCertificateHistory("cert-hl3-001")
   → Full provenance from publisher to current owner
   → All play sessions anchored
   → Certificate value includes history
```

---

*LoamSpine: The permanent record that gives memory its meaning.*

