<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Service Lifecycle Protocol Specification

**Version**: 1.0.0  
**Status**: Draft  
**Date**: December 24, 2025

---

## 🎯 Overview

This specification defines the lifecycle protocol for coordinating LoamSpine with other primals in the ecoPrimals ecosystem. It addresses Gap #4 from the comprehensive audit: service startup/coordination patterns.

### Philosophy

- **Zero Hardcoding**: No primal names or endpoints in code
- **Runtime Discovery**: Services find each other through Songbird
- **Graceful Degradation**: Continue operation when optional services unavailable
- **Self-Healing**: Automatic reconnection after failures
- **Clear States**: Well-defined lifecycle states with clean transitions

---

## 📊 Lifecycle States

### Service States

```
┌─────────────┐
│  STARTING   │ ← Service initialization
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   READY     │ ← Core functionality available
└──────┬──────┘
       │
       ├─────────────────┐
       │                 │
       ▼                 ▼
┌─────────────┐   ┌─────────────┐
│  RUNNING    │   │  DEGRADED   │ ← Missing optional services
└──────┬──────┘   └──────┬──────┘
       │                 │
       │                 │
       ▼                 ▼
┌─────────────┐   ┌─────────────┐
│  STOPPING   │   │   ERROR     │ ← Critical failure
└──────┬──────┘   └──────┬──────┘
       │                 │
       ▼                 ▼
┌─────────────┐   ┌─────────────┐
│  STOPPED    │   │   FAILED    │
└─────────────┘   └─────────────┘
```

### State Definitions

| State | Description | Can Serve Requests? |
|-------|-------------|---------------------|
| **STARTING** | Initializing core components | No |
| **READY** | Core ready, discovering services | Limited (core only) |
| **RUNNING** | Fully operational with all services | Yes (full) |
| **DEGRADED** | Running but missing optional services | Yes (partial) |
| **STOPPING** | Graceful shutdown in progress | No (draining) |
| **STOPPED** | Cleanly stopped | No |
| **ERROR** | Temporary failure, attempting recovery | No |
| **FAILED** | Permanent failure, manual intervention needed | No |

---

## 🚀 Startup Sequence

### Phase 1: Core Initialization (STARTING → READY)

**Duration**: ~100-500ms  
**Required Services**: None  
**Can Fail**: Yes (critical)

```rust
1. Load configuration from env/file
2. Initialize storage backend (redb/InMemory)
3. Verify storage integrity
4. Initialize capability registry
5. Start RPC servers (tarpc + JSON-RPC)
6. Transition to READY
```

**Health Check Response**: `503 Service Unavailable`

### Phase 2: Service Discovery (READY → RUNNING/DEGRADED)

**Duration**: ~1-5 seconds  
**Required Services**: Songbird (optional)  
**Can Fail**: No (degrades gracefully)

```rust
1. Connect to Songbird (if configured)
   ├─ Success: Continue to step 2
   └─ Failure: Log warning, enter DEGRADED mode

2. Advertise LoamSpine capabilities
   ├─ Success: Continue to step 3
   └─ Failure: Log warning, stay in READY mode

3. Discover optional services:
   ├─ Signing service (for entry verification)
   ├─ Storage service (for payload hosting)
   └─ Other primals (for inter-primal features)

4. Start heartbeat task (30s interval)

5. Transition to:
   ├─ RUNNING: All desired services discovered
   └─ DEGRADED: Some services missing but core works
```

**Health Check Response**: 
- READY: `200 OK` (basic)
- RUNNING: `200 OK` (full)
- DEGRADED: `200 OK` (partial)

### Phase 3: Runtime Operation

**Service Availability Monitoring**:
```rust
Every 30 seconds:
1. Send heartbeat to Songbird
2. Check discovered services health
3. Attempt to discover missing services
4. Update state (RUNNING ↔ DEGRADED)
```

**Automatic Recovery**:
```rust
On service failure:
1. Log failure
2. Remove from registry
3. Transition to DEGRADED (if was RUNNING)
4. Background task: Retry discovery every 60s
5. Auto-transition back to RUNNING when recovered
```

---

## 🛑 Shutdown Sequence

### Graceful Shutdown (RUNNING/DEGRADED → STOPPING → STOPPED)

**Duration**: ~1-3 seconds  
**Trigger**: SIGTERM, SIGINT, or programmatic shutdown

```rust
1. Stop accepting new requests
2. Mark health check as unhealthy (503)
3. Drain in-flight requests (timeout: 5s)
4. Deregister from Songbird
5. Stop heartbeat task
6. Flush storage backend
7. Close RPC servers
8. Release resources
9. Transition to STOPPED
```

### Emergency Shutdown (ERROR → FAILED)

**Duration**: Immediate  
**Trigger**: Critical error, panic, or timeout

