# 🦴 Demo: Auto-Advertise & Lifecycle Management

**Goal**: Demonstrate automatic service lifecycle with Songbird  
**Time**: 5 minutes  
**Prerequisites**: Songbird binary at `../bins/songbird-orchestrator`

---

## 🎯 What You'll Learn

- Auto-registration on service startup
- Heartbeat mechanism (keep-alive)
- Auto-deregistration on shutdown
- Graceful lifecycle management

---

## 🚀 Quick Start

```bash
# Run the demo
./demo.sh
```

---

## 📋 What This Demo Shows

### 1. Auto-Registration
When LoamSpine starts, it automatically:
1. Discovers Songbird (via env var, mDNS, or config)
2. Registers itself with capabilities
3. No manual configuration needed!

### 2. Heartbeat Mechanism
Once registered, LoamSpine sends heartbeats:
- **Frequency**: Every 30 seconds (configurable)
- **Purpose**: "I'm still alive and healthy"
- **Failure**: If heartbeat fails, retry with exponential backoff

### 3. Auto-Deregistration
When LoamSpine shuts down:
1. Receives SIGTERM signal
2. Stops accepting new requests
3. Drains in-flight requests (5s timeout)
4. Deregisters from Songbird
5. Exits gracefully

---

## 💡 Key Concepts

### Service Lifecycle States

```
STARTING → READY → RUNNING → STOPPING → STOPPED
              ↓        ↕
            ERROR   DEGRADED
```

**STARTING**: Initializing, not ready for traffic  
**READY**: Initialized, ready to register  
**RUNNING**: Registered with Songbird, accepting requests  
**DEGRADED**: Running but some capabilities unavailable  
**ERROR**: Critical failure, needs intervention  
**STOPPING**: Graceful shutdown in progress  
**STOPPED**: Clean exit

### Heartbeat Protocol

```rust
// Heartbeat sent every 30 seconds
struct Heartbeat {
    service_name: String,
    timestamp: DateTime<Utc>,
    status: HealthStatus,  // Healthy, Degraded, Error
}

// Songbird tracks last heartbeat
// If no heartbeat for 90s (3 cycles), mark as stale
// If no heartbeat for 180s (6 cycles), auto-deregister
```

### Discovery Priority

LoamSpine discovers Songbird in this order:
1. **Environment variable**: `SONGBIRD_ENDPOINT=http://...`
2. **Songbird discovery**: Query existing Songbird
3. **mDNS**: Local network broadcast
4. **Config file**: `primal-capabilities.toml`
5. **Fallback**: Operate without discovery (degraded mode)

---

## 🔍 What You'll See

```
================================================================
  🦴 LoamSpine: Auto-Advertise & Lifecycle Management
================================================================

Step 1: Checking prerequisites...
✓ Songbird binary found

Step 2: Starting Songbird orchestrator...
✓ Songbird running (PID: 12345)

Step 3: Auto-register LoamSpine on startup...
✓ LoamSpine auto-registered on startup
   Startup time: 2025-12-25T10:30:00+00:00
✓ Registration verified

Step 4: Heartbeat mechanism (keep-alive)...

   Simulating 3 heartbeat cycles
   Heartbeat 1/3...
   ✓ Heartbeat sent (10:30:02)
   Heartbeat 2/3...
   ✓ Heartbeat sent (10:30:04)
   Heartbeat 3/3...
   ✓ Heartbeat sent (10:30:06)

✓ Heartbeat mechanism working
   In production: heartbeat every 30s

Step 5: Cleanup - Auto-deregistration...
✓ LoamSpine deregistered from Songbird

================================================================
  Demo Complete!
================================================================

Gap discovered:
  ⚠️  Need to implement LifecycleManager in LoamSpine
```

---

## 🎓 Learning Points

### Pattern 1: Zero-Config Startup

