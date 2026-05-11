# LoamSpine — Stadial Gate: API Contract Reconciliation

**Date:** May 11, 2026
**From:** loamSpine team
**To:** primalSpring, biomeOS (RootPulse graph), rhizoCrypt, sweetGrass
**Ref:** primalSpring full stadial gate blurb — "loamSpine API contract mismatch"

---

## Status: ALREADY RESOLVED — No code changes required

The audit flagged that RootPulse Phase 5 uses a different method name or param
shape than `CommitSessionRequest`. All three resolution layers are already
in place:

### 1. Method Name Aliases (since v0.9.16)

```
"commit.session"    → "session.commit"   (canonical)
"provenance.commit" → "session.commit"   (canonical)
```

Normalization happens in `normalize_method()` before dispatch. The RootPulse
graph can use any of these three names.

### 2. Hex String Acceptance (Gap 9, May 5, 2026)

All `ContentHash`/`EntryHash` fields — including `session_hash` — accept both:
- JSON byte arrays: `[1, 2, 3, ..., 32]`
- 64-char hex strings: `"0102...0320"` (with optional `0x` prefix)

biomeOS graph schemas can send hex strings instead of constructing byte arrays.

### 3. Entry Signing Contract (RP-5, May 7, 2026)

Documented in `API_SPECIFICATION.md` §3.4:
- Callers do **not** provide signatures
- LoamSpine calls BearDog `crypto.sign_ed25519` internally when `BEARDOG_SOCKET` is set
- `committer` on `AppendEntryRequest` is optional and ignored (`Entry.committer` = `spine.owner`)
- `committer` on `CommitSessionRequest` is embedded in the `SessionCommit` entry type

### 4. Spine Prerequisite (RP-2, May 7, 2026)

`session.commit` requires a pre-existing spine (`spine.create` first).
`permanence.commit_session` auto-creates a spine per committer DID.

---

## Canonical API Contract

### `session.commit` (native path — explicit spine management)

```json
{
  "jsonrpc": "2.0",
  "method": "session.commit",
  "params": {
    "spine_id": "01234567-89ab-cdef-0123-456789abcdef",
    "session_id": "fedcba98-7654-3210-fedc-ba9876543210",
    "session_hash": "a1b2c3d4e5f6...64_hex_chars",
    "vertex_count": 42,
    "committer": { "value": "did:key:z6MkCommitter" }
  },
  "id": 1
}
```

### `permanence.commit_session` (compat path — auto-creates spine)

```json
{
  "jsonrpc": "2.0",
  "method": "permanence.commit_session",
  "params": {
    "session_id": "fedcba98-7654-3210-fedc-ba9876543210",
    "merkle_root": "a1b2c3d4e5f6...64_hex_chars",
    "summary": {
      "session_type": "rootpulse",
      "vertex_count": 42,
      "leaf_count": 5,
      "started_at": 1710000000000000000,
      "ended_at": 1710000060000000000,
      "outcome": "Success"
    },
    "committer_did": "did:key:z6MkCommitter"
  },
  "id": 1
}
```

---

## Provenance Trio Verification

10 integration tests in `crates/loam-spine-api/tests/provenance_trio.rs` confirm:

| Test | What it validates |
|------|-------------------|
| `dehydration_flow_native_session_commit` | Native `session.commit` with explicit spine |
| `dehydration_flow_permanent_storage_compat` | Compat path with auto-created spine |
| `dehydration_flow_compat_auto_creates_spine` | Spine auto-creation per committer DID |
| `braid_anchoring_flow` | sweetGrass → loamSpine braid anchoring |
| `braid_anchoring_with_proof` | Braid with inclusion proof |
| `full_provenance_trio_flow` | rhizoCrypt → loamSpine → sweetGrass end-to-end |
| `provenance_trio_via_compat_layer` | Full trio via compat layer |
| `permanent_storage_health_check` | Compat health endpoint |
| + 2 additional commit/verify tests | Session verification roundtrip |

---

## For biomeOS (RootPulse Graph Fix)

The RootPulse `rootpulse_commit.toml` Phase 5 should:

1. Use method `session.commit` (or alias `commit.session`)
2. Include `spine_id` (from a Phase 0 `spine.create` step) or use
   `permanence.commit_session` which auto-creates
3. Send `session_hash` as hex string (not byte array)
4. Do NOT send a signature — loamSpine handles signing internally

This is a biomeOS graph spec update, not a loamSpine code change.

---

## Other Primals in This Audit (Awareness Only)

| Primal | Item | loamSpine involvement |
|--------|------|----------------------|
| NestGate | `content.*` transport parity (CRITICAL) | None |
| petalTongue | `backend=nestgate` (blocked on NestGate) | None |
| skunkBat | JH-5 Phase 3 forwarding | None |
| rhizoCrypt | Provenance trio pipeline | loamSpine is the middle link — verified working |
| sweetGrass | Braid creation in composition | loamSpine is the bridge — verified working |
| toadStool | IPC env var expansion | None |
| squirrel | Compute delegation | None |
| barraCuda | Crypto dedup | None |
| bearDog | Crypto IPC surface | None |
| biomeOS | Content routing | None |
| songbird | CLEAN | None |
| coralReef | CLEAN | None |
