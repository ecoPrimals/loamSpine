# Showcase (Fossilized)

Contents archived to `ecoPrimals/fossilRecord/primals/loamSpine/showcase_wave49/`
as part of Wave 49 ecosystem tightening (May 2026).

These demos powered the prokaryotic → post-primordial evolution and are
preserved as fossil record. Active patterns live in `primalSpring/wateringHole/`.

## What Was Here

- **Level 1**: 10 local primal demos (spine CRUD, certificates, proofs, backup, storage, concurrency, temporal, waypoint, recursive)
- **Level 2**: 6 RPC API demos (tarpc, JSON-RPC, health, concurrent, error handling, lifecycle)
- **Level 3+4**: Inter-primal demos (fossilized prior to Wave 49 — mined into primalSpring experiments)

## Active Validation

API surface validation now lives in `infra/benchScale/validate_roundtrip.sh` —
52 validations across all 47 JSON-RPC methods via live TCP roundtrip.

## Running Examples

Runnable Rust examples remain in the crate:

```bash
cargo run --example demo_hello_loamspine
cargo run --example demo_entry_types
cargo run --example demo_certificate_lifecycle
cargo run --example demo_backup_restore
cargo run --example storage_backends
cargo run --example demo_rpc_service
cargo run --example demo_inter_primal
```
