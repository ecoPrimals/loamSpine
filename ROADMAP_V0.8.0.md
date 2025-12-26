# 🦴 LoamSpine — Roadmap to v0.8.0

**Current Version**: v0.7.0-dev (Infant Discovery Complete)  
**Target Version**: v0.8.0  
**Status**: Planning Phase  
**Estimated Effort**: 2-3 weeks

---

## ✅ v0.7.0-dev STATUS (COMPLETE)

### Achievements
- ✅ Infant discovery module (350+ LOC)
- ✅ Environment variable discovery
- ✅ Development fallback
- ✅ Graceful degradation
- ✅ 30% hardcoding reduction
- ✅ 372 tests passing (90.39% coverage)
- ✅ Zero unsafe code
- ✅ Production ready

### Philosophy Realized
**"Start with zero knowledge, discover everything at runtime"** ✅

---

## 🎯 v0.8.0 OBJECTIVES

### Primary Goal
**Complete Production Discovery Stack** — Implement DNS SRV and mDNS discovery methods

### Success Criteria
1. ✅ DNS SRV discovery functional
2. ✅ mDNS discovery functional
3. ✅ Full discovery chain operational (Env → DNS SRV → mDNS → Fallback)
4. ✅ Zero-configuration local network discovery
5. ✅ Production-ready service discovery
6. ✅ Test coverage maintained (>90%)
7. ✅ Zero unsafe code maintained
8. ✅ Backward compatible (100%)

---

## 📋 WORK BREAKDOWN

### Phase 1: DNS SRV Discovery (5-7 days)

#### Dependencies
- `trust-dns-resolver` crate (or `hickory-dns`)
- Standard DNS infrastructure

#### Implementation Tasks
1. **Add DNS Resolver** (1 day)
   - Add `trust-dns-resolver` to `Cargo.toml`
   - Create DNS resolver configuration
   - Handle DNS timeout and errors gracefully

2. **Implement SRV Lookup** (2 days)
   - Query `_discovery._tcp.local` SRV records
   - Parse SRV response (priority, weight, port, target)
   - Sort by priority and weight
   - Extract endpoint (protocol + host + port)

3. **Configuration** (1 day)
   - Add DNS configuration options
   - DNS server override (for testing)
   - Timeout and retry configuration
   - Domain suffix configuration

4. **Testing** (2-3 days)
   - Unit tests for SRV parsing
   - Integration tests with mock DNS server
   - Real DNS testing (optional)
   - Error handling tests (timeout, NXDOMAIN, etc.)

#### Files to Modify
- `crates/loam-spine-core/Cargo.toml` — Add dependencies
- `crates/loam-spine-core/src/service/infant_discovery.rs` — Implement `try_dns_srv_discovery`
- `crates/loam-spine-core/src/config.rs` — Add DNS config options
- `crates/loam-spine-core/tests/infant_discovery.rs` — Add DNS SRV tests

#### Example Implementation
```rust
async fn try_dns_srv_discovery(&self) -> Option<String> {
    use trust_dns_resolver::TokioAsyncResolver;
    use trust_dns_resolver::config::*;
    
    tracing::debug!("🔍 Attempting DNS SRV discovery (_discovery._tcp.local)...");
    
    // Create resolver
    let resolver = TokioAsyncResolver::tokio(
        ResolverConfig::default(),
        ResolverOpts::default()
    ).ok()?;
    
    // Query SRV record
    match resolver.srv_lookup("_discovery._tcp.local").await {
        Ok(response) => {
            // Sort by priority, then weight
            let mut records: Vec<_> = response.iter().collect();
            records.sort_by_key(|r| (r.priority(), std::cmp::Reverse(r.weight())));
            
            if let Some(record) = records.first() {
                let endpoint = format!("http://{}:{}", record.target(), record.port());
                tracing::info!("✅ Found discovery service via DNS SRV: {}", endpoint);
                return Some(endpoint);
            }
        }
        Err(e) => {
            tracing::debug!("🔍 DNS SRV query failed: {}, trying next method", e);
        }
    }
    
    None
}
```

---

### Phase 2: mDNS Discovery (5-7 days)

#### Dependencies
- `mdns` crate (or `libmdns`)
- Multicast networking support

#### Implementation Tasks
1. **Add mDNS Library** (1 day)
   - Add `mdns` crate to `Cargo.toml`
   - Handle platform differences (Linux, macOS, Windows)
   - Configure multicast address (224.0.0.251:5353)

2. **Implement mDNS Discovery** (2 days)
   - Broadcast query for `_discovery._tcp.local`
   - Listen for responses (timeout: 5s)
   - Parse TXT records for metadata
   - Extract endpoint from response

3. **Configuration** (1 day)
   - mDNS timeout configuration
   - Network interface selection
   - Multicast group configuration

4. **Testing** (2-3 days)
   - Unit tests for mDNS parsing
   - Integration tests with mock mDNS responder
   - Local network testing
   - Error handling tests

#### Files to Modify
- `crates/loam-spine-core/Cargo.toml` — Add dependencies
- `crates/loam-spine-core/src/service/infant_discovery.rs` — Implement `try_mdns_discovery`
- `crates/loam-spine-core/src/config.rs` — Add mDNS config options
- `crates/loam-spine-core/tests/infant_discovery.rs` — Add mDNS tests