```rust
// LoamSpine starts with zero configuration
// It discovers Songbird and registers itself automatically

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize
    let config = Config::discover()?;  // No hardcoding!
    
    // 2. Start lifecycle manager
    let lifecycle = LifecycleManager::new(config);
    lifecycle.start().await?;  // Auto-registers with Songbird
    
    // 3. Run service
    run_service().await?;
    
    // 4. Graceful shutdown on SIGTERM
    lifecycle.stop().await?;  // Auto-deregisters
    Ok(())
}
```

### Pattern 2: Heartbeat Loop

```rust
// Background task sends heartbeats
async fn heartbeat_loop(songbird: SongbirdClient) {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        
        match songbird.heartbeat().await {
            Ok(_) => log::debug!("Heartbeat sent"),
            Err(e) => {
                log::warn!("Heartbeat failed: {e}");
                // Retry with exponential backoff
                retry_with_backoff().await;
            }
        }
    }
}
```

### Pattern 3: Graceful Shutdown

```rust
// SIGTERM handler
async fn handle_shutdown(lifecycle: &LifecycleManager) {
    // 1. Stop accepting new requests
    lifecycle.stop_accepting_requests().await;
    
    // 2. Drain in-flight requests (5s timeout)
    lifecycle.drain_requests(Duration::from_secs(5)).await;
    
    // 3. Deregister from Songbird
    lifecycle.deregister().await.ok();
    
    // 4. Flush storage
    lifecycle.flush_storage().await.ok();
    
    // 5. Clean exit
    log::info!("Graceful shutdown complete");
}
```

---

## 🔧 Gap Discovered

This demo reveals a gap in our implementation:

### Gap: LifecycleManager Not Fully Implemented

**Current State**:
- ✅ `LifecycleManager` struct exists
- ✅ Basic registration methods exist
- ⚠️ Auto-registration not implemented
- ⚠️ Heartbeat loop not implemented
- ⚠️ SIGTERM handler not implemented
- ⚠️ Graceful shutdown not complete

**What We Need**:
1. Auto-registration on startup
2. Background heartbeat task
3. SIGTERM signal handler
4. Graceful shutdown sequence
5. Retry logic for failed operations

**Specification**: Already complete in `specs/SERVICE_LIFECYCLE.md`!

**Priority**: HIGH (Gap #4 from INTEGRATION_GAPS.md)

---

## 📊 Heartbeat Timing

**Configuration** (from specs/SERVICE_LIFECYCLE.md):
```toml
[lifecycle]
heartbeat_interval = 30  # seconds
heartbeat_timeout = 90   # 3 missed heartbeats
auto_deregister = 180    # 6 missed heartbeats
retry_backoff = [10, 30, 60, 120]  # exponential backoff
```

**State Transitions**:
- `t=0`: Service starts, registers
- `t=30`: First heartbeat
- `t=60`: Second heartbeat
- `t=90`: If no heartbeat, mark as DEGRADED
- `t=180`: If still no heartbeat, auto-deregister

---

## 🔧 Troubleshooting

### Error: "Heartbeat failed"
```bash
# Check Songbird is running
curl http://localhost:8082/health

# Check network connectivity
ping localhost

# Check Songbird logs
cat /tmp/songbird.log
```

### Error: "Registration failed"
```bash
# Verify Songbird accepts registrations
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"name":"test","endpoint":"http://localhost:9999","capabilities":[]}' \
  http://localhost:8082/api/v1/register
```

---

## ➡️ Next Steps

After completing this demo:
- **Next Demo**: `04-heartbeat-monitoring` - Health check implementation
- **Implement**: LifecycleManager auto-advertise feature
- **Deep Dive**: `../../specs/SERVICE_LIFECYCLE.md`
- **Related**: Gap #4 in `INTEGRATION_GAPS.md`

---

## 🎯 Success Criteria

You'll know this demo worked if you see:
- ✅ LoamSpine auto-registers on startup
- ✅ Heartbeats sent successfully
- ✅ Registration persists during heartbeats
- ✅ Auto-deregistration on shutdown

---

**Updated**: December 25, 2025  
**Gap Discovered**: LifecycleManager needs auto-advertise implementation  
**Principle**: Zero-config service lifecycle

🦴 **LoamSpine: Services that manage themselves**
