# LoamSpine — Data Model Specification

**Version**: 0.2.0  
**Status**: Draft  
**Last Updated**: December 22, 2025

---

## 1. Overview

This document defines the core data structures of LoamSpine: the Entry, Spine, and Chain types that form the foundation of the permanent ledger.

---

## 2. Content Addressing

All LoamSpine data structures use Blake3 for content addressing:

```rust
use blake3::Hasher;

/// 32-byte content hash
pub type ContentHash = [u8; 32];

/// Entry hash (Blake3 of canonical entry)
pub type EntryHash = ContentHash;

/// Spine identifier (UUID v7 for time-ordering)
pub type SpineId = uuid::Uuid;

/// Certificate identifier (UUID v7)
pub type CertificateId = uuid::Uuid;

/// Compute Blake3 hash of bytes
pub fn hash_bytes(data: &[u8]) -> ContentHash {
    blake3::hash(data).into()
}
```

---

## 3. Entry Structure

An Entry is a single, immutable record in a LoamSpine.

### 3.1 Core Definition

```rust
use serde::{Deserialize, Serialize};

/// A single entry in a LoamSpine
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    // === Identity ===
    
    /// Entry hash (computed, not serialized in canonical form)
    #[serde(skip)]
    hash: Option<EntryHash>,
    
    // === Chain Links ===
    
    /// Sequential index within this spine (0 for genesis)
    pub index: u64,
    
    /// Hash of the previous entry (None for genesis)
    pub previous: Option<EntryHash>,
    
    // === Metadata ===
    
    /// Timestamp of commitment (nanoseconds since epoch)
    pub timestamp: u64,
    
    /// The agent committing this entry (BearDog DID)
    pub committer: Did,
    
    // === Content ===
    
    /// Entry type
    pub entry_type: EntryType,
    
    /// Optional payload reference (content-addressed)
    pub payload: Option<PayloadRef>,
    
    /// Inline metadata
    pub metadata: HashMap<String, Value>,
    
    // === Cryptographic ===
    
    /// Cryptographic signature from committer
    pub signature: Signature,
    
    /// Additional attestations (witnesses, validators)
    pub attestations: Vec<Attestation>,
}

impl Entry {
    /// Compute the entry hash (Blake3 of canonical form)
    pub fn compute_hash(&self) -> EntryHash {
        let canonical = self.to_canonical_bytes();
        hash_bytes(&canonical)
    }
    
    /// Get or compute the entry hash
    pub fn hash(&mut self) -> EntryHash {
        if let Some(hash) = self.hash {
            hash
        } else {
            let hash = self.compute_hash();
            self.hash = Some(hash);
            hash
        }
    }
    
    /// Serialize to canonical bytes (for hashing)
    /// Excludes the hash field itself
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let canonical = CanonicalEntry {
            index: self.index,
            previous: self.previous,
            timestamp: self.timestamp,
            committer: self.committer.clone(),
            entry_type: self.entry_type.clone(),
            payload: self.payload.clone(),
            metadata: self.metadata.clone(),
            signature: self.signature.clone(),
            attestations: self.attestations.clone(),
        };
        
        let mut buf = Vec::new();
        ciborium::into_writer(&canonical, &mut buf)
            .expect("Entry serialization cannot fail");
        buf
    }
    
    /// Check if this is a genesis entry
    pub fn is_genesis(&self) -> bool {
        self.index == 0 && self.previous.is_none()
    }
    
    /// Verify the entry signature
    pub async fn verify_signature(&self, beardog: &impl BearDogClient) -> Result<bool, LoamError> {
        let data_to_sign = self.to_signable_bytes();
        beardog.verify_signature(&data_to_sign, &self.signature, &self.committer).await
    }
    
    /// Get bytes that are signed (excludes signature and attestations)
    fn to_signable_bytes(&self) -> Vec<u8> {
        let signable = SignableEntry {
            index: self.index,
            previous: self.previous,
            timestamp: self.timestamp,
            committer: self.committer.clone(),
            entry_type: self.entry_type.clone(),
            payload: self.payload.clone(),
            metadata: self.metadata.clone(),
        };
        
        let mut buf = Vec::new();
        ciborium::into_writer(&signable, &mut buf)
            .expect("Signable serialization cannot fail");
        buf
    }
}
```

