<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# LoamSpine — Waypoint Semantics Specification

**Version**: 1.0.0  
**Status**: Active  
**Last Updated**: December 22, 2025

---

## 1. Overview

**Waypoint Spines** are a special type of LoamSpine that provides local permanence for borrowed state. When a slice is "loaned" from one entity to another, the borrower's waypoint spine anchors that slice, creating a local permanent record that **cannot propagate upward** to the origin spine or global commons.

This enables:
- **Lending without trust** — Owner retains control even while borrower has use
- **Local provenance** — Borrower's usage is recorded in their own spine
- **Selective return** — Only summary data returns to origin
- **Depth-limited relending** — Controlled subloan chains

---

## 2. The Waypoint Layer

### 2.1 Layer Position

```
════════════════════════════════════════════════════════════════════
                    THE RHIZO-LOAM LAYER CAKE
════════════════════════════════════════════════════════════════════

        ┌─────────────────────────────────────────────────────┐
        │                  gAIa COMMONS                       │
        │           (Global permanent anchor)                 │
        │                                                     │
        │        Entries can NEVER propagate here from       │
        │        waypoint spines directly                     │
        └─────────────────────────┬───────────────────────────┘
                                  │
        ┌─────────────────────────┼───────────────────────────┐
        │                         │                           │
        ▼                         ▼                           ▼
    ┌────────┐              ┌────────┐                  ┌────────┐
    │ Spine A│              │ Spine B│                  │ Spine C│
    │(Owner) │              │(Owner) │                  │(Owner) │
    └───┬────┘              └───┬────┘                  └───┬────┘
        │                       │                           │
        │ CANONICAL SPINE LAYER ╪═══════════════════════════╪═════
        │ (Personal/Org truth)  │                           │
        │                       │                           │
        │                       │                           │
        │ ┌──────────────────┐  │                           │
        │ │   Slice Loan     │  │                           │
        │ │   (from A to X)  │  │                           │
        │ └────────┬─────────┘  │                           │
        │          │            │                           │
        │          ▼            │                           │
        │     ┌─────────┐       │                           │
        │     │Waypoint │       │                           │
        │     │ Spine X │       │                           │
        │     └────┬────┘       │                           │
        │          │            │                           │
        │ WAYPOINT LAYER ═══════╪═══════════════════════════╪═════
        │ (Local permanence,    │                           │
        │  NO upward propagation)                           │
        │          │            │                           │
        │          │ Can relend │                           │
        │          ▼            │                           │
        │     ┌─────────┐       │                           │
        │     │Waypoint │       │                           │
        │     │ Spine Y │       │                           │
        │     └─────────┘       │                           │
        │          │            │                           │
        │          ▼            │                           │
        │       (Depth limit    │                           │
        │        reached)       │                           │
        
════════════════════════════════════════════════════════════════════
```

### 2.2 Key Properties

| Property | Canonical Spine | Waypoint Spine |
|----------|----------------|----------------|
| **Persistence** | Permanent | Permanent |
| **Propagation** | Upward to federation/gAIa | Never upward |
| **Ownership** | Full owner control | Borrower control of local ops |
| **Entry Types** | All | Restricted (anchor, operation, departure) |
| **Replication** | Per policy | Never |
| **Relending** | N/A | Depth-limited |
| **Return** | N/A | Mandatory (loan terms) |

---

## 3. Waypoint Spine Structure

### 3.1 Waypoint Configuration

