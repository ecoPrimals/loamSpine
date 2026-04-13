<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Contributing to LoamSpine

Thank you for your interest in contributing to LoamSpine! This document provides guidelines and best practices for contributing to the project.

---

## 🦴 Core Principles

### Primal Sovereignty
- **Pure Rust**: No C++ dependencies (no gRPC, protobuf, protoc)
- **Cargo-Only Builds**: `cargo build` must be sufficient
- **Self-Knowledge**: Primal code knows only itself, discovers others at runtime
- **Runtime Discovery**: Use capability registry, not hardcoded dependencies

### Code Quality
- **Zero Unsafe in Production**: `#![forbid(unsafe_code)]` workspace-wide; test modules use inner/outer function pattern for env injection
- **Pedantic Linting**: `clippy::pedantic` and `clippy::nursery` must pass
- **High Coverage**: Aim for 90%+ line coverage (current: 90.92% line / 92.92% region, 1,395 tests)
- **File Size**: Keep files under 1000 lines; refactor smartly, not just split
- **Modular Design**: Use domain-specific modules (see `service/` pattern)
- **Zero-Copy**: Use `bytes::Bytes` for network buffers when possible
- **SPDX Headers**: `// SPDX-License-Identifier: AGPL-3.0-or-later` on all `.rs` files
- **No Hardcoding**: Primal names, ports, and endpoints discovered at runtime
- **`#[expect]` over `#[allow]`**: Use `#[expect(lint, reason = "...")]` for lint exceptions — documents why and warns when the exception becomes stale

### Human Dignity
- **No Surveillance**: No tracking, analytics, or telemetry
- **Sovereign Data**: Users own their spines and history
- **Open Standards**: JSON-RPC for external access

---

## 🚀 Getting Started

### Prerequisites
- Rust 1.85.0 or later (edition 2024 MSRV)
- `cargo-llvm-cov` for coverage: `cargo install cargo-llvm-cov`
- `cargo-deny` for security: `cargo install cargo-deny`
- `cargo-fuzz` for fuzzing (optional): `cargo install cargo-fuzz`

### Build Environment

If your workspace is on a `noexec` mount, override the target directory via
environment variable (`.cargo/config.toml` does not set `target-dir`):

```bash
CARGO_TARGET_DIR=~/.cargo-build/loamSpine/target cargo build
```

### musl-static Builds (ecoBin / plasmidBin)

```bash
# Prerequisites
rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl
sudo apt install musl-tools gcc-aarch64-linux-gnu

# Build via aliases defined in .cargo/config.toml
cargo build-x64    # x86_64-unknown-linux-musl
cargo build-arm64  # aarch64-unknown-linux-musl
```

### Build and Test
```bash
# Build
cargo build

# Test (1,395 tests, all concurrent)
cargo test --workspace

# Linting (must pass, zero warnings)
cargo clippy --workspace --all-targets -- -D warnings

# Formatting
cargo fmt --all -- --check

# Coverage (requires cargo-llvm-cov)
cargo llvm-cov --workspace --summary-only

# License and dependency audit (requires cargo-deny)
cargo deny check licenses bans sources

# Full verification
./verify.sh

# Build docs
cargo doc --workspace --no-deps --open
```

---

## 📝 Code Style

### File Structure
```rust
//! Module documentation
//!
//! Describe purpose and key types.

use std::collections::HashMap;  // Std imports first
use tokio::sync::RwLock;        // External crates second

use crate::types::SpineId;      // Internal imports last

/// Type documentation
pub struct MyType { ... }

impl MyType { ... }

#[cfg(test)]
mod tests { ... }
```

### Error Handling
- Use `LoamSpineResult<T>` for fallible operations
- Return descriptive errors with context
- Never use `.unwrap()` or `.expect()` in production code
- In tests, prefer `.unwrap_or_else(|_| unreachable!())` for infallible cases

### Async Patterns
```rust
// Use async/await, not raw futures
pub async fn process(&self, data: Data) -> LoamSpineResult<Output> {
    let result = self.inner.process(data).await?;
    Ok(result)
}
```

### Cloning and Borrowing
- Avoid unnecessary clones
- Use references where ownership isn't needed
- Use `Arc` for shared state, not excessive cloning
- Consider `bytes::Bytes` for network buffers

### Constants (No Magic Numbers)
```rust
// ✅ CORRECT: Use named constants
use loam_spine_core::{KB, MB, SECONDS_PER_HOUR, SECONDS_PER_DAY};

let buffer_size = 64 * KB;
let loan_duration = 24 * SECONDS_PER_HOUR;

// ❌ WRONG: Magic numbers
let buffer_size = 65536;
let loan_duration = 86400;
```

Available constants:
- Size: `KB`, `MB`, `GB`
- Time: `SECONDS_PER_MINUTE`, `SECONDS_PER_HOUR`, `SECONDS_PER_DAY`, `SECONDS_PER_WEEK`, `SECONDS_PER_YEAR`

---

## 🧪 Testing

