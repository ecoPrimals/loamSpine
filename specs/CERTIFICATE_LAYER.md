<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# LoamSpine — Certificate Layer Specification

**Version**: 1.0.0  
**Status**: Active  
**Last Updated**: December 22, 2025

---

## 1. Overview

The **Loam Certificate Layer** defines memory-bound objects—digital assets with verifiable ownership, transferability, and complete provenance history. Certificates are the primary mechanism for representing value in the ecoPrimals ecosystem.

Unlike blockchain NFTs, Loam Certificates:
- Are **sovereign** (owned by individuals, not platforms)
- Are **lendable** (can be temporarily transferred)
- Have **complete history** (every transfer, loan, and operation is recorded)
- Support **recursive stacking** (certificates can reference other certificates)

---

## 2. Certificate Lifecycle

```
┌─────────────────────────────────────────────────────────────────┐
│                    Certificate Lifecycle                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────┐    ┌──────────┐    ┌───────────┐    ┌──────────┐ │
│  │  MINTED  │───▶│  ACTIVE  │───▶│  LOANED   │───▶│  ACTIVE  │ │
│  │          │    │          │    │           │    │          │ │
│  │ Genesis  │    │ Held by  │    │ Held by   │    │ Returned │ │
│  │ entry    │    │ owner    │    │ borrower  │    │          │ │
│  └──────────┘    └────┬─────┘    └───────────┘    └──────────┘ │
│                       │                                         │
│                       │ Transfer                                │
│                       ▼                                         │
│                  ┌──────────┐                                   │
│                  │  ACTIVE  │                                   │
│                  │          │                                   │
│                  │ New owner│                                   │
│                  └────┬─────┘                                   │
│                       │                                         │
│                       │ Revoke/Expire                           │
│                       ▼                                         │
│                  ┌──────────┐                                   │
│                  │ REVOKED  │                                   │
│                  │          │                                   │
│                  │ No longer│                                   │
│                  │ valid    │                                   │
│                  └──────────┘                                   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Certificate Data Structures

### 3.1 Core Certificate

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A Loam Certificate (memory-bound object)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Certificate {
    // === Identity ===
    
    /// Unique certificate ID
    pub id: CertificateId,
    
    /// Certificate type
    pub cert_type: CertificateType,
    
    /// Certificate version (for schema evolution)
    pub version: u32,
    
    // === Ownership ===
    
    /// Current owner
    pub owner: Did,
    
    /// Current holder (if loaned, different from owner)
    pub holder: Option<Did>,
    
    // === Provenance ===
    
    /// Minting information
    pub mint_info: MintInfo,
    
    /// Current location
    pub current_location: CertificateLocation,
    
    // === State ===
    
    /// Certificate state
    pub state: CertificateState,
    
    /// Transfer count
    pub transfer_count: u64,
    
    /// Active loan (if any)
    pub active_loan: Option<LoanInfo>,
    
    // === Metadata ===
    
    /// Certificate metadata
    pub metadata: CertificateMetadata,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last update timestamp  
    pub updated_at: u64,
}

/// Minting information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MintInfo {
    /// Who minted the certificate
    pub minter: Did,
    
    /// Spine where minted
    pub spine: SpineId,
    
    /// Mint entry hash
    pub entry: EntryHash,
    
    /// Mint timestamp
    pub timestamp: u64,
    
    /// Minting authority (if delegated)
    pub authority: Option<MintingAuthority>,
}

/// Current certificate location
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificateLocation {
    /// Current spine
    pub spine: SpineId,
    
    /// Latest state entry
    pub entry: EntryHash,
    
    /// Entry index in spine
    pub index: u64,
}

/// Certificate state
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CertificateState {
    /// Active and owned
    Active,
    
    /// Currently loaned out
    Loaned {
        loan_entry: EntryHash,
    },
    
    /// Pending transfer (escrow)
    PendingTransfer {
        transfer_entry: EntryHash,
        to: Did,
    },
    
    /// Revoked (no longer valid)
    Revoked {
        revoke_entry: EntryHash,
        reason: RevocationReason,
    },
    
    /// Expired (time-limited certificate)
    Expired {
        expired_at: u64,
    },
}

/// Loan information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoanInfo {
    /// Loan entry
    pub loan_entry: EntryHash,
    
    /// Borrower
    pub borrower: Did,
    
    /// Loan terms
    pub terms: LoanTerms,
    
    /// Start time
    pub started_at: u64,
    
    /// Expected end time
    pub expires_at: Option<u64>,
    
    /// Waypoint spine (if anchored)
    pub waypoint: Option<SpineId>,
    
    /// Waypoint anchor entry
    pub waypoint_anchor: Option<EntryHash>,
}
```

