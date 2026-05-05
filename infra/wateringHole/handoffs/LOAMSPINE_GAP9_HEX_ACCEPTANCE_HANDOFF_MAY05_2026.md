<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# loamSpine — Gap 9: Hex String Acceptance + Redundant Committer Fix

**Date**: May 5, 2026  
**Version**: 0.9.16  
**From**: loamSpine team  
**To**: primalSpring, all composition consumers

---

## Summary

Two wire-format ergonomic issues from primalSpring Phase 58+ audit are resolved:

1. **Hex string acceptance**: All `ContentHash` / `EntryHash` fields (`[u8; 32]`) now accept
   both the native JSON byte array (`[1,2,3,...,32]`) and a 64-character hex string
   (`"0102..."` or `"0x0102..."`). Serialization output is unchanged (byte arrays).

2. **Redundant `committer` in `AppendEntryRequest`**: The top-level `committer: Did` field
   was required but never read — `Entry.committer` is always derived from `spine.owner`.
   Changed to `committer: Option<Did>` with `#[serde(default)]`. Old callers sending it
   are unaffected; new callers can omit it.

## Technical Details

### Hex acceptance

Custom serde modules `serde_content_hash` and `serde_opt_content_hash` in
`loam-spine-core/src/types.rs` provide `deserialize` functions using a visitor that
accepts `visit_seq` (byte array), `visit_str` (hex string with optional `0x` prefix),
and `visit_bytes` (raw bytes). Applied via `#[serde(deserialize_with = "...")]` on all
wire-facing `ContentHash`/`EntryHash` fields:

- **Entry struct**: `previous`
- **EntryType variants**: `merkle_root`, `source_entry`, `checkout_entry`, `summary`,
  `data_hash`, `braid_hash`, `subject_hash`, `loan_entry`, `origin_entry`, `moment_id`,
  `state_hash`
- **API request types**: `entry_hash`, `session_hash`, `braid_hash`
- **API response types**: `genesis_hash`, `seal_hash`, `entry_hash`, `tip_hash`,
  `anchor_hash`, `checkout_hash`, `commit_hash`, `merkle_root`
- **Temporal types**: `EphemeralProvenance::merkle_root`, `Moment::id`, `Moment::state_hash`,
  `MomentContext` variant hashes
- **Proof types**: `InclusionProof::entry_hash`, `InclusionProof::tip`,
  `CertificateOwnershipProof::chain_root`, `ProvenanceProof::data_hash`

MessagePack (rmp_serde) storage serialization is unaffected — byte arrays continue
to round-trip as before.

### Optional committer

`AppendEntryRequest.committer` changed from `Did` to `Option<Did>` with `#[serde(default)]`.
The handler in `entry_ops.rs` never reads this field. Backward-compatible.

## Test Coverage

- 9 serde unit tests (byte array, hex, 0x-hex, mixed case, wrong length, invalid chars,
  null option, option byte array, option hex)
- 5 API wire tests (hex DataAnchor, hex session_hash, hex entry_hash, omitted committer,
  explicit committer backward compat)
- 1,504 total tests, all pass
- Clippy clean, `cargo deny check` clean, `cargo fmt` clean

## Port / Discovery (informational, no action)

- Port 9700 acknowledged. Default is 8080, configurable via `LOAMSPINE_JSONRPC_PORT`.
  primalSpring sets 9700 externally for ironGate. No code change needed.
- Discovery Escalation Hierarchy acknowledged. loamSpine supports UDS (tier 3) and
  socket registry (tier 4).
