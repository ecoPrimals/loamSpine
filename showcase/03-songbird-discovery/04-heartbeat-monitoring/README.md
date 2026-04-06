# 🦴 Demo: Heartbeat Monitoring & Health Checks

**Goal**: Demonstrate health monitoring and failure detection  
**Time**: 5 minutes  
**Prerequisites**: Songbird binary at `../bins/songbird-orchestrator`

---

## 🎯 What You'll Learn

- Health check endpoint patterns (/health, /health/live, /health/ready)
- Heartbeat failure detection
- Service state transitions (healthy → degraded → stale → deregistered)
- Auto-recovery mechanisms

---

## 🚀 Quick Start

```bash
# Run the demo
./demo.sh
```

---

## 📋 What This Demo Shows

### 1. Health Check Endpoints

LoamSpine should expose three health endpoints:

**GET /health** - Detailed Status
```json
{
  "status": "healthy",
  "version": "0.9.16",
  "uptime_seconds": 3600,
  "dependencies": {
    "storage": "healthy",
    "songbird": "healthy"
  },
  "capabilities": [
    "persistent-ledger",
    "waypoint-anchoring"
  ]
}
```

**GET /health/live** - Kubernetes Liveness
```json
{
  "alive": true
}
```
*Purpose*: Is the process running? (Kubernetes restarts if false)

**GET /health/ready** - Kubernetes Readiness
```json
{
  "ready": true
}
```
*Purpose*: Ready for traffic? (Kubernetes removes from load balancer if false)

### 2. Service States

```
HEALTHY → Everything working
   ↓
DEGRADED → Running but some capabilities unavailable
   ↓
STALE → No heartbeat for 90s (3 cycles)
   ↓
AUTO-DEREGISTERED → No heartbeat for 180s (6 cycles)
```

### 3. Failure Detection Timeline

```
t=0s:   Service registered, heartbeat starts
t=30s:  First heartbeat ✓
t=60s:  Second heartbeat ✓
t=90s:  Heartbeat missed (1st time) ⚠️
t=120s: Heartbeat missed (2nd time) ⚠️
t=150s: Heartbeat missed (3rd time) → Mark as STALE
t=180s: Heartbeat missed (6th time) → AUTO-DEREGISTER
```

---

## 💡 Key Concepts

### Liveness vs Readiness

**Liveness**: "Is the process alive?"
- Process crashed? → Not alive
- Deadlocked? → Not alive
- Running? → Alive

**Readiness**: "Can it handle requests?"
- Initializing? → Not ready
- Database down? → Not ready
- Fully operational? → Ready

### Retry with Exponential Backoff

When heartbeat fails, retry with increasing delays:
```
Attempt 1: Retry after 10s
Attempt 2: Retry after 30s
Attempt 3: Retry after 60s
Attempt 4: Retry after 120s
Attempt 5+: Retry after 120s (max)
```

### Graceful Degradation

Service can continue running in degraded mode:
```rust
match check_storage_health().await {
    Ok(_) => ServiceState::Healthy,
    Err(e) => {
        log::warn!("Storage degraded: {e}");
        // Still serve requests, but mark as degraded
        ServiceState::Degraded
    }
}
```

---

## 🔍 What You'll See

```
================================================================
  🦴 LoamSpine: Heartbeat Monitoring & Health Checks
================================================================

Step 1: Checking prerequisites...
✓ Songbird binary found

Step 2: Starting Songbird...
✓ Songbird running

Step 3: Registering LoamSpine...
✓ LoamSpine registered

Step 4: Testing health check endpoints...
   Testing /health (detailed status)...
   Response: {"status":"service_not_running"}
   Note: LoamSpine not actually running (demonstration only)

   Expected in production:
   /health        → Detailed status with dependencies
   /health/live   → Simple liveness check
   /health/ready  → Readiness for traffic

Step 5: Simulating heartbeat with failure and recovery...
   Cycle 1: Successful heartbeat
   ✓ Heartbeat sent (healthy)

   Cycle 2: Successful heartbeat
   ✓ Heartbeat sent (healthy)

   Cycle 3: Simulating failure (degraded state)
   ⚠️  Heartbeat sent (degraded)
   Service running but some capabilities unavailable

   Cycle 4: Recovery (back to healthy)
   ✓ Heartbeat sent (healthy)
   Service recovered from degraded state

Step 6: Heartbeat failure detection...
   Simulating missed heartbeats...
   t=30s: No heartbeat
   t=60s: No heartbeat
   t=90s: No heartbeat (3 cycles missed)
   ⚠️  Service marked as STALE by Songbird

   t=120s: No heartbeat
   t=150s: No heartbeat
   t=180s: No heartbeat (6 cycles missed)
   ✗ Service AUTO-DEREGISTERED by Songbird

✓ Failure detection mechanism demonstrated

================================================================
  Demo Complete!
================================================================
```