### 3.2 Certificate Types

```rust
/// Certificate type taxonomy
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CertificateType {
    // ==================== Digital Assets ====================
    
    /// Digital game license
    DigitalGame {
        platform: GamePlatform,
        game_id: String,
        edition: Option<String>,
        dlc_included: Vec<String>,
    },
    
    /// In-game item
    GameItem {
        game_id: String,
        item_type: GameItemType,
        item_id: String,
        attributes: HashMap<String, Value>,
    },
    
    /// Digital collectible
    DigitalCollectible {
        collection_id: String,
        item_number: Option<u64>,
        total_supply: Option<u64>,
        rarity: Option<Rarity>,
        traits: Vec<Trait>,
    },
    
    /// Software license
    SoftwareLicense {
        software_id: String,
        license_type: LicenseType,
        seats: Option<u32>,
        features: Vec<String>,
        expires: Option<u64>,
    },
    
    /// Digital media (book, music, video)
    DigitalMedia {
        media_type: MediaType,
        content_id: String,
        format: String,
        drm_info: Option<DrmInfo>,
    },
    
    // ==================== Physical Assets ====================
    
    /// Vehicle title
    VehicleTitle {
        vin: String,
        make: String,
        model: String,
        year: u32,
        jurisdiction: String,
        lien_holder: Option<Did>,
    },
    
    /// Property deed
    PropertyDeed {
        parcel_id: String,
        address: Address,
        jurisdiction: String,
        property_type: PropertyType,
    },
    
    /// Artwork provenance
    ArtworkProvenance {
        artist: String,
        title: String,
        medium: String,
        dimensions: Option<Dimensions>,
        year_created: Option<u32>,
        exhibition_history: Vec<ExhibitionRecord>,
    },
    
    // ==================== Credentials ====================
    
    /// Academic credential
    AcademicCredential {
        institution: String,
        credential_type: CredentialType,
        field_of_study: String,
        date_awarded: u64,
        honors: Option<String>,
    },
    
    /// Professional license
    ProfessionalLicense {
        issuing_authority: String,
        license_type: String,
        license_number: String,
        jurisdiction: String,
        issued_date: u64,
        expires_date: Option<u64>,
        specializations: Vec<String>,
    },
    
    /// Professional certification
    Certification {
        issuer: String,
        certification_name: String,
        certification_id: String,
        issued_date: u64,
        expires_date: Option<u64>,
        verification_url: Option<String>,
    },
    
    // ==================== Scientific ====================
    
    /// Biological sample
    BiologicalSample {
        sample_type: String,
        origin: SampleOrigin,
        collection_date: u64,
        storage_conditions: String,
        chain_of_custody: Vec<CustodyRecord>,
    },
    
    /// Research dataset
    ResearchDataset {
        dataset_id: String,
        title: String,
        description: String,
        methodology: String,
        data_format: String,
        access_level: AccessLevel,
    },
    
    /// Experimental result
    ExperimentalResult {
        experiment_id: String,
        protocol: String,
        result_type: String,
        confidence: f64,
        reproducibility_score: Option<f64>,
    },
    
    // ==================== Financial ====================
    
    /// Tokenized share
    TokenizedShare {
        entity_id: String,
        share_class: String,
        quantity: f64,
        par_value: Option<f64>,
        voting_rights: bool,
    },
    
    /// Bond certificate
    BondCertificate {
        issuer: String,
        series: String,
        face_value: f64,
        coupon_rate: f64,
        maturity_date: u64,
    },
    
    // ==================== Custom ====================
    
    /// Custom certificate type
    Custom {
        type_uri: String,
        schema_version: u32,
    },
}

/// Game item types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameItemType {
    Weapon { damage_type: String },
    Armor { defense_type: String },
    Consumable { effect: String },
    Cosmetic { slot: String },
    Mount { speed: f64 },
    Pet { abilities: Vec<String> },
    Currency { unit: String },
    Crafting { material_type: String },
    Blueprint { crafts: String },
    Container { capacity: u32 },
    Quest { quest_id: String },
    Achievement { achievement_id: String },
    Custom { item_class: String },
}

/// Rarity levels
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythic,
    Unique,
    Custom(String),
}
```