```rust
/// Waypoint spine configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaypointConfig {
    /// Accept anchors from external spines
    pub accept_anchors: bool,
    
    /// Maximum concurrent anchored slices
    pub max_anchored_slices: Option<usize>,
    
    /// Maximum anchor depth (for relending)
    /// 0 = cannot relend
    /// 1 = can relend once
    /// None = unlimited (not recommended)
    pub max_anchor_depth: Option<u32>,
    
    /// Allowed origin spines (None = any)
    pub allowed_origins: Option<Vec<SpineId>>,
    
    /// Forbidden origin spines
    pub forbidden_origins: Vec<SpineId>,
    
    /// Propagation policy to origin on return
    pub propagation_policy: PropagationPolicy,
    
    /// Auto-return on expiry
    pub auto_return_expired: bool,
    
    /// Grace period before forced return
    pub expiry_grace_period: Duration,
    
    /// Require attestation for operations
    pub operation_attestation: AttestationRequirement,
}

impl Default for WaypointConfig {
    fn default() -> Self {
        Self {
            accept_anchors: true,
            max_anchored_slices: Some(100),
            max_anchor_depth: Some(2),
            allowed_origins: None,
            forbidden_origins: Vec::new(),
            propagation_policy: PropagationPolicy::SummaryOnly,
            auto_return_expired: true,
            expiry_grace_period: Duration::from_secs(3600), // 1 hour
            operation_attestation: AttestationRequirement::None,
        }
    }
}

/// Propagation policy on return
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum PropagationPolicy {
    /// Never propagate anything
    Never,
    
    /// Propagate summary statistics only
    #[default]
    SummaryOnly,
    
    /// Propagate specific event types
    Selective {
        allowed_types: Vec<String>,
    },
    
    /// Full propagation (rare, requires owner consent)
    Full {
        require_owner_signature: bool,
    },
}
```

### 3.2 Waypoint Spine Type

```rust
/// Create a waypoint spine
impl SpineBuilder {
    /// Create a waypoint spine with default config
    pub fn waypoint() -> Self {
        Self::new(SpineType::Waypoint {
            max_anchor_depth: Some(2),
        }).with_config(SpineConfig {
            replication: ReplicationPolicy::None, // Never replicate waypoints
            retention: RetentionPolicy::UntilEmpty, // GC when all slices returned
            waypoint: Some(WaypointConfig::default()),
            ..Default::default()
        })
    }
    
    /// Create a waypoint with custom depth limit
    pub fn waypoint_with_depth(max_depth: u32) -> Self {
        Self::new(SpineType::Waypoint {
            max_anchor_depth: Some(max_depth),
        }).with_config(SpineConfig {
            replication: ReplicationPolicy::None,
            waypoint: Some(WaypointConfig {
                max_anchor_depth: Some(max_depth),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
```

---

## 4. Slice Anchoring

### 4.1 Anchor Request

```rust
/// Request to anchor a slice at a waypoint
#[derive(Clone, Debug)]
pub struct AnchorRequest {
    /// Slice being anchored
    pub slice_id: SliceId,
    
    /// Origin spine (where slice was checked out)
    pub origin_spine: SpineId,
    
    /// Origin entry (the checkout entry)
    pub origin_entry: EntryHash,
    
    /// Original owner of the slice
    pub owner: Did,
    
    /// Current depth (0 = direct from owner)
    pub current_depth: u32,
    
    /// Slice terms (from loan agreement)
    pub terms: SliceTerms,
    
    /// Waypoint where slice is being anchored
    pub waypoint_spine: SpineId,
    
    /// Requester (must be waypoint owner)
    pub requester: Did,
}

/// Slice terms for waypoint anchoring
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SliceTerms {
    /// Loan duration (None = until manual return)
    pub duration: Option<Duration>,
    
    /// Grace period after expiry
    pub grace_period: Option<Duration>,
    
    /// Allowed operations
    pub allowed_operations: Option<HashSet<String>>,
    
    /// Forbidden operations
    pub forbidden_operations: HashSet<String>,
    
    /// Can this slice be relent
    pub allow_relend: bool,
    
    /// Maximum relend depth from this point
    pub max_relend_depth: Option<u32>,
    
    /// Propagation policy
    pub propagation: PropagationPolicy,
    
    /// Custom constraints
    pub constraints: HashMap<String, Value>,
}
```

### 4.2 Anchor Flow

