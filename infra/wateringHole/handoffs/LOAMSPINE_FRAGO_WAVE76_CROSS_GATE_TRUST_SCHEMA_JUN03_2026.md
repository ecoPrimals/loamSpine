<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# loamSpine — FRAGO ACK: Wave 76 Cross-Gate Trust Schema

**Date**: 2026-06-03  
**In Response To**: `wave76-parity-sprint-provenance` (P1)  
**From**: loamSpine team (strandGate)  
**To**: primalSpring (eastGate), rhizoCrypt + sweetGrass (provenance trio)  
**Status**: **ACK — COMPLETE**

---

## Schema Definitions

Three new `EntryType` variants added to `crates/loam-spine-core/src/entry/mod.rs` under domain `"trust"`:

### 1. `KeyExchange`

Permanent record of an Ed25519 key exchange between two gates.

```rust
KeyExchange {
    local_gate: Did,          // DID of the local gate
    remote_gate: Did,         // DID of the remote gate
    public_key_hash: ContentHash, // Blake3 hash of the exchanged public key
    direction: String,        // "initiated" or "accepted"
    family_id: Option<String>, // Family scope (skip_serializing_if None)
}
```

**Design notes**: The raw key material stays with the crypto capability primal (bearDog). loamSpine records only the Blake3 hash — enough for audit and verification without storing secrets.

### 2. `TrustIssuerRegistration`

Permanent record of a trust issuer being added to the `TrustedIssuerRegistry`.

```rust
TrustIssuerRegistration {
    issuer_did: Did,           // DID of the issuer being registered
    registering_gate: Did,     // Gate that registered this issuer
    trust_scope: String,       // "family", "cross-gate", or "global"
    capabilities: Vec<String>, // Trusted capabilities (e.g. ["signing", "verification"])
    expires_at: Option<Timestamp>, // Expiry (skip_serializing_if None)
}
```

**Design notes**: `capabilities` uses `Vec<String>` for extensibility — any capability the issuer is trusted for can be recorded without schema changes.

### 3. `TokenVerificationCrossGate`

Permanent audit trail of cross-gate token verification exercises.

```rust
TokenVerificationCrossGate {
    issuer_gate: Did,           // Gate that issued the token
    verifier_gate: Did,         // Gate that verified the token
    token_hash: ContentHash,    // Blake3 hash of the verified token
    verified: bool,             // Verification outcome
    failure_reason: Option<String>, // Reason on failure (skip_serializing_if None)
}
```

**Design notes**: Records both success and failure — the immutable audit trail captures all trust exercises, including rejected ones.

---

## Domain & Routing

All three variants return domain `"trust"` from `EntryType::domain()`. None are allowed in waypoint spines (`allowed_in_waypoint()` returns `false`).

---

## Test Results

**9 new tests** (1,574 → 1,583 total):

| Test | What it validates |
|------|-------------------|
| `entry_type_domain_cross_gate_trust` | All 3 variants return domain `"trust"` |
| `cross_gate_trust_not_allowed_in_waypoint` | All 3 excluded from waypoint spines |
| `entry_type_serde_roundtrip_key_exchange` | JSON serialize/deserialize roundtrip |
| `entry_type_serde_roundtrip_trust_issuer_registration` | JSON serialize/deserialize roundtrip |
| `entry_type_serde_roundtrip_token_verification_cross_gate` | Success case roundtrip |
| `entry_type_serde_roundtrip_token_verification_failed` | Failure case with reason |
| `entry_type_serde_key_exchange_no_family` | Optional `family_id` skipped when None |
| `entry_type_serde_trust_issuer_no_expiry` | Optional `expires_at` skipped when None |
| `cross_gate_entry_full_roundtrip` | Full `Entry` with `KeyExchange` type — canonical bytes + JSON + metadata |

All 1,583 tests pass. Zero clippy warnings. Zero cargo check warnings.

---

## Coordination Notes

- **bearDog w135**: The `KeyExchange` variant records the hash of the key, not the key itself. bearDog owns the raw key material; loamSpine owns the permanent record.
- **rhizoCrypt**: Can record `trust_issuer_registered` and `key_exchange_completed` DAG events. loamSpine will anchor those sessions via `session.commit` as before.
- **sweetGrass**: Can weave cross-gate attribution braids referencing these trust entries as provenance anchors.
- **Schema extensibility**: `EntryType` is `#[non_exhaustive]` — future trust event types can be added without breaking downstream consumers.
