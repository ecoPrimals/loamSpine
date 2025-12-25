# 🦴🏠 Demo: Storage Capability (LoamSpine + NestGate)

**Goal**: Demonstrate conceptual storage integration pattern  
**Time**: 5 minutes  
**Status**: Conceptual (NestGate binary not yet available)

---

## 🎯 What You'll Learn

- Storage capability pattern (local + distributed)
- Backup and restore workflows
- Disaster recovery strategy
- Inter-primal storage integration
- Gaps in current implementation

---

## 🚀 Quick Start

```bash
# Run the conceptual demo
./demo.sh
```

---

## 📋 What This Demo Shows

### 1. Dual Storage Strategy

```
┌─────────────────┐         ┌──────────────────┐
│   LoamSpine     │  Sync   │    NestGate      │
│  (Local/Fast)   │────────▶│ (Distributed)    │
│                 │         │                  │
│  - Fast access  │         │ - Durability     │
│  - Volatile     │         │ - Redundancy     │
│  - In-memory    │         │ - Recovery       │
└─────────────────┘         └──────────────────┘
```

### 2. Backup Workflow

```
1. Create Session → 2. Commit Entries → 3. Finalize
                                            ↓
                                    4. Trigger Backup
                                            ↓
                                    5. Send to NestGate
                                            ↓
                                    6. Confirm Storage
```

### 3. Restore Workflow

```
1. Detect Data Loss → 2. Query NestGate → 3. Retrieve Backup
                                               ↓
                                       4. Restore Locally
                                               ↓
                                       5. Verify Integrity
```

---

## 🔍 What You'll See

```
================================================================
  🦴 LoamSpine + 🏠 NestGate: Storage Capability Demo
  (Conceptual — NestGate binary not yet available)
================================================================

Step 1: Understanding Storage Capability Integration...

What is NestGate?
  - Distributed storage primal
  - Provides persistent, redundant storage
  - Capability: 'persistent-storage'

Why integrate LoamSpine + NestGate?
  - LoamSpine: Fast local ledger
  - NestGate: Durable remote backup
  - Together: Best of both worlds

Note: NestGate binary not yet in ../../bins/
This demo shows the pattern using local filesystem

Step 2: Preparing environment...
✓ Environment ready

Step 3: Starting LoamSpine server...
✓ LoamSpine running (PID: 12345)

Step 4: Creating session with multiple entries...
✓ Session created
   Session ID: 550e8400-e29b-41d4-a716-446655440000
   ✓ Entry 1 committed
   ✓ Entry 2 committed
   ✓ Entry 3 committed
✓ Session finalized

Step 5: Backing up session to external storage...
   (Simulating NestGate with local filesystem)
✓ Session backed up
   Backup location: /tmp/nestgate-backups/session_550e8400...

Step 6: Simulating local data loss...
⚠️  Local storage deleted

Step 7: Restoring session from external storage...
   (Simulating restoration from NestGate)
✓ Session restored from backup

Step 8: Verifying restored session...
⚠️  Session retrieval not yet implemented
   Gap: Need session loading on startup

================================================================
  Demo Complete!
================================================================

Gaps discovered:
  ⚠️  NestGate binary not yet available
  ⚠️  Session loading not implemented
  ⚠️  Backup mechanism not automatic
```

---

## 🎓 Learning Points

### Pattern 1: Capability Discovery

```rust
// Discover storage capability via Songbird
let songbird = SongbirdClient::new("http://localhost:8082");
let storage_services = songbird
    .discover_capability("persistent-storage")
    .await?;

// Get NestGate endpoint
let nestgate = storage_services
    .iter()
    .find(|s| s.name == "nestgate")
    .ok_or("NestGate not available")?;

// Use NestGate client
let storage = NestGateClient::new(&nestgate.endpoint);
```

### Pattern 2: Automatic Backup

```rust
// Background backup task
async fn backup_loop(
    loamspine: Arc<LoamSpine>,
    nestgate: Arc<NestGateClient>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
    
    loop {
        interval.tick().await;
        
        // Get finalized sessions since last backup
        let sessions = loamspine.get_finalized_sessions_since(last_backup_time).await?;
        
        for session in sessions {
            match nestgate.backup_session(&session).await {
                Ok(backup_id) => {
                    log::info!("Backed up session {}: {}", session.id, backup_id);
                    loamspine.mark_backed_up(&session.id, &backup_id).await?;
                }
                Err(e) => {
                    log::warn!("Failed to backup session {}: {}", session.id, e);
                    // Retry later
                }
            }
        }
    }
}
```

### Pattern 3: Session Loading on Startup

```rust
// Load sessions from storage on startup
pub async fn load_sessions(storage_path: &Path) -> Result<Vec<Session>> {
    let mut sessions = Vec::new();
    
    // Scan storage directory
    for entry in std::fs::read_dir(storage_path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            // Load session
            if let Ok(session) = Session::load_from_path(&entry.path()).await {
                sessions.push(session);
            }
        }
    }
    
    log::info!("Loaded {} sessions from storage", sessions.len());
    Ok(sessions)
}
```

### Pattern 4: Disaster Recovery

```rust
// Restore all sessions from NestGate
pub async fn disaster_recovery(
    nestgate: &NestGateClient,
    loamspine: &mut LoamSpine,
) -> Result<usize> {
    // Query NestGate for all backups
    let backups = nestgate.list_backups().await?;
    
    let mut restored = 0;
    for backup in backups {
        // Download session data
        let session_data = nestgate.download_backup(&backup.id).await?;
        
        // Restore to local storage
        loamspine.restore_session(session_data).await?;
        restored += 1;
    }
    
    log::info!("Restored {} sessions from NestGate", restored);
    Ok(restored)
}
```