---

## 🎓 Learning Points

### Pattern 1: Health Check Implementation

```rust
// src/health.rs
pub async fn health_check() -> Json<HealthStatus> {
    let storage_health = check_storage().await;
    let songbird_health = check_songbird().await;
    
    let status = match (storage_health, songbird_health) {
        (Ok(_), Ok(_)) => ServiceStatus::Healthy,
        (Ok(_), Err(_)) => ServiceStatus::Degraded, // Can continue without discovery
        (Err(_), _) => ServiceStatus::Error,  // Storage critical
    };
    
    Json(HealthStatus {
        status,
        version: env!("CARGO_PKG_VERSION"),
        uptime: get_uptime(),
        dependencies: Dependencies {
            storage: storage_health.is_ok(),
            songbird: songbird_health.is_ok(),
        },
    })
}
```

### Pattern 2: Heartbeat with Retry

```rust
// Background task with retry logic
async fn heartbeat_with_retry(songbird: SongbirdClient) {
    let mut failures = 0;
    let backoff = [10, 30, 60, 120];  // seconds
    
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        
        match songbird.heartbeat().await {
            Ok(_) => {
                failures = 0;  // Reset on success
                log::debug!("Heartbeat sent");
            }
            Err(e) => {
                failures += 1;
                let delay = backoff.get(failures).unwrap_or(&120);
                log::warn!("Heartbeat failed: {e}, retry in {delay}s");
                tokio::time::sleep(Duration::from_secs(*delay)).await;
            }
        }
    }
}
```

### Pattern 3: State Transitions

```rust
pub enum ServiceState {
    Starting,
    Ready,
    Running,
    Degraded,
    Error,
    Stopping,
    Stopped,
}

impl ServiceState {
    pub fn can_accept_requests(&self) -> bool {
        matches!(self, Self::Running | Self::Degraded)
    }
    
    pub fn should_register(&self) -> bool {
        matches!(self, Self::Ready | Self::Running)
    }
}
```

---

## 🔧 Gaps Discovered

This demo reveals implementation gaps:

### Gap 1: Health Check Endpoints Missing

**What we need**:
- `GET /health` - Detailed status
- `GET /health/live` - Liveness probe
- `GET /health/ready` - Readiness probe

**Where to add**: `crates/loam-spine-api/src/jsonrpc.rs`

### Gap 2: Retry Logic Not Implemented

**What we need**:
- Exponential backoff on heartbeat failures
- Max retry attempts configuration
- Circuit breaker pattern

**Where to add**: `crates/loam-spine-core/src/service/lifecycle.rs`

### Gap 3: State Transition Logic

**What we need**:
- ServiceState enum with transitions
- Health check that determines state
- State-dependent behavior

**Where to add**: `crates/loam-spine-core/src/service/lifecycle.rs`

---

## 📊 Configuration

From `specs/SERVICE_LIFECYCLE.md`:

```toml
[lifecycle.heartbeat]
interval_seconds = 30
timeout_seconds = 90  # 3 missed cycles
auto_deregister_seconds = 180  # 6 missed cycles

[lifecycle.retry]
backoff_seconds = [10, 30, 60, 120]
max_attempts = 5

[lifecycle.health]
check_interval_seconds = 60
dependencies_required = ["storage"]
dependencies_optional = ["songbird"]
```

---

## 🔧 Troubleshooting

### Error: "Service marked as stale"
This is expected behavior! After 3 missed heartbeats, Songbird marks the service as stale but doesn't remove it yet.

### Error: "Service auto-deregistered"
After 6 missed heartbeats, Songbird removes the service. The service should automatically re-register when it recovers.

---

## ➡️ Next Steps

After completing this demo:
- **Implement**: Health check endpoints
- **Implement**: Retry logic with exponential backoff
- **Implement**: State transition logic
- **Next Level**: `../04-inter-primal/` demos
- **Deep Dive**: `../../specs/SERVICE_LIFECYCLE.md`

---

## 🎯 Success Criteria

You'll know this demo worked if you see:
- ✅ Health check endpoints explained
- ✅ Heartbeat success/failure demonstrated
- ✅ Degraded state recovery shown
- ✅ Stale detection timeline clear
- ✅ Auto-deregistration logic understood

---

**Updated**: December 25, 2025  
**Gaps Discovered**: Health endpoints, retry logic, state transitions  
**Principle**: Resilient services that recover automatically

🦴 **LoamSpine: Self-healing through monitoring**