```rust
1. Log critical error
2. Attempt to flush storage (timeout: 1s)
3. Attempt to deregister from Songbird (timeout: 1s)
4. Force close all connections
5. Exit with error code
```

---

## 🔍 Health Check Protocol

### Endpoint: `GET /health`

**Response Format**:
```json
{
  "status": "running|degraded|ready|starting|stopping",
  "version": "0.9.14",
  "uptime_seconds": 3600,
  "services": {
    "storage": {
      "available": true,
      "backend": "redb",
      "healthy": true
    },
    "rpc": {
      "tarpc": {
        "available": true,
        "endpoint": "http://localhost:9001",
        "connections": 5
      },
      "jsonrpc": {
        "available": true,
        "endpoint": "http://localhost:8080",
        "connections": 12
      }
    },
    "discovery": {
      "songbird": {
        "connected": true,
        "endpoint": "http://localhost:8082",
        "last_heartbeat": "2025-12-24T10:30:00Z"
      }
    },
    "capabilities": {
      "signing": {
        "discovered": true,
        "service": "beardog",
        "healthy": true
      },
      "storage": {
        "discovered": false,
        "attempts": 5,
        "next_retry": "2025-12-24T10:31:00Z"
      }
    }
  },
  "metrics": {
    "spines": 42,
    "entries": 1337,
    "certificates": 23
  }
}
```

**HTTP Status Codes**:
- `200 OK`: RUNNING, DEGRADED, or READY
- `503 Service Unavailable`: STARTING, STOPPING, ERROR
- `500 Internal Server Error`: FAILED

### Liveness Check: `GET /health/live`

Simple check: "Is the process alive?"

```json
{"alive": true}
```

Always returns `200 OK` unless process is dead.

### Readiness Check: `GET /health/ready`

Check: "Can we serve requests?"

```json
{"ready": true}
```

Returns:
- `200 OK`: State is READY, RUNNING, or DEGRADED
- `503 Service Unavailable`: State is STARTING, STOPPING, ERROR, or FAILED

---

## 🔄 Service Discovery Protocol

### Discovery Configuration

```toml
[discovery]
# Methods to try, in order
methods = ["environment", "service-registry", "mdns", "local_binaries", "config_file"]

# Service registry connection (Songbird, Consul, etcd, or any compatible registry)
service_registry_endpoint = "http://localhost:8082"
service_registry_timeout_ms = 5000
service_registry_retry_interval_s = 60

# Advertisement
advertise_on_startup = true
heartbeat_interval_s = 30
deregister_on_shutdown = true

# Required vs optional services
required_capabilities = []  # Empty = all are optional
optional_capabilities = ["signing", "storage", "ephemeral-storage"]

# Timeouts
startup_discovery_timeout_s = 5
capability_health_check_interval_s = 60
```

### Discovery Methods

#### 1. Environment Variables (Highest Priority)

```bash
# Direct service endpoints
LOAMSPINE_SIGNING_SERVICE="http://localhost:7001"
LOAMSPINE_STORAGE_SERVICE="http://localhost:7002"

# Binary paths
LOAMSPINE_SIGNER_PATH="/path/to/beardog"
LOAMSPINE_STORAGE_PATH="/path/to/nestgate"
```

#### 2. Songbird (Primary Discovery)

```rust
1. Connect to Songbird endpoint
2. Register LoamSpine capabilities
3. Query for desired capabilities
4. Cache discovered services
5. Subscribe to updates (if supported)
```

#### 3. mDNS (Local Network)

```rust
1. Broadcast mDNS query for ecoPrimals services
2. Listen for responses
3. Validate service metadata
4. Cache discovered services
```

#### 4. Local Binaries (Development)

```rust
1. Check ../bins/ directory
2. Check $PATH
3. Execute binary with --version flag
4. If matches required primal, use it
```

#### 5. Config File (Fallback)

```toml
# loamspine.toml
[services.signing]
endpoint = "http://beardog.local:7001"
healthy = true

[services.storage]
endpoint = "http://nestgate.local:7002"
healthy = true
```

---

## ⚠️ Failure Scenarios

### Scenario 1: Songbird Unavailable on Startup

**Behavior**: Enter DEGRADED mode
```
1. Log warning: "Songbird not available at {endpoint}"
2. Transition to DEGRADED state
3. Start background retry task (every 60s)
4. Serve requests with core functionality only
5. Auto-transition to RUNNING when Songbird reconnects
```

**Impact**: Limited - core functionality still works

### Scenario 2: Service Discovered But Unhealthy

**Behavior**: Mark as unavailable, retry
```
1. Attempt connection to discovered service
2. On failure:
   - Mark service as unhealthy
   - Remove from active registry
   - Schedule retry (exponential backoff: 10s, 30s, 60s, 120s)
3. Log warning
4. Continue operation without that service
```