```rust
/// Anchor a slice at a waypoint spine
pub async fn anchor_slice(
    request: AnchorRequest,
    waypoint_store: &impl SpineStore,
    origin_store: &impl SpineStore,
    beardog: &impl BearDogClient,
    signer: &impl Signer,
) -> Result<AnchorResponse, LoamError> {
    // 1. Validate waypoint spine
    let waypoint = waypoint_store.get_spine(request.waypoint_spine).await?
        .ok_or(LoamError::SpineNotFound(request.waypoint_spine))?;
    
    if !matches!(waypoint.spine_type, SpineType::Waypoint { .. }) {
        return Err(LoamError::NotWaypointSpine(request.waypoint_spine));
    }
    
    // 2. Check waypoint configuration
    let config = waypoint.config.waypoint.as_ref()
        .ok_or(LoamError::InvalidWaypointConfig)?;
    
    if !config.accept_anchors {
        return Err(LoamError::AnchorNotAllowed);
    }
    
    // Check origin allowed
    if let Some(allowed) = &config.allowed_origins {
        if !allowed.contains(&request.origin_spine) {
            return Err(LoamError::OriginNotAllowed(request.origin_spine));
        }
    }
    if config.forbidden_origins.contains(&request.origin_spine) {
        return Err(LoamError::OriginForbidden(request.origin_spine));
    }
    
    // Check depth limit
    if let Some(max_depth) = config.max_anchor_depth {
        if request.current_depth >= max_depth {
            return Err(LoamError::DepthLimitReached {
                current: request.current_depth,
                max: max_depth,
            });
        }
    }
    
    // Check concurrent anchors
    if let Some(max) = config.max_anchored_slices {
        let current = count_active_anchors(&waypoint, waypoint_store).await?;
        if current >= max {
            return Err(LoamError::TooManyAnchors { current, max });
        }
    }
    
    // 3. Verify slice ownership and loan validity
    let origin_checkout = origin_store.get_by_hash(&request.origin_entry).await?
        .ok_or(LoamError::EntryNotFound(request.origin_entry))?;
    
    // Verify this is actually a slice checkout
    if !matches!(origin_checkout.entry_type, EntryType::SliceCheckout { .. }) {
        return Err(LoamError::InvalidSliceCheckout);
    }
    
    // 4. Create anchor entry
    let anchor_entry = EntryBuilder::new(EntryType::SliceAnchor {
        slice_id: request.slice_id,
        origin_spine: request.origin_spine,
        origin_entry: request.origin_entry,
        terms: request.terms.clone(),
    })
    .with_metadata("depth", request.current_depth + 1)
    .with_metadata("owner", request.owner.clone())
    .build(&waypoint, request.requester.clone(), signer)
    .await?;
    
    // 5. Append to waypoint spine
    let anchor_hash = waypoint_store.append_entry(request.waypoint_spine, anchor_entry).await?;
    
    // 6. Track active anchor
    let anchor = ActiveAnchor {
        slice_id: request.slice_id,
        anchor_entry: anchor_hash,
        origin_spine: request.origin_spine,
        origin_entry: request.origin_entry,
        owner: request.owner,
        depth: request.current_depth + 1,
        terms: request.terms,
        anchored_at: current_timestamp_nanos(),
        expires_at: compute_expiry(&request.terms),
        state: AnchorState::Active,
    };
    
    Ok(AnchorResponse {
        anchor,
        anchor_hash,
    })
}

/// Active anchor tracking
#[derive(Clone, Debug)]
pub struct ActiveAnchor {
    pub slice_id: SliceId,
    pub anchor_entry: EntryHash,
    pub origin_spine: SpineId,
    pub origin_entry: EntryHash,
    pub owner: Did,
    pub depth: u32,
    pub terms: SliceTerms,
    pub anchored_at: u64,
    pub expires_at: Option<u64>,
    pub state: AnchorState,
}

/// Anchor state
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnchorState {
    Active,
    Expired,
    Departing { started_at: u64 },
    Departed { departure_entry: EntryHash },
}
```

---

## 5. Waypoint Operations

### 5.1 Operation Entry

When a borrower uses a sliced item at their waypoint, operations are recorded:

```rust
/// Record an operation on an anchored slice
pub async fn record_operation(
    slice_id: SliceId,
    operation: SliceOperationType,
    payload: Option<PayloadRef>,
    waypoint_store: &impl SpineStore,
    signer: &impl Signer,
) -> Result<EntryHash, LoamError> {
    // 1. Find active anchor
    let anchor = find_active_anchor(slice_id, waypoint_store).await?
        .ok_or(LoamError::SliceNotAnchored(slice_id))?;
    
    // 2. Validate operation allowed
    if !is_operation_allowed(&anchor.terms, &operation) {
        return Err(LoamError::OperationNotAllowed(operation.name()));
    }
    
    // 3. Check expiry
    if let Some(expires) = anchor.expires_at {
        if current_timestamp_nanos() > expires {
            return Err(LoamError::SliceExpired(slice_id));
        }
    }
    
    // 4. Get waypoint spine
    let waypoint = waypoint_store.get_spine(anchor.waypoint_spine()).await?
        .ok_or(LoamError::SpineNotFound(anchor.waypoint_spine()))?;
    
    // 5. Create operation entry
    let operation_entry = EntryBuilder::new(EntryType::SliceOperation {
        slice_id,
        operation: operation.clone(),
        payload,
    })
    .build(&waypoint, waypoint.owner.clone(), signer)
    .await?;
    
    // 6. Append to waypoint
    let entry_hash = waypoint_store.append_entry(waypoint.id, operation_entry).await?;
    
    Ok(entry_hash)
}

/// Slice operation types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SliceOperationType {
    // === General ===
    Use {
        action: String,
        duration: Option<Duration>,
    },
    
    View {
        viewport: Option<String>,
    },
    
    // === Gaming ===
    EquipItem,
    UnequipItem,
    UseInCombat {
        target: Option<Did>,
        outcome: String,
    },
    ExtractWith {
        success: bool,
    },
    
    // === Documents ===
    Read {
        pages: Option<usize>,
    },
    Edit {
        operation_type: String,
    },
    Export {
        format: String,
    },
    
    // === Scientific ===
    RunExperiment {
        protocol: String,
    },
    TakeReading {
        instrument: String,
        reading_type: String,
    },
    
    // === Custom ===
    Custom {
        operation_name: String,
        metadata: HashMap<String, Value>,
    },
}

impl SliceOperationType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Use { .. } => "use",
            Self::View { .. } => "view",
            Self::EquipItem => "equip",
            Self::UnequipItem => "unequip",
            Self::UseInCombat { .. } => "combat",
            Self::ExtractWith { .. } => "extract",
            Self::Read { .. } => "read",
            Self::Edit { .. } => "edit",
            Self::Export { .. } => "export",
            Self::RunExperiment { .. } => "experiment",
            Self::TakeReading { .. } => "reading",
            Self::Custom { operation_name, .. } => operation_name,
        }
    }
}
```

### 5.2 Operation Validation

```rust
/// Check if operation is allowed by slice terms
fn is_operation_allowed(terms: &SliceTerms, operation: &SliceOperationType) -> bool {
    let op_name = operation.name();
    
    // Check forbidden list first
    if terms.forbidden_operations.contains(op_name) {
        return false;
    }
    
    // If allowed list exists, must be in it
    if let Some(allowed) = &terms.allowed_operations {
        return allowed.contains(op_name);
    }
    
    // Default: allowed
    true
}
```

---

## 6. Slice Departure (Return)

### 6.1 Departure Flow