### 3.2 Entry Types

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
    
    /// Spine sealed (no more entries)
    SpineSealed {
        reason: Option<String>,
    },
    
    // === RhizoCrypt Integration ===
    
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
        mime_type: Option<String>,
        size: u64,
    },
    
    /// SweetGrass Braid commitment
    BraidCommit {
        braid_id: BraidId,
        braid_hash: ContentHash,
        subject_hash: ContentHash,
    },
    
    // === Certificate Operations ===
    
    /// Mint a new certificate
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
    
    /// Loan certificate (temporary transfer)
    CertificateLoan {
        cert_id: CertificateId,
        lender: Did,
        borrower: Did,
        terms: LoanTerms,
    },
    
    /// Return loaned certificate
    CertificateReturn {
        cert_id: CertificateId,
        loan_entry: EntryHash,
        usage_summary: Option<UsageSummary>,
    },
    
    // === Slice Operations ===
    
    /// Slice checked out (borrowed state leaves)
    SliceCheckout {
        slice_id: SliceId,
        session_id: SessionId,
        mode: SliceMode,
        terms: Option<SliceTerms>,
    },
    
    /// Slice anchored at this spine (waypoint)
    SliceAnchor {
        slice_id: SliceId,
        origin_spine: SpineId,
        origin_entry: EntryHash,
        terms: SliceTerms,
    },
    
    /// Slice operation at waypoint
    SliceOperation {
        slice_id: SliceId,
        operation: SliceOperationType,
        payload: Option<PayloadRef>,
    },
    
    /// Slice departing waypoint
    SliceDeparture {
        slice_id: SliceId,
        reason: DepartureReason,
        summary: WaypointSummary,
    },
    
    /// Slice returned to origin
    SliceReturn {
        slice_id: SliceId,
        checkout_entry: EntryHash,
        waypoint_spine: Option<SpineId>,
        waypoint_summary: Option<WaypointSummary>,
    },
    
    // === Recursive Stacking ===
    
    /// Reference to another spine's entry
    SpineReference {
        referenced_spine: SpineId,
        referenced_entry: EntryHash,
        reference_type: ReferenceType,
    },
    
    /// Rollup of multiple entries
    Rollup {
        start_index: u64,
        end_index: u64,
        rollup_hash: ContentHash,
        summary: RollupSummary,
    },
    
    // === External Anchors ===
    
    /// Anchor to external system
    ExternalAnchor {
        entry: EntryHash,
        anchor_type: ExternalAnchorType,
        anchor_ref: String,
        proof: Option<AnchorProof>,
    },
    
    // === Attestations ===
    
    /// Third-party attestation
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
    
    // === Custom ===
    
    Custom {
        type_uri: String,
        payload: Vec<u8>,
    },
}

impl EntryType {
    /// Get the domain for this entry type
    pub fn domain(&self) -> &'static str {
        match self {
            Self::Genesis { .. } | Self::MetadataUpdate { .. } | Self::SpineSealed { .. } => "spine",
            Self::SessionCommit { .. } => "rhizocrypt",
            Self::DataAnchor { .. } | Self::BraidCommit { .. } => "data",
            Self::CertificateMint { .. } | Self::CertificateTransfer { .. } | 
            Self::CertificateLoan { .. } | Self::CertificateReturn { .. } => "certificate",
            Self::SliceCheckout { .. } | Self::SliceAnchor { .. } | 
            Self::SliceOperation { .. } | Self::SliceDeparture { .. } | 
            Self::SliceReturn { .. } => "slice",
            Self::SpineReference { .. } | Self::Rollup { .. } => "stacking",
            Self::ExternalAnchor { .. } => "external",
            Self::Attestation { .. } | Self::Revocation { .. } => "attestation",
            Self::Custom { .. } => "custom",
        }
    }
    
    /// Check if this entry type is allowed in a waypoint spine
    pub fn allowed_in_waypoint(&self) -> bool {
        matches!(
            self,
            Self::Genesis { .. } |
            Self::SliceAnchor { .. } |
            Self::SliceOperation { .. } |
            Self::SliceDeparture { .. }
        )
    }
}
```

### 3.3 Entry Builder

```rust
/// Builder for creating entries
pub struct EntryBuilder {
    entry_type: EntryType,
    payload: Option<PayloadRef>,
    metadata: HashMap<String, Value>,
    attestations: Vec<Attestation>,
}

