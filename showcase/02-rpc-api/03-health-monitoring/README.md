# 🏥 Health Monitoring - Service Observability

**Time**: 5 minutes  
**Difficulty**: Beginner  
**Prerequisites**: Basic RPC understanding

---

## 🎯 What You'll Learn

- Check service health status
- Monitor readiness and liveness
- Get performance metrics
- Understand health check patterns

---

## 📖 Concepts

### Health Check Types

**Liveness Probe**:
- Is the service alive?
- Can it respond to requests?
- Use: Kill & restart if failing

**Readiness Probe**:
- Is the service ready to accept traffic?
- Are dependencies available?
- Use: Remove from load balancer if not ready

---

## 🔍 Demo Flow

```
1. Start LoamSpine service
   ↓
2. Check liveness (basic health)
   ↓
3. Check readiness (with dependencies)
   ↓
4. Get detailed metrics
   ↓
5. Simulate failure scenarios
```

---

## 💡 Health Check Response

```json
{
  "status": "healthy",
  "uptime_seconds": 3600,
  "spines_count": 42,
  "total_entries": 1337,
  "storage_backend": "Redb",
  "discovery_connected": true,
  "last_heartbeat": "2025-12-24T12:00:00Z"
}
```

---

## 📊 Monitoring Patterns

### Kubernetes/Docker
```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 5
```

### Prometheus Metrics
- Request latency histograms
- Operation counters
- Error rates
- Storage size gauges

---

## 🎯 Production Usage

**Alert On**:
- ❌ Liveness failing > 30s
- ❌ Readiness failing > 60s
- ❌ Error rate > 1%
- ❌ Latency p99 > 100ms

**Don't Alert On**:
- ✅ Temporary readiness blips
- ✅ Startup initialization
- ✅ Graceful shutdown

---

**Status**: ⏳ Example needed  
**Related**: `crates/loam-spine-core/src/service/lifecycle.rs`

