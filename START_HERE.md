# 🚀 LoamSpine — Start Here

**Welcome to LoamSpine!** This guide will get you up and running quickly.

---

## ⚡ Quick Start (5 minutes)

```bash
# 1. Clone and navigate
cd /path/to/loamSpine

# 2. Build
cargo build --release

# 3. Run tests
cargo test --workspace

# 4. Try an example
cargo run --example hello_loamspine

# 5. View documentation
cargo doc --open --no-deps
```

**✅ If all steps pass, you're ready to develop!**

---

## 📊 Project Status

**Version**: 0.6.0  
**Status**: ✅ **PRODUCTION READY**  
**Last Audit**: December 26, 2025

| Metric | Status |
|--------|--------|
| Tests | 407 passing (100%) ✅ |
| Coverage | 77.66% (exceeds target) ✅ |
| Linting | 0 warnings (pedantic) ✅ |
| Unsafe Code | 0 (forbidden) ✅ |
| Technical Debt | ZERO ✅ |

---

## 🎯 What is LoamSpine?

LoamSpine is the **immutable permanence layer** for the ecoPrimals ecosystem:

- **Selective Memory** — Only deliberately committed data becomes permanent
- **Sovereign Spines** — Each user controls their own history ledger
- **Loam Certificates** — Digital ownership with lending and provenance
- **Pure Rust RPC** — No gRPC, no protobuf, just Rust
- **Capability Discovery** — Services find each other at runtime (no hardcoding)

---

## 📚 Essential Reading

1. **[README.md](./README.md)** — Overview, API reference, quick start
2. **[FINAL_STATUS_DEC_26_2025.md](./FINAL_STATUS_DEC_26_2025.md)** — Complete status report
3. **[specs/LOAMSPINE_SPECIFICATION.md](./specs/LOAMSPINE_SPECIFICATION.md)** — Core specification
4. **[CONTRIBUTING.md](./CONTRIBUTING.md)** — How to contribute

---

## 🏗️ Architecture Overview

```
LoamSpine
├── Core Library (loam-spine-core)
│   ├── Spines (immutable ledgers)
│   ├── Entries (timestamped records)
│   ├── Certificates (digital ownership)
│   ├── Proofs (cryptographic verification)
│   └── Discovery (runtime service location)
│
└── API Layer (loam-spine-api)
    ├── tarpc (high-performance binary RPC)
    ├── JSON-RPC 2.0 (universal access)
    └── Health checks
```

**Key Principle**: LoamSpine knows only itself. All other services are discovered at runtime.

---

## 💻 Development Workflow

### 1. Make Changes
```bash
# Edit files in crates/loam-spine-core/src/ or crates/loam-spine-api/src/
```

### 2. Run Tests
```bash
# Run all tests
cargo test --workspace

# Run specific test
cargo test --test e2e

# Run with output
cargo test -- --nocapture
```

### 3. Check Quality
```bash
# Lint (must pass with 0 warnings)
cargo clippy --workspace --all-features -- -D warnings

# Format
cargo fmt --all

# Build docs
cargo doc --no-deps
```

### 4. Verify Coverage
```bash
# Generate coverage report
cargo llvm-cov --workspace

# View in browser
cargo llvm-cov --open
```

### 5. Run Benchmarks
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench core_ops
```

---

## 🎭 Try the Showcase

21 interactive demos demonstrate all features:

```bash
# Quick start all demos
cd showcase && ./QUICK_START.sh

# Or run individual levels:
cd showcase/01-local-primal && ./RUN_ALL.sh     # Local capabilities
cd showcase/02-rpc-api && ./RUN_ALL.sh          # RPC interactions
cd showcase/03-songbird-discovery && ./RUN_ALL.sh  # Service discovery
cd showcase/04-inter-primal && ./RUN_ALL.sh     # Primal integration
```

**Each demo includes**:
- README explaining concepts
- Shell script for execution
- Example output and receipts

---

## 📖 Code Examples

### Create a Spine
```rust
use loam_spine_core::{Spine, SpineBuilder, Did};

let owner = Did::new("did:key:z6MkOwner");
let spine = SpineBuilder::new(owner)
    .with_name("My History")
    .build()?;

