# loamSpine benchScale Roundtrip Validation Handoff

**Date**: May 25, 2026  
**Type**: Infrastructure — Deployment Validation  
**Standard**: DEPLOYMENT_VALIDATION_STANDARD v1.1

---

## Summary

Created `infra/benchScale/validate_roundtrip.sh` — a 19-phase roundtrip
validation harness that starts a live loamSpine TCP server and exercises
all 43 canonical JSON-RPC methods via HTTP POST.

## What Was Built

### `infra/benchScale/validate_roundtrip.sh`

- Builds loamSpine (or skips with `SKIP_BUILD=1`)
- Starts TCP JSON-RPC server on `127.0.0.1:19710`
- Runs **51 validations** across **19 phases**
- Validates every response with `jq` assertions
- Reports pass/fail with color-coded output
- Cleans up server on exit (trap)

### Validation Coverage

| Phase | Methods | What It Validates |
|-------|---------|-------------------|
| 1 | `health.*` | Health triad (liveness, readiness, check) |
| 2 | `capabilities.list`, `identity.get`, `lifecycle.status` | Meta/discovery |
| 3 | `auth.*` | JH-0 method gate |
| 4 | `btsp.*` | BTSP cipher negotiation |
| 5 | `spine.*` | Spine CRUD lifecycle |
| 6 | `entry.*` | Entry append, get, tip, list |
| 7 | `session.commit`, `braid.commit`, `provenance.commit` | Provenance integration + alias |
| 8 | `certificate.*` | Certificate mint + get |
| 9 | `slice.*` | Slice anchor + checkout |
| 10 | `proof.*` | Inclusion proof generate + verify |
| 11 | `anchor.*` | Bitcoin, Ethereum, RFC 3161 TSA (publish + verify + batch) |
| 12 | `bonding.ledger.*` | Bond ledger store/retrieve/list |
| 13 | `permanence.*` | Legacy compat layer roundtrip |
| 14 | `tools.*` | MCP tool discovery + invocation |
| 15 | aliases | Method normalization roundtrip |
| 16 | `spine.seal` | Seal + sealed-spine rejection |
| 17 | error paths | Unknown method (-32601), not-found spine |
| 18 | `lifecycle.status` | Uptime verification (uptime_s > 0) |
| 19 | `primal.announce` | Self-registration payload |

## Metrics

- **51 validations, 0 failures**
- **43 canonical methods** covered
- **Transport**: HTTP POST JSON-RPC 2.0 (curl-based)
- **Runtime**: ~4s (including server startup)
- **Dependencies**: curl, jq, bash 4+

## Files Changed

| File | Change |
|------|--------|
| `infra/benchScale/validate_roundtrip.sh` | Created (19-phase validation harness) |
| `infra/benchScale/README.md` | Created (usage, phase matrix, env vars) |
| `STATUS.md` | Added benchScale v0.9.16 entry |
| `WHATS_NEXT.md` | Added May 25 changelog entry |
| `README.md` | Test count 1,527→1,528, added benchScale to Quick Start |
| `CONTEXT.md` | Test count 1,527→1,528 |
| `CONTRIBUTING.md` | Test count 1,527→1,528 |
| `sporeprint/validation-summary.md` | Test count 1,527→1,528, added benchScale to evolution table |

## Relationship to Canonical benchScale

The canonical benchScale substrate (`sort-after/benchScale/`) provides
multi-node Docker/libvirt labs for composition testing. This local harness
validates a **single loamSpine instance** for pre-deploy gating — the same
checks that `benchscale validate ipc 127.0.0.1:<port>` runs, plus full
method coverage.

## Upstream Audit Request

For primalSpring:
- All 43 methods validated via live TCP roundtrip (0 failures)
- 1,528 tests passing, 0 clippy warnings
- RFC 3161 TSA test coverage gap closed (prior session)
- benchScale harness ready for integration into plasmidBin deploy gate