impl EntryBuilder {
    /// Create a new builder with required entry type
    pub fn new(entry_type: EntryType) -> Self {
        Self {
            entry_type,
            payload: None,
            metadata: HashMap::new(),
            attestations: Vec::new(),
        }
    }
    
    /// Set the payload reference
    pub fn with_payload(mut self, payload: PayloadRef) -> Self {
        self.payload = Some(payload);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Add an attestation
    pub fn with_attestation(mut self, attestation: Attestation) -> Self {
        self.attestations.push(attestation);
        self
    }
    
    /// Build the entry (requires spine context for index/previous/committer)
    pub async fn build(
        self,
        spine: &Spine,
        committer: Did,
        signer: &impl Signer,
    ) -> Result<Entry, LoamError> {
        let previous = spine.tip();
        let index = spine.height();
        let timestamp = current_timestamp_nanos();
        
        // Create entry without signature
        let mut entry = Entry {
            hash: None,
            index,
            previous,
            timestamp,
            committer: committer.clone(),
            entry_type: self.entry_type,
            payload: self.payload,
            metadata: self.metadata,
            signature: Signature::default(),
            attestations: self.attestations,
        };
        
        // Sign entry
        let signable = entry.to_signable_bytes();
        entry.signature = signer.sign(&signable).await?;
        
        Ok(entry)
    }
}
```

---

## 4. Spine Structure

A Spine is a linear chain of entries with common ownership.

### 4.1 Core Definition

```rust
/// A LoamSpine (linear chain of entries)
#[derive(Clone, Debug)]
pub struct Spine {
    // === Identity ===
    
    /// Unique spine identifier
    pub id: SpineId,
    
    /// Human-readable spine name
    pub name: Option<String>,
    
    // === Ownership ===
    
    /// Spine owner (can transfer ownership)
    pub owner: Did,
    
    // === Configuration ===
    
    /// Spine type
    pub spine_type: SpineType,
    
    /// Spine configuration
    pub config: SpineConfig,
    
    // === Chain State ===
    
    /// Genesis entry hash
    pub genesis: EntryHash,
    
    /// Current tip (latest entry hash)
    pub tip: EntryHash,
    
    /// Current height (number of entries)
    pub height: u64,
    
    // === Metadata ===
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last update timestamp
    pub updated_at: u64,
    
    /// Spine state
    pub state: SpineState,
    
    /// Custom metadata
    pub metadata: HashMap<String, Value>,
}

/// Spine type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpineType {
    /// Personal history
    Personal,
    
    /// Professional/work spine
    Professional,
    
    /// Community shared spine
    Community {
        community_id: String,
    },
    
    /// Waypoint for borrowed state
    Waypoint {
        max_anchor_depth: Option<u32>,
    },
    
    /// Public, globally verifiable
    Public,
    
    /// Custom type
    Custom {
        type_name: String,
    },
}

/// Spine state
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpineState {
    /// Actively accepting entries
    Active,
    
    /// Temporarily frozen
    Frozen {
        reason: String,
        until: Option<u64>,
    },
    
    /// Permanently sealed (read-only)
    Sealed {
        sealed_at: u64,
        final_entry: EntryHash,
    },
    
    /// Archived (cold storage)
    Archived {
        archived_at: u64,
        archive_location: String,
    },
}

impl SpineState {
    /// Check if spine is accepting entries
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
    
    /// Check if spine is terminal (sealed or archived)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Sealed { .. } | Self::Archived { .. })
    }
}
```

### 4.2 Spine Configuration

```rust
/// Spine configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpineConfig {
    // === Replication ===
    
    /// Replication policy
    pub replication: ReplicationPolicy,
    
    // === Access Control ===
    
    /// Who can read this spine
    pub read_access: AccessPolicy,
    
    /// Who can write to this spine
    pub write_access: AccessPolicy,
    
    /// Who can administer this spine
    pub admin_access: AccessPolicy,
    
    // === Attestations ===
    
    /// Required attestations for certain entry types
    pub attestation_requirements: AttestationRequirements,
    
    // === Rollups ===
    
    /// Automatic rollup after N entries
    pub auto_rollup_threshold: Option<u64>,
    
    /// Rollup compression level
    pub rollup_compression: CompressionLevel,
    
    // === Retention ===
    
    /// Retention policy
    pub retention: RetentionPolicy,
    
    // === Waypoint (if waypoint spine) ===
    
    /// Waypoint-specific configuration
    pub waypoint: Option<WaypointConfig>,
}