### 3.3 Certificate Metadata

```rust
/// Certificate metadata
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CertificateMetadata {
    /// Display name
    pub name: Option<String>,
    
    /// Description
    pub description: Option<String>,
    
    /// Image/thumbnail reference
    pub image: Option<PayloadRef>,
    
    /// Animation/video reference  
    pub animation: Option<PayloadRef>,
    
    /// External URI
    pub external_url: Option<String>,
    
    /// Background color (hex)
    pub background_color: Option<String>,
    
    /// Custom attributes
    pub attributes: Vec<MetadataAttribute>,
    
    /// External links
    pub links: Vec<ExternalLink>,
    
    /// Tags
    pub tags: Vec<String>,
    
    /// Localized names
    pub localized_names: HashMap<String, String>,
    
    /// Localized descriptions
    pub localized_descriptions: HashMap<String, String>,
}

/// Metadata attribute
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetadataAttribute {
    pub trait_type: String,
    pub value: Value,
    pub display_type: Option<DisplayType>,
    pub max_value: Option<Value>,
}

/// Display type for attributes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DisplayType {
    Number,
    BoostNumber,
    BoostPercentage,
    Date,
    String,
    Rank,
}
```

---

## 4. Certificate Operations

### 4.1 Minting

```rust
/// Mint a new certificate
pub async fn mint_certificate(
    request: MintRequest,
    spine_store: &impl SpineStore,
    beardog: &impl BearDogClient,
    signer: &impl Signer,
) -> Result<Certificate, LoamError> {
    // 1. Validate minting authority
    let has_authority = beardog
        .check_permission(&request.minter, "loamspine:certificate:mint", &request.cert_type.type_name())
        .await?;
    
    if !has_authority.allowed {
        return Err(LoamError::MintNotAuthorized);
    }
    
    // 2. Get spine
    let spine = spine_store.get_spine(request.spine_id).await?
        .ok_or(LoamError::SpineNotFound(request.spine_id))?;
    
    // Verify minter owns spine or has write permission
    if spine.owner != request.minter {
        let can_write = beardog
            .check_permission(&request.minter, &format!("loamspine:spine:{}:write", spine.id), "mint")
            .await?;
        if !can_write.allowed {
            return Err(LoamError::SpineNotOwned(request.spine_id));
        }
    }
    
    // 3. Generate certificate ID
    let cert_id = CertificateId::now_v7();
    
    // 4. Create mint entry
    let mint_entry = EntryBuilder::new(EntryType::CertificateMint {
        cert_id,
        cert_type: request.cert_type.clone(),
        initial_owner: request.initial_owner.clone(),
        metadata: request.metadata.clone(),
    })
    .with_payload_if(request.metadata.image.clone())
    .build(&spine, request.minter.clone(), signer)
    .await?;
    
    let entry_hash = spine_store.append_entry(spine.id, mint_entry).await?;
    
    // 5. Create certificate
    let now = current_timestamp_nanos();
    let certificate = Certificate {
        id: cert_id,
        cert_type: request.cert_type,
        version: 1,
        owner: request.initial_owner.clone(),
        holder: None,
        mint_info: MintInfo {
            minter: request.minter,
            spine: spine.id,
            entry: entry_hash,
            timestamp: now,
            authority: request.authority,
        },
        current_location: CertificateLocation {
            spine: spine.id,
            entry: entry_hash,
            index: spine.height,
        },
        state: CertificateState::Active,
        transfer_count: 0,
        active_loan: None,
        metadata: request.metadata,
        created_at: now,
        updated_at: now,
    };
    
    Ok(certificate)
}

/// Mint request
#[derive(Clone, Debug)]
pub struct MintRequest {
    /// Who is minting
    pub minter: Did,
    
    /// Which spine to mint on
    pub spine_id: SpineId,
    
    /// Certificate type
    pub cert_type: CertificateType,
    
    /// Initial owner (usually same as minter)
    pub initial_owner: Did,
    
    /// Certificate metadata
    pub metadata: CertificateMetadata,
    
    /// Minting authority (if delegated)
    pub authority: Option<MintingAuthority>,
}
```