#### Example Implementation
```rust
async fn try_mdns_discovery(&self) -> Option<String> {
    use mdns::{Record, RecordKind};
    use std::time::Duration;
    
    tracing::debug!("🔍 Attempting mDNS discovery (local network)...");
    
    // Query for service
    let service_name = "_discovery._tcp.local";
    let timeout = Duration::from_secs(5);
    
    match tokio::time::timeout(timeout, async {
        let stream = mdns::discover::all(service_name, timeout).ok()?;
        let responses: Vec<_> = stream.listen().collect().await;
        
        for response in responses {
            if let Some(Record { kind: RecordKind::SRV { port, target, .. }, .. }) = response.records().next() {
                let endpoint = format!("http://{}:{}", target, port);
                tracing::info!("✅ Found discovery service via mDNS: {}", endpoint);
                return Some(endpoint);
            }
        }
        None
    }).await {
        Ok(Some(endpoint)) => Some(endpoint),
        Ok(None) => {
            tracing::debug!("🔍 No mDNS responses, trying next method");
            None
        }
        Err(_) => {
            tracing::debug!("🔍 mDNS discovery timeout, trying next method");
            None
        }
    }
}
```

---

### Phase 3: Integration & Testing (3-5 days)

#### Tasks
1. **End-to-End Testing** (2 days)
   - Test full discovery chain
   - Test priority/fallback logic
   - Test with multiple discovery services
   - Test failure scenarios

2. **Documentation** (1 day)
   - Update `infant_discovery.rs` documentation
   - Add discovery configuration guide
   - Update integration specifications
   - Add troubleshooting guide

3. **Performance Testing** (1 day)
   - Measure discovery latency
   - Test DNS timeout behavior
   - Test mDNS timeout behavior
   - Optimize critical paths

4. **Production Testing** (1 day)
   - Deploy to staging environment
   - Test with real DNS infrastructure
   - Test on local network with mDNS
   - Verify graceful degradation

---

## 📊 EFFORT ESTIMATION

| Phase | Days | Effort |
|-------|------|--------|
| **Phase 1: DNS SRV** | 5-7 | Medium |
| **Phase 2: mDNS** | 5-7 | Medium |
| **Phase 3: Integration** | 3-5 | Low-Medium |
| **Total** | **13-19 days** | **2-3 weeks** |

---

## 🎯 MILESTONES

### Milestone 1: DNS SRV Complete (Week 1)
- ✅ DNS SRV discovery functional
- ✅ Tests passing
- ✅ Documentation updated

### Milestone 2: mDNS Complete (Week 2)
- ✅ mDNS discovery functional
- ✅ Tests passing
- ✅ Documentation updated

### Milestone 3: v0.8.0 Release (Week 3)
- ✅ All discovery methods functional
- ✅ Full test suite passing
- ✅ Production deployment successful
- ✅ Release notes complete

---

## 🚀 ACCEPTANCE CRITERIA

### Functional Requirements
- [ ] DNS SRV discovery queries `_discovery._tcp.local`
- [ ] mDNS discovery broadcasts on local network
- [ ] Discovery chain operates in priority order
- [ ] Graceful fallback on method failure
- [ ] Configurable timeouts and retries
- [ ] Environment variable override always highest priority

### Quality Requirements
- [ ] Test coverage >90% maintained
- [ ] Zero unsafe code
- [ ] Zero clippy warnings
- [ ] All 372+ tests passing
- [ ] Backward compatible (100%)
- [ ] Documentation complete

### Performance Requirements
- [ ] DNS SRV lookup < 2s
- [ ] mDNS discovery < 5s
- [ ] Total discovery time < 10s
- [ ] No blocking on main thread

---

## 🔧 TECHNICAL DECISIONS

### DNS SRV
**Choice**: `trust-dns-resolver` (now `hickory-dns`)
- **Pros**: Pure Rust, async, well-maintained, feature-rich
- **Cons**: Larger dependency
- **Alternative**: `simple-dns` (lighter, but less features)

### mDNS
**Choice**: `mdns` crate
- **Pros**: Simple API, cross-platform
- **Cons**: Less maintained, potential platform issues
- **Alternative**: `libmdns` (more features, harder to use)

### Timeout Strategy
- DNS SRV: 2s timeout
- mDNS: 5s timeout
- Development fallback: instant

---

## 📈 METRICS TARGETS (v0.8.0)

| Metric | v0.7.0 | v0.8.0 Target |
|--------|--------|---------------|
| Tests | 372 | 390+ |
| Coverage | 90.39% | >90% |
| Discovery Methods | 2 | 4 |
| Zero Unsafe | 0 | 0 |
| Production Ready | ✅ | ✅ |
| Zero-Config Local | ❌ | ✅ |

---

## 🎯 STRATEGIC VALUE

### v0.8.0 Enables
1. **Production Deployment** — Full DNS SRV support
2. **Zero-Config Local Dev** — mDNS auto-discovery
3. **Multi-Environment Support** — Dev, staging, prod
4. **Kubernetes Native** — DNS SRV standard in K8s
5. **LAN Deployment** — mDNS for local networks

---

## 🔄 NEXT AFTER v0.8.0

### v0.9.0 (Future)
- Enhanced capability registry
- Dynamic service registration
- Service health monitoring
- Advanced failure recovery

### v1.0.0 (Future)
- Network federation
- Zero-copy RPC migration
- Production metrics
- Advanced observability

---

## 📚 REFERENCES

### DNS SRV
- RFC 2782: DNS SRV Records
- Kubernetes DNS-Based Service Discovery
- `trust-dns-resolver` documentation

### mDNS
- RFC 6762: Multicast DNS
- Apple Bonjour specification
- `mdns` crate documentation

---

**Status**: Ready to begin Phase 1 (DNS SRV)  
**Estimated Start**: Upon approval  
**Estimated Completion**: 2-3 weeks from start

🦴 **LoamSpine: Evolving toward complete production discovery**