```rust
/// Initiate slice departure from waypoint
pub async fn depart_slice(
    slice_id: SliceId,
    reason: DepartureReason,
    waypoint_store: &impl SpineStore,
    origin_store: &impl SpineStore,
    signer: &impl Signer,
) -> Result<DepartureResponse, LoamError> {
    // 1. Find active anchor
    let anchor = find_active_anchor(slice_id, waypoint_store).await?
        .ok_or(LoamError::SliceNotAnchored(slice_id))?;
    
    // 2. Get waypoint spine
    let waypoint = waypoint_store.get_spine(anchor.waypoint_spine()).await?
        .ok_or(LoamError::SpineNotFound(anchor.waypoint_spine()))?;
    
    // 3. Compute usage summary
    let summary = compute_waypoint_summary(slice_id, &anchor, waypoint_store).await?;
    
    // 4. Create departure entry
    let departure_entry = EntryBuilder::new(EntryType::SliceDeparture {
        slice_id,
        reason: reason.clone(),
        summary: summary.clone(),
    })
    .build(&waypoint, waypoint.owner.clone(), signer)
    .await?;
    
    let departure_hash = waypoint_store.append_entry(waypoint.id, departure_entry).await?;
    
    // 5. Apply propagation policy
    let propagated = apply_propagation_policy(
        &anchor,
        &summary,
        &anchor.terms.propagation,
        origin_store,
        signer,
    ).await?;
    
    // 6. If this was a relent, notify parent waypoint
    if anchor.depth > 1 {
        notify_parent_waypoint(slice_id, &departure_hash, waypoint_store).await?;
    }
    
    Ok(DepartureResponse {
        departure_hash,
        summary,
        propagated,
    })
}

/// Reasons for slice departure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DepartureReason {
    /// Loan term expired
    Expired,
    
    /// Borrower manually returned
    ManualReturn,
    
    /// Owner recalled slice
    OwnerRecall,
    
    /// Relent to another waypoint
    Relend {
        target_waypoint: SpineId,
    },
    
    /// Slice resolved through RhizoCrypt session
    SessionResolution {
        session_id: SessionId,
        outcome: SessionOutcome,
    },
    
    /// Administrative action
    Administrative {
        reason: String,
    },
}

/// Waypoint usage summary
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaypointSummary {
    /// Slice ID
    pub slice_id: SliceId,
    
    /// Total time anchored
    pub duration_nanos: u64,
    
    /// Number of operations performed
    pub operation_count: u64,
    
    /// Operation types performed
    pub operation_types: Vec<String>,
    
    /// First operation timestamp
    pub first_operation: Option<u64>,
    
    /// Last operation timestamp
    pub last_operation: Option<u64>,
    
    /// Hash of full operation log (for verification)
    pub operations_hash: ContentHash,
    
    /// Whether slice was relent
    pub was_relent: bool,
    
    /// Relend depth reached
    pub max_relend_depth: u32,
    
    /// Custom summary data
    pub custom: HashMap<String, Value>,
}
```

### 6.2 Propagation to Origin

```rust
/// Apply propagation policy to origin spine
async fn apply_propagation_policy(
    anchor: &ActiveAnchor,
    summary: &WaypointSummary,
    policy: &PropagationPolicy,
    origin_store: &impl SpineStore,
    signer: &impl Signer,
) -> Result<PropagationResult, LoamError> {
    match policy {
        PropagationPolicy::Never => {
            // Nothing propagated
            Ok(PropagationResult::None)
        }
        
        PropagationPolicy::SummaryOnly => {
            // Propagate only the summary (not individual operations)
            let origin = origin_store.get_spine(anchor.origin_spine).await?
                .ok_or(LoamError::SpineNotFound(anchor.origin_spine))?;
            
            let return_entry = EntryBuilder::new(EntryType::SliceReturn {
                slice_id: anchor.slice_id,
                checkout_entry: anchor.origin_entry,
                waypoint_spine: Some(anchor.waypoint_spine()),
                waypoint_summary: Some(summary.clone()),
            })
            .build(&origin, origin.owner.clone(), signer)
            .await?;
            
            let entry_hash = origin_store.append_entry(origin.id, return_entry).await?;
            
            Ok(PropagationResult::Summary { entry_hash })
        }
        
        PropagationPolicy::Selective { allowed_types } => {
            // Propagate only specific operation types
            let operations = get_operations_by_types(
                anchor.slice_id,
                allowed_types,
                origin_store,
            ).await?;
            
            // Create entries for each allowed operation
            let mut propagated = Vec::new();
            for op in operations {
                // ... propagate each allowed operation
            }
            
            Ok(PropagationResult::Selective { entries: propagated })
        }
        
        PropagationPolicy::Full { require_owner_signature } => {
            if *require_owner_signature {
                // Need owner to sign full propagation
                return Err(LoamError::OwnerSignatureRequired);
            }
            
            // Propagate everything
            let operations = get_all_operations(anchor.slice_id, origin_store).await?;
            
            let mut propagated = Vec::new();
            for op in operations {
                // ... propagate each operation
            }
            
            Ok(PropagationResult::Full { entries: propagated })
        }
    }
}

/// Result of propagation
#[derive(Clone, Debug)]
pub enum PropagationResult {
    None,
    Summary { entry_hash: EntryHash },
    Selective { entries: Vec<EntryHash> },
    Full { entries: Vec<EntryHash> },
}
```