### 4.2 Transfer

```rust
/// Transfer a certificate to a new owner
pub async fn transfer_certificate(
    cert_id: CertificateId,
    to: Did,
    conditions: Option<TransferConditions>,
    cert_store: &impl CertificateStore,
    spine_store: &impl SpineStore,
    beardog: &impl BearDogClient,
    signer: &impl Signer,
) -> Result<TransferResult, LoamError> {
    // 1. Get certificate
    let cert = cert_store.get(cert_id).await?
        .ok_or(LoamError::CertificateNotFound(cert_id))?;
    
    // 2. Validate state
    match &cert.state {
        CertificateState::Active => {}
        CertificateState::Loaned { .. } => {
            return Err(LoamError::CertificateLoaned);
        }
        CertificateState::PendingTransfer { .. } => {
            return Err(LoamError::TransferPending);
        }
        CertificateState::Revoked { .. } => {
            return Err(LoamError::CertificateRevoked);
        }
        CertificateState::Expired { .. } => {
            return Err(LoamError::CertificateExpired);
        }
    }
    
    // 3. Validate ownership
    // (In real implementation, requester DID would come from auth context)
    
    // 4. Handle conditions (escrow if needed)
    if let Some(conditions) = &conditions {
        if conditions.requires_escrow() {
            return create_escrow_transfer(cert, to, conditions, spine_store, signer).await;
        }
    }
    
    // 5. Create transfer entry on current spine
    let current_spine = spine_store.get_spine(cert.current_location.spine).await?
        .ok_or(LoamError::SpineNotFound(cert.current_location.spine))?;
    
    let transfer_entry = EntryBuilder::new(EntryType::CertificateTransfer {
        cert_id,
        from: cert.owner.clone(),
        to: to.clone(),
        conditions: conditions.clone(),
    })
    .build(&current_spine, cert.owner.clone(), signer)
    .await?;
    
    let entry_hash = spine_store.append_entry(current_spine.id, transfer_entry).await?;
    
    // 6. Update certificate
    let updated_cert = Certificate {
        owner: to.clone(),
        current_location: CertificateLocation {
            spine: current_spine.id,
            entry: entry_hash,
            index: current_spine.height,
        },
        transfer_count: cert.transfer_count + 1,
        updated_at: current_timestamp_nanos(),
        ..cert
    };
    
    cert_store.update(updated_cert.clone()).await?;
    
    Ok(TransferResult {
        certificate: updated_cert,
        transfer_entry: entry_hash,
    })
}

/// Transfer conditions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransferConditions {
    /// Require payment
    pub payment: Option<PaymentCondition>,
    
    /// Require attestation from specific parties
    pub required_attestations: Vec<Did>,
    
    /// Time lock (cannot complete before)
    pub time_lock: Option<u64>,
    
    /// Expiry (must complete before)
    pub expiry: Option<u64>,
    
    /// Escrow agent (if needed)
    pub escrow_agent: Option<Did>,
}

impl TransferConditions {
    pub fn requires_escrow(&self) -> bool {
        self.payment.is_some() || 
        !self.required_attestations.is_empty() ||
        self.escrow_agent.is_some()
    }
}
```

### 4.3 Loaning