/// Replication policy
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ReplicationPolicy {
    /// No replication
    None,
    
    /// Replicate to specific peers
    Peers {
        peers: Vec<PeerId>,
        min_copies: usize,
    },
    
    /// Replicate within federation
    Federation {
        min_copies: usize,
        prefer_geographic_distribution: bool,
    },
    
    /// Full replication to all federation members
    Full,
}

/// Access policy
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AccessPolicy {
    /// Only owner
    Owner,
    
    /// Owner and specific DIDs
    AllowList(Vec<Did>),
    
    /// Anyone except specific DIDs
    DenyList(Vec<Did>),
    
    /// Public (anyone)
    Public,
    
    /// Based on BearDog capability
    Capability(String),
}

/// Waypoint-specific configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaypointConfig {
    /// Can this waypoint anchor slices from other spines
    pub accept_anchors: bool,
    
    /// Maximum concurrent anchored slices
    pub max_anchored_slices: Option<usize>,
    
    /// Maximum anchor depth (for re-lending)
    pub max_anchor_depth: Option<u32>,
    
    /// Allowed origin spines (None = any)
    pub allowed_origins: Option<Vec<SpineId>>,
    
    /// Propagation policy to origin
    pub propagation_policy: PropagationPolicy,
}
```

### 4.3 Spine Builder

```rust
/// Builder for creating spines
pub struct SpineBuilder {
    name: Option<String>,
    spine_type: SpineType,
    config: SpineConfig,
    metadata: HashMap<String, Value>,
}

impl SpineBuilder {
    /// Create a new spine builder
    pub fn new(spine_type: SpineType) -> Self {
        Self {
            name: None,
            spine_type,
            config: SpineConfig::default(),
            metadata: HashMap::new(),
        }
    }
    
    /// Create a personal spine
    pub fn personal() -> Self {
        Self::new(SpineType::Personal)
    }
    
    /// Create a waypoint spine
    pub fn waypoint() -> Self {
        Self::new(SpineType::Waypoint {
            max_anchor_depth: Some(3),
        })
    }
    
    /// Set spine name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    /// Set replication policy
    pub fn with_replication(mut self, policy: ReplicationPolicy) -> Self {
        self.config.replication = policy;
        self
    }
    
    /// Set read access
    pub fn with_read_access(mut self, policy: AccessPolicy) -> Self {
        self.config.read_access = policy;
        self
    }
    
    /// Build the spine (creates genesis entry)
    pub async fn build(
        self,
        owner: Did,
        signer: &impl Signer,
    ) -> Result<(Spine, Entry), LoamError> {
        let spine_id = SpineId::now_v7();
        let created_at = current_timestamp_nanos();
        
        // Create genesis entry
        let genesis_type = EntryType::Genesis {
            spine_id,
            owner: owner.clone(),
            config: self.config.clone(),
        };
        
        // Sign genesis entry
        let genesis_entry = create_genesis_entry(
            genesis_type,
            owner.clone(),
            signer,
        ).await?;
        
        let genesis_hash = genesis_entry.compute_hash();
        
        let spine = Spine {
            id: spine_id,
            name: self.name,
            owner,
            spine_type: self.spine_type,
            config: self.config,
            genesis: genesis_hash,
            tip: genesis_hash,
            height: 1,
            created_at,
            updated_at: created_at,
            state: SpineState::Active,
            metadata: self.metadata,
        };
        
        Ok((spine, genesis_entry))
    }
}
```

---

## 5. Certificate Structure

Certificates are memory-bound objects with ownership and history.

### 5.1 Core Definition

```rust
/// A Loam Certificate (memory-bound object)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Certificate {
    // === Identity ===
    
    /// Unique certificate ID
    pub id: CertificateId,
    
    /// Certificate type
    pub cert_type: CertificateType,
    
    // === Ownership ===
    
    /// Current owner
    pub owner: Did,
    
    /// Current holder (if loaned, different from owner)
    pub holder: Option<Did>,
    
    // === Provenance ===
    
    /// Spine where certificate was minted
    pub mint_spine: SpineId,
    
    /// Mint entry hash
    pub mint_entry: EntryHash,
    
    /// Current spine (where certificate currently lives)
    pub current_spine: SpineId,
    
    /// Current entry hash (latest state)
    pub current_entry: EntryHash,
    
    // === History ===
    
    /// Transfer count
    pub transfer_count: u64,
    
    /// Loan status
    pub loan_status: Option<LoanStatus>,
    
    // === Metadata ===
    
    /// Certificate metadata
    pub metadata: CertificateMetadata,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last update timestamp
    pub updated_at: u64,
}