println!("Created spine: {}", spine.id());
```

### Add an Entry
```rust
use loam_spine_core::{Entry, EntryType, ByteBuffer};

let data = ByteBuffer::from(b"Hello, LoamSpine!".to_vec());
let entry = Entry::new(
    spine.id(),
    EntryType::GenericData,
    data,
    owner.clone(),
);

spine.append(entry)?;
```

### Mint a Certificate
```rust
use loam_spine_core::{Certificate, CertificateType};

let cert = Certificate::new(
    spine.id(),
    entry.hash(),
    CertificateType::DigitalCollectible {
        title: "My First NFT".to_string(),
        creator: owner.clone(),
    },
    owner.clone(),
);

println!("Minted certificate: {}", cert.id());
```

### Discover Services
```rust
use loam_spine_core::service::infant_discovery::InfantDiscovery;

// Infant discovery: start with self-knowledge only
let infant = InfantDiscovery::new(vec![
    "persistent-ledger".to_string(),
    "waypoint-anchoring".to_string(),
]);

// Discover discovery service (tries env vars, DNS SRV, mDNS)
match infant.discover_discovery_service().await {
    Ok(endpoint) => println!("Found discovery at: {}", endpoint),
    Err(_) => println!("Operating standalone"),
}
```

---

## 🧪 Testing

### Test Organization
```
crates/
├── loam-spine-core/
│   ├── src/              # Unit tests (inline)
│   └── tests/
│       ├── e2e.rs        # End-to-end scenarios (6 tests)
│       ├── fault_tolerance.rs  # Fault injection (16 tests)
│       └── songbird_integration.rs  # Discovery tests (8 tests)
└── loam-spine-api/
    ├── src/              # Unit tests (inline)
    └── tests/
        └── api_integration.rs  # API integration tests
```

### Test Categories
1. **Unit Tests** (338 tests) — Fast, focused, isolated
2. **Integration Tests** (69 tests) — API interactions, workflows
3. **Fault Tolerance** (16 tests) — Network, disk, memory, clock, Byzantine
4. **E2E Tests** (6 tests) — Complete user scenarios

### Running Tests
```bash
# All tests
cargo test --workspace

# Specific test
cargo test test_full_spine_lifecycle

# Test with logging
RUST_LOG=debug cargo test -- --nocapture

# Test coverage
cargo llvm-cov --workspace --open
```

---

## 🐛 Debugging

### Enable Logging
```bash
# All modules
RUST_LOG=debug cargo test

# Specific module
RUST_LOG=loam_spine_core::service=trace cargo test

# Multiple modules
RUST_LOG=loam_spine_core=debug,loam_spine_api=info cargo test
```

### Common Issues

**Issue**: Tests fail with "Address already in use"
```bash
# Kill existing processes
pkill -f loamspine
```

**Issue**: Clippy warnings
```bash
# Fix automatically where possible
cargo clippy --fix --workspace --all-features
```

**Issue**: Formatting errors
```bash
# Auto-format
cargo fmt --all
```

---

## 📦 Project Structure

```
loamSpine/
├── README.md                 # Project overview
├── START_HERE.md            # This file
├── ROOT_DOCS_INDEX.md       # Documentation index
├── FINAL_STATUS_DEC_26_2025.md  # Status report
├── Cargo.toml               # Workspace manifest
├── bin/
│   └── loamspine-service/   # Standalone service binary
├── crates/
│   ├── loam-spine-core/     # Core library (~10,000 LOC)
│   └── loam-spine-api/      # API layer (~3,000 LOC)
├── specs/                   # Specifications (11 files)
├── showcase/                # Interactive demos (21 demos)
├── fuzz/                    # Fuzz testing (3 targets)
├── examples/                # Top-level examples
└── docs/                    # Additional documentation
```

---

## 🔧 Tools & Dependencies

### Required
- **Rust** 1.75.0+ (MSRV)
- **Cargo** (comes with Rust)

### Optional (for full development)
- **llvm-cov** — Coverage reporting (`cargo install cargo-llvm-cov`)
- **cargo-deny** — Security audits (`cargo install cargo-deny`)
- **cargo-fuzz** — Fuzz testing (`cargo install cargo-fuzz`)
- **Docker** — Container deployment

### Install Tools
```bash
# Coverage
cargo install cargo-llvm-cov

