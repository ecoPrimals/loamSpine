# loamSpine — Wave 47: Deployment Behavioral Convergence

**Date:** 2026-05-24
**Audit:** Wave 47 — Post-Primordial Behavioral Convergence
**License:** AGPL-3.0-or-later

---

## CRITICAL Blocker Resolution

The Wave 47 audit reported: "Tokio double-runtime crash when started by
nucleus_launcher.sh — blocks southGate."

**Root cause analysis**: This was a **misdiagnosis**. The actual failure is:

```
infra/plasmidBin/start_primal.sh passes "serve" subcommand
loamSpine binary only accepts "server" → immediate CLI parse error
```

The original LS-03 nested-runtime bug (v0.9.15) was `Runtime::new().block_on()`
inside async `infant_discovery.rs` — fixed by replacing with `tokio::spawn`.
Further hardened in v0.9.16 when `mdns` 3.0 (async-std) was replaced with
`mdns-sd` 0.19 (pure tokio). **Zero `Runtime::new()` or `block_on()` in
production code** as of v0.9.16+.

### Fix

`start_primal.sh` line 228: `ARGS+=(serve)` → `ARGS+=(server)`.

Same fix applies to sweetGrass and rhizoCrypt in the same case block.

---

## Deployment Surface Compliance

| Check | Status | Detail |
|-------|--------|--------|
| `--socket PATH` CLI flag | **PASS** | `#[arg(long)] socket: Option<String>` + `LOAMSPINE_SOCKET` env |
| `health.liveness` → `{"status":"alive"}` | **PASS** | `LivenessProbe { status: "alive" }` |
| SIGTERM graceful shutdown | **PASS** | `SignalHandler` with `SignalKind::terminate()` + `SignalKind::interrupt()` |
| `lifecycle.status` | **PASS** | Now includes `uptime_s` per standard |
| UDS-first, TCP opt-in | **PASS** | No TCP without `--port` or env var |
| Socket cleanup | **PASS** | Stale socket `unlink` before bind, PID sidecar |
| `primal.announce` on startup | **PASS** | Wave 43 schema (signal_tiers, cost_hints, etc.) |

## Additional Fixes

| Item | Change |
|------|--------|
| `LOAMSPINE_DISCOVERY_ENABLED` | New env var. Set `0`/`false`/`no` to disable infant discovery. NUCLEUS can skip DNS/mDNS probes. |
| `lifecycle.status` `uptime_s` | Added elapsed seconds field per DEPLOYMENT_BEHAVIOR_STANDARD |

---

## Unaffected by This Wave (Already Compliant)

- `server` subcommand (UniBin pattern)
- PID file at `<socket>.pid`
- `capabilities.list` with Wire Standard L3
- `identity.get` canonical response
- JH-0 method gate (Public/Protected classification)

---

## Files Changed

### loamSpine
- `crates/loam-spine-core/src/config.rs` — `LOAMSPINE_DISCOVERY_ENABLED` env gate
- `crates/loam-spine-api/src/jsonrpc/mod.rs` — `uptime_s` in lifecycle.status, `Instant` tracking

### plasmidBin
- `start_primal.sh` — `serve` → `server` for loamspine/sweetgrass/rhizocrypt

---

## Upstream Action Items

| Consumer | Action |
|----------|--------|
| primalSpring | Remove loamSpine from CRITICAL blockers list — resolved |
| plasmidBin | Rebuild with `start_primal.sh` fix (serve → server) |
| southGate | loamSpine should now start in NUCLEUS compositions |
| sweetGrass/rhizoCrypt | Verify their binaries also accept `server` subcommand |

---

## 1,527 tests passing. Zero warnings. Zero compilation errors.
