# LoamSpine — JH-0 Method Gate Adoption

**Date:** May 8, 2026
**From:** loamSpine team
**To:** primalSpring, biomeOS, all composition consumers
**Ref:** `wateringHole/METHOD_GATE_STANDARD.md` — JH-0 adoption wave

---

## Status: ADOPTED (8/13 primals)

LoamSpine now enforces JH-0 pre-dispatch method gating on all JSON-RPC
methods across both TCP and UDS transports.

## Implementation

### Method Classification

**Public** (always allowed, regardless of auth mode):
- `health.check`, `health.liveness`, `health.readiness`
- `identity.get`
- `capabilities.list`
- `tools.list`
- `auth.check`, `auth.mode`, `auth.peer_info`

**Protected** (blocked with `-32001` in enforced mode):
- `spine.*` — `create`, `get`, `seal`
- `entry.*` — `append`, `get`, `get_tip`
- `certificate.*` — `mint`, `transfer`, `loan`, `return`, `get`
- `session.commit`, `braid.commit`
- `slice.*`, `proof.*`, `anchor.*`
- `bonding.ledger.*`
- `btsp.negotiate`
- `permanence.*`
- `tools.call`

### Auth Methods (3 new)

| Method | Params | Returns |
|--------|--------|---------|
| `auth.check` | `{ "method": "spine.create" }` | `{ method, access, allowed, mode }` |
| `auth.mode` | none | `{ mode, public_prefixes, public_methods }` |
| `auth.peer_info` | none | `{ authenticated, peer_id, transport }` |

### Error Codes

| Code | Meaning |
|------|---------|
| `-32001` | Unauthorized — method requires authentication |
| `-32000` | Auth error (reserved for future token/credential errors) |

### Environment Variable

```bash
# Default: permissive (all methods allowed)
LOAMSPINE_AUTH_MODE=permissive

# Protected methods return -32001 unless authenticated
LOAMSPINE_AUTH_MODE=enforced
```

### Wire Placement

The gate check runs **after** method normalization and **before** handler
dispatch. This means legacy aliases (`commit.session`, `capability.list`,
etc.) are normalized to canonical names before classification.

```
request → normalize_method → gate.check → dispatch_inner
```

### Architecture

- `method_gate.rs` — `MethodGate` struct, `AuthMode` enum, `classify_method()`
- `dispatch_auth()` — extracted auth method handler (keeps `dispatch_inner` under 100 LOC)
- `MethodGate::from_env()` — reads `LOAMSPINE_AUTH_MODE` at startup
- `MethodGate` uses `AtomicU8` for lock-free mode reads across connections

### Test Coverage

- 11 unit tests in `method_gate.rs` (classification, gate logic, mode roundtrip)
- 7 integration tests in `tests.rs` (auth methods through full JSON-RPC dispatch, enforced gate blocking)
- Total: 1,522 tests (was 1,504)

---

## Future Evolution

- **BTSP-authenticated peers**: `auth.peer_info` will return `authenticated: true`
  with the peer's session ID and DID when the connection was established via BTSP.
- **Per-peer authorization**: Protected methods could be allowed for BTSP-authenticated
  peers even in enforced mode, enabling fine-grained access control.
- **Role-based gate**: Extend `AuthMode` with `role_based` mode where classification
  depends on the peer's identity (e.g., Tower peers get full access, leaf peers get read-only).