### Scenario 3: Service Disappears During Runtime

**Behavior**: Graceful degradation
```
1. Health check fails for service
2. Mark service as unavailable
3. Remove from active registry
4. Transition to DEGRADED (if was RUNNING)
5. Background task continues discovery attempts
6. Auto-recover when service returns
```

### Scenario 4: Cascade Failure (Multiple Services Down)

**Behavior**: Intelligent degradation
```
1. Multiple services fail simultaneously
2. Assess which are critical vs optional
3. If critical services down:
   - Transition to ERROR
   - Attempt recovery
   - If recovery fails after 3 attempts → FAILED
4. If only optional services down:
   - Transition to DEGRADED
   - Continue serving core requests
```

---

## 🔧 Configuration

### Environment Variables

```bash
# Core
LOAMSPINE_STORAGE_PATH="/var/lib/loamspine"
LOAMSPINE_CONFIG_PATH="/etc/loamspine/config.toml"

# Discovery
LOAMSPINE_SONGBIRD_ENDPOINT="http://localhost:8082"
LOAMSPINE_DISCOVERY_TIMEOUT="5"
LOAMSPINE_HEARTBEAT_INTERVAL="30"

# RPC
LOAMSPINE_TARPC_BIND="0.0.0.0:9001"
LOAMSPINE_JSONRPC_BIND="0.0.0.0:8080"

# Lifecycle
LOAMSPINE_STARTUP_TIMEOUT="30"
LOAMSPINE_SHUTDOWN_TIMEOUT="10"
LOAMSPINE_REQUIRED_SERVICES=""  # Comma-separated
```

### Configuration File

```toml
# loamspine.toml
[service]
name = "loamspine"
version = "0.9.14"

[storage]
backend = "redb"
path = "/var/lib/loamspine"

[discovery]
methods = ["service-registry", "environment"]
service_registry_endpoint = "http://localhost:8082"
startup_timeout_seconds = 5
heartbeat_interval_seconds = 30

[rpc.tarpc]
bind = "0.0.0.0:9001"

[rpc.jsonrpc]
bind = "0.0.0.0:8080"

[lifecycle]
startup_timeout_seconds = 30
shutdown_timeout_seconds = 10
drain_timeout_seconds = 5

[capabilities]
required = []
optional = ["signing", "storage"]
```

---

## 🎯 Implementation Checklist

### Core Lifecycle Manager

- [ ] Define `ServiceState` enum
- [ ] Implement state machine transitions
- [ ] Add state change event logging
- [ ] Add metrics for state duration

### Health Checks

- [ ] Implement `/health` endpoint
- [ ] Implement `/health/live` endpoint
- [ ] Implement `/health/ready` endpoint
- [ ] Add detailed service status

### Discovery Integration

- [ ] Enhance `LifecycleManager` with discovery
- [ ] Add service health monitoring
- [ ] Implement retry logic with exponential backoff
- [ ] Add auto-recovery on reconnection

### Graceful Shutdown

- [ ] Implement signal handlers (SIGTERM, SIGINT)
- [ ] Add request draining
- [ ] Add storage flush on shutdown
- [ ] Add Songbird deregistration

### Configuration

- [ ] Load from environment variables
- [ ] Load from TOML file
- [ ] Merge with sane defaults
- [ ] Validate required fields

---

## 📊 Metrics & Monitoring

### Key Metrics

```
loamspine_state{state="running|degraded|..."} 1
loamspine_uptime_seconds 3600
loamspine_state_transitions_total{from="ready",to="running"} 5

loamspine_services_discovered_total 3
loamspine_services_healthy{service="signing"} 1
loamspine_services_healthy{service="storage"} 0

loamspine_heartbeat_success_total 120
loamspine_heartbeat_failure_total 2

loamspine_requests_in_flight 12
loamspine_requests_total{status="200"} 15420
loamspine_requests_total{status="503"} 5
```

---

## 🧪 Testing Strategy

### Unit Tests

- State machine transitions
- Configuration parsing
- Health check responses
- Retry logic

### Integration Tests

- Startup with Songbird available
- Startup with Songbird unavailable
- Service discovery and reconnection
- Graceful shutdown
- Cascade failure scenarios

### E2E Tests

- Full ecosystem startup
- Service restarts during operation
- Network partitions
- Load under degraded mode

---

## 📚 References

- **Kubernetes Health Checks**: https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/
- **12-Factor App**: https://12factor.net/
- **Resilience Patterns**: Circuit breaker, retry, timeout

---

**Status**: ✅ Specification complete, ready for implementation  
**Next**: Implement in `crates/loam-spine-core/src/service/lifecycle.rs`

🦴 **LoamSpine: Clear contracts for reliable coordination**