```rust
/// Loan a certificate temporarily
pub async fn loan_certificate(
    cert_id: CertificateId,
    borrower: Did,
    terms: LoanTerms,
    cert_store: &impl CertificateStore,
    spine_store: &impl SpineStore,
    signer: &impl Signer,
) -> Result<LoanResult, LoamError> {
    // 1. Get and validate certificate
    let cert = cert_store.get(cert_id).await?
        .ok_or(LoamError::CertificateNotFound(cert_id))?;
    
    if !matches!(cert.state, CertificateState::Active) {
        return Err(LoamError::CertificateNotActive);
    }
    
    // 2. Create loan entry
    let current_spine = spine_store.get_spine(cert.current_location.spine).await?
        .ok_or(LoamError::SpineNotFound(cert.current_location.spine))?;
    
    let loan_entry = EntryBuilder::new(EntryType::CertificateLoan {
        cert_id,
        lender: cert.owner.clone(),
        borrower: borrower.clone(),
        terms: terms.clone(),
    })
    .build(&current_spine, cert.owner.clone(), signer)
    .await?;
    
    let entry_hash = spine_store.append_entry(current_spine.id, loan_entry).await?;
    
    // 3. Compute expiry
    let now = current_timestamp_nanos();
    let expires_at = terms.duration.map(|d| now + d.as_nanos() as u64);
    
    // 4. Update certificate
    let updated_cert = Certificate {
        holder: Some(borrower.clone()),
        state: CertificateState::Loaned {
            loan_entry: entry_hash,
        },
        active_loan: Some(LoanInfo {
            loan_entry: entry_hash,
            borrower: borrower.clone(),
            terms: terms.clone(),
            started_at: now,
            expires_at,
            waypoint: None,
            waypoint_anchor: None,
        }),
        updated_at: now,
        ..cert
    };
    
    cert_store.update(updated_cert.clone()).await?;
    
    Ok(LoanResult {
        certificate: updated_cert,
        loan_entry: entry_hash,
    })
}

/// Loan terms
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoanTerms {
    /// Loan duration
    pub duration: Option<Duration>,
    
    /// Grace period after expiry
    pub grace_period: Option<Duration>,
    
    /// Automatic return on expiry
    pub auto_return: bool,
    
    /// Allowed operations
    pub allowed_operations: Option<HashSet<String>>,
    
    /// Forbidden operations
    pub forbidden_operations: HashSet<String>,
    
    /// Can borrower sub-loan
    pub allow_subloan: bool,
    
    /// Maximum subloan depth
    pub max_subloan_depth: Option<u32>,
    
    /// What to propagate on return
    pub propagation_policy: PropagationPolicy,
    
    /// Usage reporting requirements
    pub usage_reporting: UsageReportingRequirement,
}

/// Usage reporting requirement
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum UsageReportingRequirement {
    #[default]
    None,
    Summary,
    Detailed { 
        include_operations: bool,
        include_timestamps: bool,
    },
}
```

### 4.4 Return from Loan

```rust
/// Return a loaned certificate
pub async fn return_certificate(
    cert_id: CertificateId,
    usage_summary: Option<UsageSummary>,
    cert_store: &impl CertificateStore,
    spine_store: &impl SpineStore,
    signer: &impl Signer,
) -> Result<ReturnResult, LoamError> {
    // 1. Get certificate
    let cert = cert_store.get(cert_id).await?
        .ok_or(LoamError::CertificateNotFound(cert_id))?;
    
    // 2. Validate state
    let loan_info = match &cert.state {
        CertificateState::Loaned { loan_entry } => {
            cert.active_loan.as_ref()
                .ok_or(LoamError::Internal("Missing loan info".into()))?
        }
        _ => return Err(LoamError::CertificateNotLoaned),
    };
    
    // 3. Create return entry
    let current_spine = spine_store.get_spine(cert.current_location.spine).await?
        .ok_or(LoamError::SpineNotFound(cert.current_location.spine))?;
    
    let return_entry = EntryBuilder::new(EntryType::CertificateReturn {
        cert_id,
        loan_entry: loan_info.loan_entry,
        usage_summary: usage_summary.clone(),
    })
    .build(&current_spine, loan_info.borrower.clone(), signer)
    .await?;
    
    let entry_hash = spine_store.append_entry(current_spine.id, return_entry).await?;
    
    // 4. Update certificate
    let updated_cert = Certificate {
        holder: None,
        state: CertificateState::Active,
        active_loan: None,
        updated_at: current_timestamp_nanos(),
        ..cert
    };
    
    cert_store.update(updated_cert.clone()).await?;
    
    Ok(ReturnResult {
        certificate: updated_cert,
        return_entry: entry_hash,
    })
}

/// Usage summary from loan period
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsageSummary {
    /// Total usage duration
    pub duration_nanos: u64,
    
    /// Number of uses/operations
    pub operation_count: u64,
    
    /// Types of operations performed
    pub operation_types: Vec<String>,
    
    /// Waypoint summary (if anchored)
    pub waypoint_summary: Option<WaypointSummary>,
    
    /// Custom usage data
    pub custom: HashMap<String, Value>,
}
```

