#!/bin/bash
# LoamSpine — Production Readiness Verification
set -euo pipefail

echo "LoamSpine — Production Readiness Verification"
echo "==============================================="
echo ""

echo "Building..."
cargo build --workspace --all-features --quiet 2>&1
echo "  Build PASSED"
echo ""

echo "Testing..."
cargo test --workspace --all-features --quiet 2>&1
echo "  Tests PASSED"
echo ""

echo "Linting..."
cargo clippy --workspace --all-features --all-targets -- -D warnings --quiet 2>&1
echo "  Clippy PASSED (zero warnings)"
echo ""

echo "Formatting..."
cargo fmt --all -- --check --quiet 2>&1
echo "  Format PASSED"
echo ""

echo "Documentation..."
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --quiet 2>&1
echo "  Docs PASSED (zero warnings)"
echo ""

echo "Dependency audit..."
if command -v cargo-deny &>/dev/null; then
    cargo deny check --quiet 2>&1
    echo "  Dependency audit PASSED (advisories, bans, licenses, sources)"
else
    echo "  (cargo-deny not installed, skipping)"
fi
echo ""

echo "Coverage..."
if command -v cargo-llvm-cov &>/dev/null; then
    COVERAGE=$(cargo llvm-cov --workspace --all-features --summary-only 2>&1 | grep "TOTAL" | awk '{for(i=NF;i>0;i--) if ($i ~ /%/) {print $i; break}}')
    echo "  Line coverage: $COVERAGE"
else
    echo "  (cargo-llvm-cov not installed, skipping)"
fi
echo ""

echo "==============================================="
echo "ALL CHECKS PASSED"
echo "==============================================="
