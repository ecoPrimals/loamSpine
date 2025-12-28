# 🦴 Phase 2: RPC API & Service Integration

**Status**: Complete ✅  
**Real Service**: YES (using loamspine-service binary)  
**Mocks**: ZERO

---

## Overview

This phase demonstrates **LoamSpine as a production service** using the real `loamspine-service` binary. All demos use actual running services, not mocks or examples.

**Key Features**:
- **Dual Protocol**: tarpc (binary) + JSON-RPC 2.0 (text)
- **Language-Agnostic**: Any HTTP client can use JSON-RPC
- **Production-Ready**: Health checks, monitoring, lifecycle management
- **19 RPC Methods**: Complete API surface

---

## 🎯 Demos

### 02-jsonrpc-basics/ ✅
**Real Service JSON-RPC Integration**

Demonstrates:
- Starting real `loamspine-service` binary
- JSON-RPC 2.0 protocol (standard, language-agnostic)
- Creating spines via HTTP POST
- Appending entries
- Retrieving spine information
- Verifying integrity

**Use Case**: External clients (Python, JavaScript, CLI tools)

**Run**:
```bash
cd 02-jsonrpc-basics
./demo.sh
```

**What It Shows**:
- HTTP-based API access
- Standard JSON-RPC 2.0 format
- Same API as tarpc (19 methods)
- curl-based interaction examples

---

### 03-health-monitoring/ ✅
**Production Health Monitoring**

Demonstrates:
- Health check endpoints (`/health`)
- Service stability under load
- Continuous monitoring patterns
- Performance metrics (CPU, memory)
- Graceful shutdown verification

**Use Case**: Production monitoring, alerting, uptime tracking

**Run**:
```bash
cd 03-health-monitoring
./demo.sh
```

**What It Shows**:
- Health polling strategies
- Load testing
- Metrics collection
- Integration with monitoring tools (Prometheus, Grafana)

---

### 06-service-lifecycle/ ✅
**Complete Service Management**

Demonstrates:
- Pre-start checks (port availability)
- Service startup with configuration
- Health verification
- Runtime operations
- Monitoring and log analysis
- Graceful shutdown
- Cleanup procedures

**Use Case**: Production deployment, DevOps, service management

**Run**:
```bash
cd 06-service-lifecycle
./demo.sh
```

**What It Shows**:
- Complete lifecycle: start → monitor → shutdown
- Best practices for production
- Integration with systemd, Docker, Kubernetes
- Log management and debugging

---

## 🏗️ Architecture

### Dual Protocol Design

```
┌─────────────────────────────────────┐
│      LoamSpine Service              │
│                                     │
│  ┌────────────┐   ┌──────────────┐ │
│  │   tarpc    │   │  JSON-RPC    │ │
│  │  Port 9001 │   │  Port 8080   │ │
│  └────────────┘   └──────────────┘ │
│         │                 │         │
└─────────┼─────────────────┼─────────┘
          │                 │
          ▼                 ▼
   ┌──────────┐      ┌────────────┐
   │  Primals │      │  External  │
   │  (Rust)  │      │  Clients   │
   └──────────┘      └────────────┘
     Binary RPC      HTTP/JSON-RPC
     High perf       Lang-agnostic
```

### Why Two Protocols?

**tarpc (Binary)**:
- For primal-to-primal communication
- Pure Rust, type-safe
- Zero-copy, high performance
- Native async/await

**JSON-RPC 2.0 (Text)**:
- For external clients
- Language-agnostic (Python, JS, Go, etc.)
- Standard protocol (tooling support)
- Human-readable debugging

**Same API, Different Transports!**

---

## 📡 API Methods (19)

### Spine Operations
- `create_spine` - Create new spine
- `get_spine` - Retrieve spine info
- `verify_spine` - Verify integrity
- `seal_spine` - Seal (make immutable)
- `list_spines` - List all spines

### Entry Operations
- `append_entry` - Add entry to spine
- `get_entry` - Retrieve entry by hash
- `get_entries` - Get entries by range

### Certificate Operations
- `create_certificate` - Issue certificate
- `get_certificate` - Retrieve certificate
- `verify_certificate` - Verify certificate
- `revoke_certificate` - Revoke certificate

### Proof Operations
- `create_proof_inclusion` - Prove entry inclusion
- `create_proof_exclusion` - Prove entry exclusion
- `verify_proof` - Verify proof