---

## 5. Certificate History

### 5.1 History Retrieval

```rust
/// Get full certificate history
pub async fn get_certificate_history(
    cert_id: CertificateId,
    cert_store: &impl CertificateStore,
    spine_store: &impl SpineStore,
) -> Result<CertificateHistory, LoamError> {
    let cert = cert_store.get(cert_id).await?
        .ok_or(LoamError::CertificateNotFound(cert_id))?;
    
    let mut ownership_records = Vec::new();
    let mut loan_records = Vec::new();
    let mut operation_records = Vec::new();
    
    // Start from mint entry
    let mint_entry = spine_store
        .get_by_hash(&cert.mint_info.entry)
        .await?
        .ok_or(LoamError::EntryNotFound(cert.mint_info.entry))?;
    
    ownership_records.push(OwnershipRecord {
        owner: cert.mint_info.minter.clone(),
        entry: cert.mint_info.entry,
        spine: cert.mint_info.spine,
        from: cert.mint_info.timestamp,
        until: None,
        acquisition: AcquisitionType::Mint,
    });
    
    // Traverse all certificate entries
    let entries = spine_store
        .get_entries_by_certificate(cert_id)
        .await?;
    
    let mut current_owner = cert.mint_info.minter.clone();
    let mut current_from = cert.mint_info.timestamp;
    
    for entry in entries {
        match &entry.entry_type {
            EntryType::CertificateTransfer { from, to, .. } => {
                // Close previous ownership
                if let Some(last) = ownership_records.last_mut() {
                    last.until = Some(entry.timestamp);
                }
                
                ownership_records.push(OwnershipRecord {
                    owner: to.clone(),
                    entry: entry.compute_hash(),
                    spine: cert.current_location.spine,
                    from: entry.timestamp,
                    until: None,
                    acquisition: AcquisitionType::Transfer { from: from.clone() },
                });
                
                current_owner = to.clone();
                current_from = entry.timestamp;
            }
            
            EntryType::CertificateLoan { lender, borrower, terms, .. } => {
                loan_records.push(LoanRecord {
                    loan_entry: entry.compute_hash(),
                    lender: lender.clone(),
                    borrower: borrower.clone(),
                    terms: terms.clone(),
                    started_at: entry.timestamp,
                    ended_at: None,
                    return_entry: None,
                    usage_summary: None,
                });
            }
            
            EntryType::CertificateReturn { loan_entry, usage_summary, .. } => {
                // Find and update matching loan
                if let Some(loan) = loan_records.iter_mut().find(|l| l.loan_entry == *loan_entry) {
                    loan.ended_at = Some(entry.timestamp);
                    loan.return_entry = Some(entry.compute_hash());
                    loan.usage_summary = usage_summary.clone();
                }
            }
            
            _ => {}
        }
    }
    
    Ok(CertificateHistory {
        certificate: cert,
        ownership_records,
        loan_records,
        operation_records,
    })
}
```

### 5.2 Provenance Proof

