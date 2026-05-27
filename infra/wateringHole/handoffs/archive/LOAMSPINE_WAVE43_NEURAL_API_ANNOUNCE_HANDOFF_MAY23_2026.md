# loamSpine — Wave 43: Neural API `primal.announce` Adoption

**Date:** 2026-05-23
**Audit:** Wave 43 — Neural API `primal.announce` Adoption Blurbs
**License:** AGPL-3.0-or-later

---

## Summary

loamSpine's startup registration evolved from legacy `lifecycle.register` to
Wave 43 `primal.announce` with full Neural API routing intelligence schema.

| Area | Before | After |
|------|--------|-------|
| Outbound method | `lifecycle.register` | `primal.announce` |
| `socket` field | `socket_path` (non-standard) | `socket` (biomeOS v3.68+) |
| `capabilities` | Flat RPC method list | Semantic domains: `["anchor", "ledger", "permanence"]` |
| `signal_tiers` | absent | `["nest"]` |
| `cost_hints` | absent | `{ anchor: 20.0, ledger: 15.0, permanence: 30.0 }` |
| `latency_estimates` | absent | `{ anchor: 50, ledger: 20, permanence: 100 }` |
| `methods` | absent from announce | Full 43-method `niche::METHODS` array |

---

## Changes

- `neural_api/mod.rs`: New constants `SIGNAL_TIERS`, `ANNOUNCE_CAPABILITIES`, `COST_HINTS`, `LATENCY_ESTIMATES`. New `announce_payload()` builder. `register_at_socket()` sends `primal.announce` instead of `lifecycle.register`.
- `jsonrpc/mod.rs`: Inbound `primal.announce` handler delegates to `announce_payload()` for unified response shape.
- `niche.rs`: `primal.announce` added to `SEMANTIC_MAPPINGS` and `COST_ESTIMATES`.
- Clippy fixes: `#[must_use]`, `is_multiple_of`, `const fn` promotions, `# Errors` doc, `unused_async` suppression, unfulfilled lint expectations.
- 4 new tests (1,527 total).

---

## Validation

After loamSpine starts with biomeOS v3.69+:
```bash
echo '{"jsonrpc":"2.0","method":"neural_api.routing_weights","params":{},"id":1}' | \
  socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/biomeos/neural-api-ecoPrimal.sock
```

Should show entries for `anchor.*`, `ledger.*`, `permanence.*` with non-default
affinity and `signal_tiers: ["nest"]`.

---

## Upstream Action Items

| Consumer | Notes |
|----------|-------|
| biomeOS | Should see loamSpine `primal.announce` with routing weights |
| primalSpring | Can validate loamSpine announce payload shape in composition |