### Discovery & Health
- `discover_capabilities` - List capabilities
- `get_primal_info` - Get primal metadata
- `health_check` - Service health status
- `get_metrics` - Service metrics

---

## 🔧 Usage Examples

### Python Client
```python
import requests

# Create spine
response = requests.post('http://localhost:8080', json={
    "jsonrpc": "2.0",
    "id": 1,
    "method": "create_spine",
    "params": {
        "owner_did": "did:key:z6MkPythonClient",
        "name": "My Python Spine"
    }
})

spine_id = response.json()['result']['spine_id']
print(f"Created spine: {spine_id}")
```

### JavaScript Client
```javascript
const response = await fetch('http://localhost:8080', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
        jsonrpc: '2.0',
        id: 1,
        method: 'create_spine',
        params: {
            owner_did: 'did:key:z6MkJSClient',
            name: 'My JS Spine'
        }
    })
});

const { result } = await response.json();
console.log('Created spine:', result.spine_id);
```

### curl (Shell)
```bash
curl -X POST http://localhost:8080 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "create_spine",
    "params": {
      "owner_did": "did:key:z6MkCurlUser",
      "name": "My Curl Spine"
    }
  }'
```

---

## 🎓 Production Patterns

### Health Monitoring
```bash
# Poll health every 30 seconds
while true; do
    STATUS=$(curl -s http://localhost:8080/health)
    echo "[$(date)] Health: ${STATUS}"
    sleep 30
done
```

### Load Balancing
```nginx
upstream loamspine {
    server localhost:8080;
    server localhost:8081;
    server localhost:8082;
}

server {
    location / {
        proxy_pass http://loamspine;
        proxy_next_upstream error timeout http_502;
    }
}
```

### systemd Service
```ini
[Unit]
Description=LoamSpine Service
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/loamspine-service --jsonrpc-port 8080
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### Docker Compose
```yaml
version: '3'
services:
  loamspine:
    image: loamspine:latest
    ports:
      - "8080:8080"
      - "9001:9001"
    environment:
      - JSONRPC_PORT=8080
      - TARPC_PORT=9001
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

---

## 📊 Monitoring Integration

### Prometheus (Metrics)
```yaml
scrape_configs:
  - job_name: 'loamspine'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

### Grafana (Dashboards)
- Request rate
- Response time (p50, p95, p99)
- Error rate
- Spine creation rate
- Memory/CPU usage

### Alerting Rules
```yaml
groups:
  - name: loamspine
    rules:
      - alert: LoamSpineDown
        expr: up{job="loamspine"} == 0
        for: 2m
        annotations:
          summary: "LoamSpine service is down"
          
      - alert: HighMemory
        expr: process_resident_memory_bytes > 1e9
        for: 5m
        annotations:
          summary: "LoamSpine using >1GB memory"
```

---

## 🎯 What This Phase Demonstrates

✅ **Production Service**: Real binary, not examples  
✅ **Dual Protocol**: tarpc + JSON-RPC 2.0  
✅ **Language-Agnostic**: Any HTTP client can use it  
✅ **Health Monitoring**: Production-ready health checks  
✅ **Service Management**: Complete lifecycle handling  
✅ **Best Practices**: Logging, metrics, graceful shutdown  
✅ **Integration Ready**: Works with monitoring tools  

---

## 🚀 Quick Start

Run all Phase 2 demos:
```bash
# Individual demos
cd 02-jsonrpc-basics && ./demo.sh
cd 03-health-monitoring && ./demo.sh
cd 06-service-lifecycle && ./demo.sh

# Or run all at once (TODO: add RUN_ALL.sh)
```

---

## 💡 Key Takeaways

1. **LoamSpine is a service**, not just a library
2. **Dual protocol** enables both performance (tarpc) and accessibility (JSON-RPC)
3. **Language-agnostic** JSON-RPC opens ecosystem to any language
4. **Production-ready** with health checks and monitoring
5. **Complete lifecycle** management for DevOps/SRE teams

---

## 🔗 Related

- **Phase 1**: Local primal capabilities
- **Phase 3**: Real inter-primal integration
- **API Docs**: `crates/loam-spine-api/README.md`
- **Service Binary**: `ecoPrimals/primalBins/loamspine-service`

---

**🦴 LoamSpine: Production-ready service with language-agnostic access!**