---

## 7. Relending (Depth Chain)

### 7.1 Relend Flow

```rust
/// Relend a slice to another waypoint
pub async fn relend_slice(
    slice_id: SliceId,
    target_waypoint: SpineId,
    new_terms: SliceTerms,
    current_waypoint_store: &impl SpineStore,
    target_waypoint_store: &impl SpineStore,
    signer: &impl Signer,
) -> Result<RelendResponse, LoamError> {
    // 1. Find current anchor
    let current_anchor = find_active_anchor(slice_id, current_waypoint_store).await?
        .ok_or(LoamError::SliceNotAnchored(slice_id))?;
    
    // 2. Check if relending is allowed
    if !current_anchor.terms.allow_relend {
        return Err(LoamError::RelendNotAllowed);
    }
    
    // 3. Check depth limit
    let new_depth = current_anchor.depth + 1;
    if let Some(max) = current_anchor.terms.max_relend_depth {
        if new_depth > max {
            return Err(LoamError::RelendDepthExceeded {
                current: current_anchor.depth,
                max,
            });
        }
    }
    
    // 4. Validate new terms don't exceed current terms
    validate_relend_terms(&current_anchor.terms, &new_terms)?;
    
    // 5. Create departure entry on current waypoint
    let departure_reason = DepartureReason::Relend {
        target_waypoint,
    };
    let departure = depart_slice(
        slice_id,
        departure_reason,
        current_waypoint_store,
        &NoOpStore, // Don't propagate to origin yet
        signer,
    ).await?;
    
    // 6. Create anchor request for target waypoint
    let anchor_request = AnchorRequest {
        slice_id,
        origin_spine: current_anchor.origin_spine,
        origin_entry: current_anchor.origin_entry,
        owner: current_anchor.owner.clone(),
        current_depth: new_depth,
        terms: new_terms,
        waypoint_spine: target_waypoint,
        requester: target_waypoint_owner,
    };
    
    let anchor = anchor_slice(
        anchor_request,
        target_waypoint_store,
        current_waypoint_store,
        beardog,
        signer,
    ).await?;
    
    Ok(RelendResponse {
        departure_hash: departure.departure_hash,
        new_anchor: anchor.anchor,
    })
}

/// Validate relend terms don't exceed current terms
fn validate_relend_terms(current: &SliceTerms, new: &SliceTerms) -> Result<(), LoamError> {
    // Duration can only be shorter
    if let (Some(current_dur), Some(new_dur)) = (current.duration, new.duration) {
        if new_dur > current_dur {
            return Err(LoamError::RelendTermsExceedParent("duration".into()));
        }
    }
    
    // Forbidden operations must be superset
    for forbidden in &current.forbidden_operations {
        if !new.forbidden_operations.contains(forbidden) {
            return Err(LoamError::RelendTermsExceedParent(
                format!("missing forbidden: {}", forbidden),
            ));
        }
    }
    
    // Allowed operations must be subset (if both specified)
    if let (Some(current_allowed), Some(new_allowed)) = 
        (&current.allowed_operations, &new.allowed_operations) 
    {
        for op in new_allowed {
            if !current_allowed.contains(op) {
                return Err(LoamError::RelendTermsExceedParent(
                    format!("operation not allowed: {}", op),
                ));
            }
        }
    }
    
    // Relend depth must be less
    if let Some(new_depth) = new.max_relend_depth {
        if let Some(current_depth) = current.max_relend_depth {
            if new_depth >= current_depth {
                return Err(LoamError::RelendTermsExceedParent("relend_depth".into()));
            }
        }
    }
    
    Ok(())
}
```