---

## 💡 Key Concepts

### Local vs Distributed Storage

**Local Storage (LoamSpine)**:
- ✅ Fast access (in-memory + disk)
- ✅ Low latency
- ❌ Volatile (machine failure = data loss)
- ❌ Single point of failure

**Distributed Storage (NestGate)**:
- ✅ Durable (replicated across nodes)
- ✅ Disaster recovery
- ✅ Geographic distribution
- ❌ Higher latency
- ❌ Network dependency

**Best Strategy**: Use both!
- Write to local storage (fast)
- Async backup to NestGate (durable)
- Restore from NestGate on failure

### Backup Policies

**Immediate Backup**:
```rust
// Backup immediately on finalization
session.finalize().await?;
nestgate.backup_session(&session).await?;
```
- ✅ Maximum durability
- ❌ Higher latency

**Periodic Backup**:
```rust
// Backup every 5 minutes
tokio::spawn(backup_loop(loamspine, nestgate));
```
- ✅ Lower latency
- ⚠️ Potential data loss (up to 5 minutes)

**On-Demand Backup**:
```rust
// Explicit backup call
loamspine.backup_now(&session_id).await?;
```
- ✅ User control
- ⚠️ Requires manual triggering

### Recovery Strategies

**Full Recovery**:
- Restore all sessions from NestGate
- Rebuild complete state
- Slowest, most thorough

**Incremental Recovery**:
- Restore only missing sessions
- Compare local vs NestGate
- Faster, more efficient

**Lazy Recovery**:
- Load sessions on-demand
- First access triggers restore
- Fastest startup, slower first access

---

## 📊 Use Cases

### Use Case 1: Machine Failure

```
1. LoamSpine server crashes
2. Local storage lost
3. Start new LoamSpine instance
4. Query NestGate for backups
5. Restore all sessions
6. Resume operation
```

### Use Case 2: Geographic Replication

```
1. LoamSpine in Region A backs up to NestGate
2. NestGate replicates to Region B
3. New LoamSpine in Region B
4. Restores from NestGate
5. Both regions have same data
```

### Use Case 3: Compliance Archival

```
1. LoamSpine finalizes session
2. Backup to NestGate with retention policy
3. Local storage can be purged
4. NestGate keeps for compliance (7 years)
5. Restore on audit request
```

---

## 🔧 Gaps Discovered

This demo reveals several implementation gaps:

### Gap 1: NestGate Binary Not Available

**Status**: Blocking real integration

**Impact**: Cannot test actual NestGate API

**Workaround**: Simulate with local filesystem

**Resolution**: Wait for NestGate phase 2 completion

### Gap 2: Session Loading Not Implemented

**Status**: Critical for recovery

**Impact**: Sessions lost on restart

**Implementation Needed**:
```rust
// src/service/session_loader.rs
pub async fn load_sessions_on_startup(
    storage_path: &Path,
) -> Result<HashMap<SessionId, Session>> {
    // Scan storage directory
    // Load each session
    // Rebuild in-memory state
}
```

**Files to Modify**:
- `crates/loam-spine-core/src/service/session_loader.rs` (new)
- `crates/loam-spine-cli/src/main.rs` — Call loader on startup

### Gap 3: Automatic Backup Not Implemented

**Status**: Important for durability

**Impact**: No persistent backup

**Implementation Needed**:
```rust
// src/service/backup_manager.rs
pub struct BackupManager {
    nestgate_client: Arc<NestGateClient>,
    backup_policy: BackupPolicy,
}

impl BackupManager {
    pub async fn start_backup_loop(&self) {
        // Background task
        // Monitor finalized sessions
        // Backup to NestGate
    }
}
```

**Files to Modify**:
- `crates/loam-spine-core/src/service/backup_manager.rs` (new)
- `crates/loam-spine-core/src/service/lifecycle.rs` — Integrate backup manager

---

## 🔧 Troubleshooting

### Error: "NestGate not found"

This is expected! NestGate binary not yet available.

**Current**: Simulated with local filesystem

**Future**: Real NestGate integration

### Error: "Session not loaded on restart"

This is a known gap!

**Cause**: Session loading not implemented

**Workaround**: Keep LoamSpine running (don't restart)

**Fix**: Implement session loading (Gap #2)

---

## 📈 Performance Characteristics

| Operation | Local | NestGate | Combined |
|-----------|-------|----------|----------|
| Write | ~1ms | ~50ms | ~1ms (async backup) |
| Read | ~0.5ms | ~20ms | ~0.5ms (cache hit) |
| Finalize | ~5ms | N/A | ~5ms + async backup |
| Restore | N/A | ~100ms | One-time cost |

**Strategy**: Write to local, async backup to NestGate, read from local

---

## ➡️ Next Steps

After completing this demo:
- **Implement**: Session loading on startup (Gap #2)
- **Implement**: Automatic backup mechanism (Gap #3)
- **Wait for**: NestGate binary availability
- **Integrate**: Real NestGate API
- **Deep Dive**: `../../specs/STORAGE_BACKENDS.md`

---

## 🎯 Success Criteria

You'll know this demo worked if you see:
- ✅ Session created and populated
- ✅ Backup simulated (conceptual)
- ✅ Restore simulated (conceptual)
- ✅ Gaps clearly identified
- ✅ Integration pattern understood

---

**Updated**: December 25, 2025  
**Status**: Conceptual (awaiting NestGate binary)  
**Gaps Discovered**: Session loading, automatic backup  
**Principle**: Local performance + distributed durability

🦴🏠 **LoamSpine + NestGate: Fast and durable together**