### Test Organization
```
crates/loam-spine-core/
├── src/
│   └── *.rs          # Unit tests at bottom of each file
├── tests/
│   ├── e2e.rs        # End-to-end integration tests
│   └── chaos.rs      # Fault injection and stress tests
└── benches/
    └── core_ops.rs   # Performance benchmarks

fuzz/
└── fuzz_targets/     # Fuzz testing targets
    ├── fuzz_entry_parsing.rs
    ├── fuzz_certificate.rs
    └── fuzz_spine_operations.rs
```

### Test Naming
```rust
#[test]
fn create_spine_with_valid_owner() { ... }

#[test]
fn create_spine_fails_with_sealed_parent() { ... }

#[tokio::test]
async fn concurrent_commits_succeed() { ... }
```

### Coverage Requirements
- New code: 90%+ line coverage
- Critical paths: 95%+ coverage
- Current project average: 90%+
- Run coverage: `cargo llvm-cov --workspace --summary-only`

---

## 🔄 Pull Request Process

### Before Submitting
1. ✅ All tests pass: `cargo test`
2. ✅ Clippy clean: `cargo clippy --all-targets -- -D warnings`
3. ✅ Formatted: `cargo fmt`
4. ✅ Docs build: `cargo doc --no-deps`
5. ✅ Coverage maintained or improved
6. ✅ Security check: `cargo deny check`

### PR Title Format
```
feat(spine): Add backup/restore functionality
fix(cert): Handle expired loan terms correctly
docs(api): Update JSON-RPC examples
test(proof): Add coverage for provenance verification
refactor(storage): Extract Sled-specific code
```

### PR Description Template
```markdown
## Summary
Brief description of changes.

## Changes
- Added X
- Fixed Y
- Refactored Z

## Testing
How was this tested?

## Coverage
Before: XX%
After: YY%
```

---

## 🏗️ Architecture Guidelines

### Adding New Features
1. **Spec First**: Update or create spec in `specs/`
2. **Types First**: Add types to `types.rs`
3. **Traits First**: Define traits before implementation
4. **Tests First**: Write tests before code (TDD encouraged)

### Mocks and Testing
- Mocks belong in `#[cfg(test)]` blocks only
- Use the `testing` feature for mock types needed by downstream crates
- Production code uses traits, tests provide mock implementations

### Capability Discovery
```rust
// ✅ CORRECT: Discover at runtime
let signer = registry.get_signer().await;
if let Some(s) = signer {
    s.sign_boxed(data).await?;
}

// ❌ WRONG: Hardcoded dependency
use beardog::Signer;
let signer = BearDogSigner::new();
```

---

## 📚 Documentation

### Doc Comments
```rust
/// Short summary in one line.
///
/// Longer description if needed. Can span multiple paragraphs.
///
/// # Examples
///
/// ```rust
/// let spine = Spine::new(owner)?;
/// ```
///
/// # Errors
///
/// Returns `LoamSpineError::SpineSealed` if the spine is sealed.
pub fn append(&mut self, entry: Entry) -> LoamSpineResult<EntryHash> { ... }
```

### Module Docs
- Every public module needs `//!` docs at the top
- Explain purpose, key types, and usage patterns
- Include code examples where helpful

---

## 🔐 Security

### Reporting Vulnerabilities
- Email: security@ecoprimals.org
- Do NOT open public issues for security bugs
- We respond within 48 hours

### Security Guidelines
- No `unsafe` code (enforced by `#![forbid(unsafe_code)]` workspace-wide)
- No deprecated crypto (use modern algorithms)
- Validate all input at boundaries
- Use constant-time comparison for secrets
- Run `cargo deny check` before submitting PRs

---

## 🎯 Good First Issues

Look for issues labeled `good-first-issue`:
- Adding tests for existing code
- Documentation improvements
- Clippy lint fixes
- Error message improvements

---

## 📊 Current Metrics

| Metric | Value |
|--------|-------|
| Version | 0.9.16 |
| Edition | 2024 |
| Tests | 1,395 (all concurrent, ~3s, zero flaky) |
| Coverage | 90.92% line / 89.09% branch / 92.92% region (llvm-cov) |
| Max File Size | 605 max production; 899 max test file |
| Clippy | pedantic + nursery + `missing_const_for_fn` (0 warnings) |
| Unsafe Code | 0 (`#![forbid(unsafe_code)]`) |
| Lint Exceptions | 4 `#[allow]` in production (2× tarpc macro, 2× feature-conditional async), tests all `#[expect(reason)]` |
| License | AGPL-3.0-or-later + ORC + CC-BY-SA-4.0 (scyBorg triple) |
| SPDX Headers | All 176 source files |
| ecoBin | Zero C dependencies (pure Rust) |
| cargo deny | advisories, bans, licenses, sources pass |
| UniBin | `loamspine server`, `capabilities`, `socket` subcommands |
| JSON-RPC Methods | 36 (semantic naming) |
| Mock isolation | All mocks cfg-gated out of production |
| Provenance Trio | Tested (rhizoCrypt + sweetGrass) |

---

## 📞 Getting Help

- **GitHub Issues**: For bugs and feature requests
- **Discussions**: For questions and ideas
- **Specs**: Read `specs/` for detailed design docs

---

*LoamSpine: Where memories become permanent.*