/// Certificate type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CertificateType {
    // === Digital Ownership ===
    DigitalGameKey {
        platform: String,
        game_id: String,
    },
    
    DigitalCollectible {
        collection: String,
        item_id: String,
        rarity: Option<String>,
    },
    
    DigitalLicense {
        software: String,
        license_type: String,
        seats: Option<u32>,
    },
    
    // === Physical Ownership ===
    VehicleTitle {
        vin: String,
        make: String,
        model: String,
        year: u32,
    },
    
    PropertyDeed {
        parcel_id: String,
        address: String,
        jurisdiction: String,
    },
    
    // === Credentials ===
    AcademicDegree {
        institution: String,
        degree: String,
        field: String,
        year: u32,
    },
    
    ProfessionalLicense {
        authority: String,
        license_type: String,
        license_number: String,
    },
    
    Certification {
        issuer: String,
        cert_name: String,
        expires: Option<u64>,
    },
    
    // === Provenance ===
    ArtworkProvenance {
        artist: String,
        title: String,
        medium: String,
        year: Option<u32>,
    },
    
    BiologicalSample {
        sample_type: String,
        origin: String,
        collection_date: u64,
    },
    
    // === Custom ===
    Custom {
        type_uri: String,
    },
}

/// Certificate metadata
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CertificateMetadata {
    /// Display name
    pub name: Option<String>,
    
    /// Description
    pub description: Option<String>,
    
    /// Image reference
    pub image: Option<PayloadRef>,
    
    /// Custom attributes
    pub attributes: HashMap<String, Value>,
    
    /// External links
    pub links: Vec<ExternalLink>,
}

/// Loan status
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoanStatus {
    /// Loan entry hash
    pub loan_entry: EntryHash,
    
    /// Borrower DID
    pub borrower: Did,
    
    /// Loan terms
    pub terms: LoanTerms,
    
    /// Loan start time
    pub started_at: u64,
    
    /// Expected return time
    pub expires_at: Option<u64>,
    
    /// Waypoint spine (if anchored)
    pub waypoint_spine: Option<SpineId>,
}
```

### 5.2 Certificate History

```rust
/// Certificate ownership record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OwnershipRecord {
    /// Owner at this point
    pub owner: Did,
    
    /// Entry that established ownership
    pub entry: EntryHash,
    
    /// Spine where entry exists
    pub spine: SpineId,
    
    /// Ownership start time
    pub from: u64,
    
    /// Ownership end time (None if current)
    pub until: Option<u64>,
    
    /// How ownership was acquired
    pub acquisition: AcquisitionType,
}

/// How ownership was acquired
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AcquisitionType {
    /// Original mint
    Mint,
    
    /// Transfer from previous owner
    Transfer { from: Did },
    
    /// Inherited (original owner dissolved)
    Inherited { from: Did },
    
    /// Returned from loan
    LoanReturn { borrower: Did },
}

/// Full certificate history
#[derive(Clone, Debug)]
pub struct CertificateHistory {
    pub certificate: Certificate,
    pub ownership_records: Vec<OwnershipRecord>,
    pub loan_records: Vec<LoanRecord>,
    pub operation_records: Vec<OperationRecord>,
}

/// Loan record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoanRecord {
    pub loan_entry: EntryHash,
    pub lender: Did,
    pub borrower: Did,
    pub terms: LoanTerms,
    pub started_at: u64,
    pub ended_at: Option<u64>,
    pub return_entry: Option<EntryHash>,
    pub usage_summary: Option<UsageSummary>,
}
```

---

## 6. Chain Structure

### 6.1 Chain Index

```rust
/// Chain index for efficient traversal
#[derive(Clone, Debug, Default)]
pub struct ChainIndex {
    /// Entry hash → Entry index
    hash_to_index: HashMap<EntryHash, u64>,
    
    /// Entry index → Entry hash
    index_to_hash: BTreeMap<u64, EntryHash>,
    
    /// Entry hash → Next entry hash (forward links)
    next: HashMap<EntryHash, EntryHash>,
    
    /// Entries by type
    by_type: HashMap<String, Vec<EntryHash>>,
    
