# loamSpine — Wave 78 Parity: Capability Registry (June 5, 2026)

**From**: loamSpine (strandGate)  
**To**: primalSpring (eastGate), biomeOS  
**Wave**: 78  
**Status**: COMPLETE — parity item RESOLVED

---

## Mission Response

primalSpring Wave 78 parity audit: "loamSpine — P2 — Create `config/capability_registry.toml`"

## What Shipped

### `config/capability_registry.toml`

Machine-readable biomeOS-compatible capability overlay following the
sweetGrass/biomeOS/petalTongue convention.

| Section | Count | Content |
|---------|-------|---------|
| `[primal]` | 1 | Identity, version, protocol, transport |
| `[domains.*]` | 19 | All exposed capability domains |
| `[operations.*]` | 47 | Every JSON-RPC method with `depends_on`, `cost`, `stability` |
| `[consumed.*]` | 6 | Runtime-discovered external capabilities |

Cross-validated: TOML operations 1:1 match `niche.rs` METHODS (47/47).

### Stability tiers in registry

- **stable** (43 operations): spine, entry, certificate, proof, waypoint, anchor, session, braid, bonding, trust, btsp, health, lifecycle, auth, capabilities, identity, tools, primal
- **compat** (4 operations): permanence.* legacy naming layer

### Deep debt (maintenance pass)

- README tree structure updated with `config/` directory
- STATUS.md compliance table includes `capability_registry.toml` PASS
- Zero warnings, zero TODO/FIXME, zero `#[allow(`

## Ecosystem Position

loamSpine now matches the Wave 78 ecosystem standard:
- [x] Zero clippy (pedantic + nursery)
- [x] Zero `#[allow]` in production
- [x] `capability_registry.toml` (machine-readable, TOML)
- [x] BTSP Phase 3
- [x] Wire Standard L2+
- [x] MethodGate pre-dispatch
- [x] plasmidBin ecoBin compliant
- [x] `forbid(unsafe_code)`
- [x] 90.9% line coverage

## Metrics

| Metric | Value |
|--------|-------|
| Tests | 1,600 |
| Source files | 198 |
| Methods | 47 |
| Domains | 19 |
| Coverage | 90.9% |
| Warnings | 0 |

---

*Holding steady per primalSpring directive. bearDog FRAGO (`wave76c-beardog-auth-events-subscribe`) in flight on southGate.*