```rust
/// Generate provenance proof for a certificate
pub async fn generate_provenance_proof(
    cert_id: CertificateId,
    cert_store: &impl CertificateStore,
    spine_store: &impl SpineStore,
) -> Result<ProvenanceProof, LoamError> {
    let history = get_certificate_history(cert_id, cert_store, spine_store).await?;
    
    // Generate inclusion proofs for key entries
    let mut entry_proofs = Vec::new();
    
    // Mint entry proof
    let mint_proof = InclusionProof::generate(
        &spine_store.get_spine(history.certificate.mint_info.spine).await?.unwrap(),
        history.certificate.mint_info.entry,
        spine_store,
    ).await?;
    entry_proofs.push(mint_proof);
    
    // Transfer entry proofs
    for record in &history.ownership_records {
        if !matches!(record.acquisition, AcquisitionType::Mint) {
            let spine = spine_store.get_spine(record.spine).await?.unwrap();
            let proof = InclusionProof::generate(&spine, record.entry, spine_store).await?;
            entry_proofs.push(proof);
        }
    }
    
    // Current state proof
    let current_proof = InclusionProof::generate(
        &spine_store.get_spine(history.certificate.current_location.spine).await?.unwrap(),
        history.certificate.current_location.entry,
        spine_store,
    ).await?;
    
    Ok(ProvenanceProof {
        cert_id,
        current_owner: history.certificate.owner.clone(),
        mint_proof: entry_proofs.first().cloned().unwrap(),
        transfer_proofs: entry_proofs[1..].to_vec(),
        current_proof,
        history_summary: HistorySummary {
            transfer_count: history.ownership_records.len() as u64 - 1,
            loan_count: history.loan_records.len() as u64,
            age_nanos: current_timestamp_nanos() - history.certificate.created_at,
        },
        timestamp: current_timestamp_nanos(),
    })
}

/// Provenance proof
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProvenanceProof {
    pub cert_id: CertificateId,
    pub current_owner: Did,
    pub mint_proof: InclusionProof,
    pub transfer_proofs: Vec<InclusionProof>,
    pub current_proof: InclusionProof,
    pub history_summary: HistorySummary,
    pub timestamp: u64,
}

/// History summary
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistorySummary {
    pub transfer_count: u64,
    pub loan_count: u64,
    pub age_nanos: u64,
}
```

---

## 6. Certificate Verification

```rust
/// Verify certificate authenticity and ownership
pub async fn verify_certificate(
    cert_id: CertificateId,
    expected_owner: Option<&Did>,
    cert_store: &impl CertificateStore,
    spine_store: &impl SpineStore,
    beardog: &impl BearDogClient,
) -> Result<VerificationResult, LoamError> {
    // 1. Get certificate
    let cert = cert_store.get(cert_id).await?
        .ok_or(LoamError::CertificateNotFound(cert_id))?;
    
    // 2. Verify chain integrity
    let chain_valid = verify_certificate_chain(cert_id, spine_store, beardog).await?;
    
    // 3. Verify current owner
    let owner_valid = expected_owner
        .map(|expected| &cert.owner == expected)
        .unwrap_or(true);
    
    // 4. Verify mint entry
    let mint_entry = spine_store.get_by_hash(&cert.mint_info.entry).await?
        .ok_or(LoamError::EntryNotFound(cert.mint_info.entry))?;
    
    let mint_valid = mint_entry.verify_signature(beardog).await?;
    
    // 5. Verify current state entry
    let current_entry = spine_store.get_by_hash(&cert.current_location.entry).await?
        .ok_or(LoamError::EntryNotFound(cert.current_location.entry))?;
    
    let current_valid = current_entry.verify_signature(beardog).await?;
    
    // 6. Build result
    let all_valid = chain_valid && owner_valid && mint_valid && current_valid;
    
    Ok(VerificationResult {
        cert_id,
        valid: all_valid,
        owner: cert.owner.clone(),
        state: cert.state.clone(),
        checks: VerificationChecks {
            chain_valid,
            owner_valid,
            mint_valid,
            current_valid,
        },
        timestamp: current_timestamp_nanos(),
    })
}

/// Verification result
#[derive(Clone, Debug)]
pub struct VerificationResult {
    pub cert_id: CertificateId,
    pub valid: bool,
    pub owner: Did,
    pub state: CertificateState,
    pub checks: VerificationChecks,
    pub timestamp: u64,
}

/// Individual verification checks
#[derive(Clone, Debug)]
pub struct VerificationChecks {
    pub chain_valid: bool,
    pub owner_valid: bool,
    pub mint_valid: bool,
    pub current_valid: bool,
}
```

---

## 7. References

- [LOAMSPINE_SPECIFICATION.md](./LOAMSPINE_SPECIFICATION.md) — Full specification
- [DATA_MODEL.md](./DATA_MODEL.md) — Data structures
- [WAYPOINT_SEMANTICS.md](./WAYPOINT_SEMANTICS.md) — Loan mechanics
- [API_SPECIFICATION.md](./API_SPECIFICATION.md) — API definitions

---

*LoamSpine: The permanent record that gives memory its meaning.*

