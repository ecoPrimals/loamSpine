# benchScale — loamSpine Roundtrip Validation

Local deployment validation harness for loamSpine, aligned with the
wateringHole `DEPLOYMENT_VALIDATION_STANDARD` v1.1.

## Quick Start

```bash
# Full build + validate
./infra/benchScale/validate_roundtrip.sh

# Skip build (binary already exists)
SKIP_BUILD=1 ./infra/benchScale/validate_roundtrip.sh

# Custom port
LOAMSPINE_PORT=8080 ./infra/benchScale/validate_roundtrip.sh
```

## What It Does

Starts a loamSpine TCP server on `127.0.0.1:19710`, exercises **all 43
canonical JSON-RPC methods** via HTTP POST, validates responses, and
reports results. Server lifecycle is fully managed (start → validate →
cleanup).

## Validation Phases

| Phase | Methods | What It Validates |
|-------|---------|-------------------|
| 1 | `health.*` | Health triad (liveness, readiness, check) |
| 2 | `capabilities.list`, `identity.get`, `lifecycle.status` | Meta/discovery |
| 3 | `auth.*` | JH-0 method gate (mode, check, peer_info) |
| 4 | `btsp.*` | BTSP cipher negotiation |
| 5 | `spine.*` | Spine CRUD lifecycle |
| 6 | `entry.*` | Entry append, get, tip, list |
| 7 | `session.commit`, `braid.commit` | Provenance integration + alias |
| 8 | `certificate.*` | Certificate mint + get |
| 9 | `slice.*` | Slice anchor + checkout |
| 10 | `proof.*` | Inclusion proof generate + verify |
| 11 | `anchor.*` | Public chain anchoring (Bitcoin, Ethereum, RFC 3161) |
| 12 | `bonding.ledger.*` | Bond ledger store/retrieve/list |
| 13 | `permanence.*` | Legacy compat layer roundtrip |
| 14 | `tools.*` | MCP tool discovery + invocation |
| 15 | aliases | Method normalization (`provenance.commit`, `capability.list`, etc.) |
| 16 | `spine.seal` | Seal + sealed-spine rejection |
| 17 | error paths | Unknown method (-32601), not-found spine |
| 18 | `lifecycle.status` | Uptime verification (uptime_s > 0) |
| 19 | `primal.announce` | Self-registration payload |

## Dependencies

- `curl` (HTTP POST transport)
- `jq` (JSON response validation)
- `ss` or `lsof` (port detection fallback)
- `bash` 4+

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `LOAMSPINE_PORT` | `19710` | JSON-RPC TCP port |
| `LOAMSPINE_TARPC_PORT` | `19711` | tarpc port |
| `LOAMSPINE_BIND` | `127.0.0.1` | Bind address |
| `SKIP_BUILD` | unset | Set to `1` to skip `cargo build --release` |

## Relationship to Canonical benchScale

The canonical benchScale substrate lives at `sort-after/benchScale/` and
provides multi-node Docker/libvirt labs. This local harness validates a
**single loamSpine instance** for pre-deploy gating — the same checks
that `benchscale validate ipc 127.0.0.1:<port>` runs, plus full method
coverage.

For multi-primal composition testing, use primalSpring experiments
(`exp084`, `exp094`) against a benchScale Docker lab with the
`ecoprimals-nucleus-3node` topology.