# Security audits
cargo install cargo-deny

# Fuzz testing
cargo install cargo-fuzz

# Check installations
cargo llvm-cov --version
cargo deny --version
cargo +nightly fuzz --version
```

---

## 🚀 Deployment

### Docker
```bash
# Build image
docker build -t loamspine:0.6.0 .

# Run container
docker run -p 9001:9001 -p 8080:8080 loamspine:0.6.0

# Or use docker-compose
docker-compose up -d
```

### Configuration
```bash
# Environment variables
export DISCOVERY_ENDPOINT=http://localhost:8082
export LOAMSPINE_STORAGE_PATH=/data/loamspine
export LOAMSPINE_TARPC_PORT=9001
export LOAMSPINE_JSONRPC_PORT=8080

# Run service
./target/release/loamspine-service
```

---

## 💡 Best Practices

### Code Style
- Follow Rust idioms and conventions
- Use clippy pedantic level
- Write doc comments for public items
- Include examples in doc comments

### Testing
- Write unit tests for business logic
- Write integration tests for APIs
- Test error cases
- Aim for >60% coverage (current: 77.66%)

### Documentation
- Update docs with code changes
- Keep README current
- Add examples for new features
- Document public APIs

### Git Workflow
```bash
# Create feature branch
git checkout -b feature/my-feature

# Make changes, run checks
cargo test --workspace
cargo clippy --workspace --all-features -- -D warnings
cargo fmt --all

# Commit
git commit -m "feat: add my feature"

# Push and create PR
git push origin feature/my-feature
```

---

## 🤝 Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed guidelines.

**Quick checklist**:
- ✅ Tests pass (`cargo test --workspace`)
- ✅ Clippy clean (`cargo clippy --workspace --all-features -- -D warnings`)
- ✅ Formatted (`cargo fmt --all`)
- ✅ Documentation updated
- ✅ Examples added for new features

---

## 📞 Getting Help

1. **Documentation**: Check [ROOT_DOCS_INDEX.md](./ROOT_DOCS_INDEX.md)
2. **Specifications**: Read [specs/](./specs/)
3. **Examples**: Try [showcase/](./showcase/)
4. **Code**: Browse [crates/](./crates/)

---

## 🎓 Learning Path

### Week 1: Foundations
1. Read [README.md](./README.md)
2. Run `cargo test --workspace`
3. Try [showcase/01-local-primal/](./showcase/01-local-primal/)
4. Read [specs/LOAMSPINE_SPECIFICATION.md](./specs/LOAMSPINE_SPECIFICATION.md)

### Week 2: Deep Dive
1. Study code in `crates/loam-spine-core/src/`
2. Try [showcase/02-rpc-api/](./showcase/02-rpc-api/)
3. Read [specs/ARCHITECTURE.md](./specs/ARCHITECTURE.md)
4. Write a small feature

### Week 3: Advanced
1. Try [showcase/03-songbird-discovery/](./showcase/03-songbird-discovery/)
2. Try [showcase/04-inter-primal/](./showcase/04-inter-primal/)
3. Read [specs/INTEGRATION_SPECIFICATION.md](./specs/INTEGRATION_SPECIFICATION.md)
4. Contribute a feature or fix

---

## ✅ Quick Verification

Run this to verify your setup:

```bash
#!/bin/bash
echo "🔍 Verifying LoamSpine setup..."

echo "✅ Building..."
cargo build --quiet || exit 1

echo "✅ Running tests..."
cargo test --workspace --quiet || exit 1

echo "✅ Checking lints..."
cargo clippy --workspace --all-features --quiet -- -D warnings || exit 1

echo "✅ Checking format..."
cargo fmt --all -- --check || exit 1

echo "🎉 All checks passed! You're ready to develop."
```

---

**🦴 LoamSpine: Where memories become permanent.**

**Ready to build? Start with [showcase/](./showcase/) or dive into [crates/loam-spine-core/src/](./crates/loam-spine-core/src/)!**
