# LoamSpine — RootPulse Audit Response (RP-2, RP-3, RP-5)

**Date:** May 7, 2026
**From:** loamSpine team
**To:** biomeOS, primalSpring, BearDog
**Ref:** primalSpring Phase 58+ — projectNUCLEUS Sovereignty Testing

---

## Items Addressed

### RP-2: `spine.create` Prerequisite — DOCUMENTED

`session.commit` requires a pre-existing spine. `spine.create` must be called
first to obtain a `spine_id`. This is documented in `specs/API_SPECIFICATION.md`
with a two-step example (create then commit).

**Auto-creation alternative:** `permanence.commit_session` auto-creates a spine
per committer DID via `ensure_spine(committer, None)`. Graph workflows that
don't need explicit spine management can use this method instead.

**Why not auto-create in `session.commit`:** Auto-creating on an unknown
`spine_id` could mask bugs where callers pass wrong UUIDs. The explicit
`spine.create` → `session.commit` lifecycle is intentional. The permanence compat
path exists for workflows that want auto-creation.

### RP-3: Hex String Acceptance — ALREADY RESOLVED (Gap 9)

All `ContentHash`/`EntryHash` fields accept both JSON byte arrays and 64-char
hex strings since `d5ed0c6` (May 5, 2026). The `serde_content_hash` /
`serde_opt_content_hash` modules handle deserialization transparently. biomeOS
graph schemas can send `"0102...0320"` instead of `[1,2,...,32]`.

### RP-5: Entry Signing Contract — DOCUMENTED

Entry signing is **internal to LoamSpine**, not orchestrator-provided:

| Aspect | Behavior |
|--------|----------|
| Who signs | LoamSpine calls BearDog `crypto.sign_ed25519` when `BEARDOG_SOCKET` is set |
| What is signed | `entry.to_canonical_bytes()` (all fields except metadata) |
| Where stored | `entry.metadata["tower_signature"]` (base64 Ed25519) |
| No Tower | Entries are unsigned (standalone mode) |
| `committer` | Derived from `spine.owner`, not from request. DID format: `did:key:z6Mk...` |
| `AppendEntryRequest.committer` | Optional (`Option<Did>`), ignored by append logic |
| `SessionCommit.committer` | Used — embedded in the `SessionCommit` entry type |

**For RootPulse graph:** The orchestrator calls `spine.create` (Phase 0) and
`session.commit` (Phase 5). It does **not** need to call `crypto.sign` for
LoamSpine entries — Tower delegation is handled internally. Phase 3 signing
(via BearDog) produces signatures for the dehydration merkle root, which is
a separate concern from LoamSpine entry signing.

---

## Cross-Primal Notes for biomeOS

- **RP-1** (method/param mismatches): loamSpine's method is `session.commit`
  (not `commit.session`), and `braid.create` (not `provenance.create_braid`).
  Update `rootpulse_commit.toml` accordingly.
- **RP-4** (graph executor): Not a loamSpine concern — biomeOS architectural item.

## Cross-Primal Notes for BearDog

- **RP-5** (signing scope): BearDog signs whatever bytes are presented via
  `crypto.sign_ed25519`. LoamSpine presents entry canonical bytes. The
  dehydration merkle root (Phase 3) is a separate signing event. Both use
  the same BearDog key but sign different payloads.

---

## Handoff from Other Primals (Awareness Only)

These items are for their respective primals, documented here for tracking:

| Primal | Item | Status |
|--------|------|--------|
| petalTongue | PT-1–5 (web mode gaps) | petalTongue team |
| NestGate | NG-1–4 (content-addressed storage) | NestGate team |
| biomeOS | RP-1,3,4 (graph schema/executor) | biomeOS team |
| barraCuda | stats.entropy, shader absorption | barraCuda team |
| rhizoCrypt | Silent timeout on connect | rhizoCrypt team |
| toadStool | Short timeout sensitivity | toadStool team |
