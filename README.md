# loamSpine

Permanence Layer - Selective Memory & Certificates

## Status

🌱 **Nascent** — Scaffolded from SourDough

## Quick Start

```bash
# Build
cargo build

# Test
cargo test

# Run
cargo run
```

## Architecture

```
loamSpine/
├── Cargo.toml           # Workspace manifest
├── crates/
│   └── loam-spine-core/  # Core library
├── specs/               # Specifications
└── showcase/            # Demonstrations
```

## Integration

loamSpine integrates with the ecoPrimals ecosystem via SourDough traits:

- `PrimalLifecycle` — Start/stop/reload
- `PrimalHealth` — Health checks
- `PrimalIdentity` — BearDog integration (TODO)
- `PrimalDiscovery` — Songbird integration (TODO)

## License

AGPL-3.0

---

*Born from SourDough, growing into an ecoPrimal.*