    /// Entries by committer
    by_committer: HashMap<Did, Vec<EntryHash>>,
    
    /// Certificate entries
    by_certificate: HashMap<CertificateId, Vec<EntryHash>>,
    
    /// Slice entries
    by_slice: HashMap<SliceId, Vec<EntryHash>>,
}

impl ChainIndex {
    /// Index an entry
    pub fn index_entry(&mut self, entry: &Entry) {
        let hash = entry.compute_hash();
        
        // Index by hash and position
        self.hash_to_index.insert(hash, entry.index);
        self.index_to_hash.insert(entry.index, hash);
        
        // Link from previous
        if let Some(prev) = entry.previous {
            self.next.insert(prev, hash);
        }
        
        // Index by type
        let type_key = entry.entry_type.domain().to_string();
        self.by_type.entry(type_key).or_default().push(hash);
        
        // Index by committer
        self.by_committer
            .entry(entry.committer.clone())
            .or_default()
            .push(hash);
        
        // Index by certificate if applicable
        if let Some(cert_id) = extract_certificate_id(&entry.entry_type) {
            self.by_certificate.entry(cert_id).or_default().push(hash);
        }
        
        // Index by slice if applicable
        if let Some(slice_id) = extract_slice_id(&entry.entry_type) {
            self.by_slice.entry(slice_id).or_default().push(hash);
        }
    }
    
    /// Get entry by index
    pub fn get_by_index(&self, index: u64) -> Option<EntryHash> {
        self.index_to_hash.get(&index).copied()
    }
    
    /// Get index by hash
    pub fn get_index(&self, hash: &EntryHash) -> Option<u64> {
        self.hash_to_index.get(hash).copied()
    }
    
    /// Get next entry
    pub fn get_next(&self, hash: &EntryHash) -> Option<EntryHash> {
        self.next.get(hash).copied()
    }
    
    /// Get entries in range
    pub fn get_range(&self, start: u64, end: u64) -> Vec<EntryHash> {
        self.index_to_hash
            .range(start..end)
            .map(|(_, hash)| *hash)
            .collect()
    }
}
```

### 6.2 Chain Verification

```rust
/// Chain verification result
#[derive(Clone, Debug)]
pub struct ChainVerification {
    pub spine_id: SpineId,
    pub entries_verified: u64,
    pub valid: bool,
    pub errors: Vec<ChainError>,
}

/// Chain error
#[derive(Clone, Debug)]
pub enum ChainError {
    /// Previous hash mismatch
    HashMismatch {
        index: u64,
        expected: EntryHash,
        actual: EntryHash,
    },
    
    /// Invalid signature
    InvalidSignature {
        index: u64,
        committer: Did,
    },
    
    /// Missing required attestation
    MissingAttestation {
        index: u64,
        required: AttestationType,
    },
    
    /// Invalid entry type for spine
    InvalidEntryType {
        index: u64,
        entry_type: String,
    },
    
    /// Timestamp regression
    TimestampRegression {
        index: u64,
        previous: u64,
        current: u64,
    },
    
    /// Gap in index
    IndexGap {
        expected: u64,
        actual: u64,
    },
}

/// Verify chain integrity
pub async fn verify_chain(
    spine: &Spine,
    store: &impl EntryStore,
    beardog: &impl BearDogClient,
) -> Result<ChainVerification, LoamError> {
    let mut errors = Vec::new();
    let mut prev_entry: Option<Entry> = None;
    
    // Iterate through all entries
    let entries = store.iter_spine(spine.id);
    pin_mut!(entries);
    
    while let Some(entry_result) = entries.next().await {
        let entry = entry_result?;
        
        // Verify index continuity
        if let Some(prev) = &prev_entry {
            if entry.index != prev.index + 1 {
                errors.push(ChainError::IndexGap {
                    expected: prev.index + 1,
                    actual: entry.index,
                });
            }
            
            // Verify hash link
            let prev_hash = prev.compute_hash();
            if entry.previous != Some(prev_hash) {
                errors.push(ChainError::HashMismatch {
                    index: entry.index,
                    expected: prev_hash,
                    actual: entry.previous.unwrap_or([0u8; 32]),
                });
            }
            
            // Verify timestamp progression
            if entry.timestamp < prev.timestamp {
                errors.push(ChainError::TimestampRegression {
                    index: entry.index,
                    previous: prev.timestamp,
                    current: entry.timestamp,
                });
            }
        }
        
        // Verify signature
        if !entry.verify_signature(beardog).await.unwrap_or(false) {
            errors.push(ChainError::InvalidSignature {
                index: entry.index,
                committer: entry.committer.clone(),
            });
        }
        
        prev_entry = Some(entry);
    }
    
    let entries_verified = prev_entry.map(|e| e.index + 1).unwrap_or(0);
    
    Ok(ChainVerification {
        spine_id: spine.id,
        entries_verified,
        valid: errors.is_empty(),
        errors,
    })
}
```

---

## 7. Proof Structures

### 7.1 Inclusion Proof

```rust
/// Proof that an entry exists in a spine
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InclusionProof {
    /// The entry being proven
    pub entry: Entry,
    
    /// Entry hash
    pub entry_hash: EntryHash,
    
    /// Path from entry to tip (chain of hashes)
    pub path: Vec<EntryHash>,
    
    /// Current tip
    pub tip: EntryHash,
    
    /// Spine ID
    pub spine_id: SpineId,
    
    /// Proof timestamp
    pub timestamp: u64,
    
    /// Optional: signature from spine owner
    pub owner_attestation: Option<Signature>,
}