### 7.2 Depth Tracking

```
Original Owner (Spine A)
    │
    │ Loan (depth=0)
    ▼
Borrower X (Waypoint X)
    │ depth=1
    │
    │ Relend (depth=1)
    ▼
Borrower Y (Waypoint Y)
    │ depth=2
    │
    │ Relend (depth=2)
    ▼
Borrower Z (Waypoint Z)
    │ depth=3
    │
    └── (max depth=3, cannot relend further)
    
Return Flow:
Z returns to Y → Y returns to X → X returns to A
Each return includes usage summary from that level
```

---

## 8. Expiry Handling

### 8.1 Background Expiry Task

```rust
/// Background task for handling expired anchors
pub async fn expiry_sweep_task(
    waypoint_store: Arc<impl SpineStore>,
    interval: Duration,
) {
    let mut interval_timer = tokio::time::interval(interval);
    
    loop {
        interval_timer.tick().await;
        
        // Find all waypoint spines
        let waypoints = waypoint_store
            .list_spines(SpineFilter::Waypoint)
            .await
            .unwrap_or_default();
        
        for waypoint in waypoints {
            // Find expired anchors
            let expired = find_expired_anchors(&waypoint, &waypoint_store).await
                .unwrap_or_default();
            
            for anchor in expired {
                // Check grace period
                let grace = waypoint.config.waypoint
                    .as_ref()
                    .map(|c| c.expiry_grace_period)
                    .unwrap_or(Duration::from_secs(3600));
                
                let expiry_with_grace = anchor.expires_at.unwrap_or(0) + 
                    grace.as_nanos() as u64;
                
                if current_timestamp_nanos() > expiry_with_grace {
                    // Force return
                    if let Err(e) = force_return(&anchor, &waypoint_store).await {
                        tracing::error!(
                            slice_id = %anchor.slice_id,
                            error = %e,
                            "Failed to force return expired slice"
                        );
                    }
                }
            }
        }
    }
}

/// Force return an expired anchor
async fn force_return(
    anchor: &ActiveAnchor,
    store: &impl SpineStore,
) -> Result<(), LoamError> {
    depart_slice(
        anchor.slice_id,
        DepartureReason::Expired,
        store,
        store, // Use same store for origin
        &system_signer(),
    ).await?;
    
    Ok(())
}
```

---

## 9. Security Considerations

### 9.1 Depth Limits

- Default max depth: 2 (can relend once)
- Owner can set to 0 (no relending)
- Unlimited depth is possible but discouraged

### 9.2 Term Inheritance

- Relent terms can only be MORE restrictive
- Cannot extend duration beyond parent
- Cannot add operations that parent forbade
- Cannot increase relend depth

### 9.3 Propagation Control

- Default: SummaryOnly (minimal exposure)
- Full propagation requires owner consent
- Never propagate to gAIa/global without explicit entry

---

## 10. References

- [LOAMSPINE_SPECIFICATION.md](./LOAMSPINE_SPECIFICATION.md) — Full specification
- [DATA_MODEL.md](./DATA_MODEL.md) — Data structures
- [CERTIFICATE_LAYER.md](./CERTIFICATE_LAYER.md) — Certificate operations
- [RhizoCrypt SLICE_SEMANTICS.md](../../rhizoCrypt/specs/SLICE_SEMANTICS.md) — Slice modes

---

*LoamSpine: The permanent record that gives memory its meaning.*