impl InclusionProof {
    /// Verify this proof
    pub fn verify(&self) -> bool {
        // Verify entry hash matches
        if self.entry.compute_hash() != self.entry_hash {
            return false;
        }
        
        // Verify chain links from entry to tip
        let mut current = self.entry_hash;
        
        for (i, next_hash) in self.path.iter().enumerate() {
            // In a valid chain, each entry links to the previous
            // We're going forward, so next_hash should have `previous = current`
            // For now, just verify we reach the tip
            current = *next_hash;
        }
        
        current == self.tip
    }
    
    /// Generate an inclusion proof
    pub async fn generate(
        spine: &Spine,
        entry_hash: EntryHash,
        store: &impl EntryStore,
    ) -> Result<Self, LoamError> {
        let entry = store.get_by_hash(&entry_hash).await?
            .ok_or(LoamError::EntryNotFound(entry_hash))?;
        
        // Build path from entry to tip
        let mut path = Vec::new();
        let mut current = entry_hash;
        
        while current != spine.tip {
            let next = store.get_next(&current).await?
                .ok_or(LoamError::Internal("Broken chain".into()))?;
            path.push(next);
            current = next;
        }
        
        Ok(Self {
            entry,
            entry_hash,
            path,
            tip: spine.tip,
            spine_id: spine.id,
            timestamp: current_timestamp_nanos(),
            owner_attestation: None,
        })
    }
}
```

### 7.2 Certificate Proof

```rust
/// Proof of certificate ownership and history
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
    pub timestamp: u64,
}

impl CertificateProof {
    /// Verify this proof
    pub fn verify(&self) -> bool {
        // Verify mint proof
        if !self.mint_proof.verify() {
            return false;
        }
        
        // Verify all transfer proofs
        for proof in &self.transfer_proofs {
            if !proof.verify() {
                return false;
            }
        }
        
        // Verify current proof
        if !self.current_proof.verify() {
            return false;
        }
        
        // Verify transfer chain integrity
        // (Each transfer should reference the previous owner)
        // ... additional verification logic
        
        true
    }
}
```

---

## 8. Type Aliases and Utilities

```rust
/// BearDog DID
pub type Did = String;

/// BearDog signature
pub type Signature = Vec<u8>;

/// RhizoCrypt session ID
pub type SessionId = uuid::Uuid;

/// RhizoCrypt slice ID
pub type SliceId = uuid::Uuid;

/// SweetGrass braid ID
pub type BraidId = uuid::Uuid;

/// Peer ID for replication
pub type PeerId = String;

/// Generic value type
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

/// Payload reference
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PayloadRef {
    pub hash: ContentHash,
    pub size: u64,
    pub mime_type: Option<String>,
}

/// Get current timestamp in nanoseconds
pub fn current_timestamp_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos() as u64
}
```

---

## 9. References

- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [WAYPOINT_SEMANTICS.md](./WAYPOINT_SEMANTICS.md) — Waypoint spines
- [CERTIFICATE_LAYER.md](./CERTIFICATE_LAYER.md) — Certificate operations
- [LOAMSPINE_SPECIFICATION.md](./LOAMSPINE_SPECIFICATION.md) — Full specification

---

*LoamSpine: The permanent record that gives memory its meaning.*

