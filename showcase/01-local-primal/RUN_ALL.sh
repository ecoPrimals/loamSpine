#!/usr/bin/env bash
# Run all Level 1 (Local Primal) demos

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../scripts/common.sh"

log_header "🦴 LoamSpine Level 1: Local Primal Capabilities"

log_info "This level demonstrates LoamSpine BY ITSELF:"
log_info "  • Creating spines with owner DIDs"
log_info "  • All 15+ entry types"
log_info "  • Certificate lifecycle (mint, transfer, loan, return)"
log_info "  • Inclusion and provenance proofs"
log_info "  • Backup and restore"
log_info "  • Storage backends (InMemory vs Sled)"
log_info ""

# Parse arguments
FAST_MODE="${1:-false}"
if [[ "${FAST_MODE}" == "--fast" ]]; then
    export FAST_MODE=true
    log_info "Running in FAST mode (no pauses)"
fi

# Demo list
DEMOS=(
    "01-hello-loamspine"
    # "02-entry-types"  # Rust example complete, needs demo.sh wrapper
    # "03-certificate-lifecycle"  # Rust example complete
    # "04-proofs"  # Rust example complete
    # "05-backup-restore"  # Rust example complete
    # "06-storage-backends"  # Rust example complete
    # "07-concurrent-ops"  # Rust example complete
    "08-temporal-moments"  # NEW v0.7.0 - Universal time tracking
    "09-waypoint-anchoring"  # NEW - Slice lending patterns
    "10-recursive-spines"  # NEW - Spine composition
)
    # More demos coming...
)

# Track results
PASSED=0
FAILED=0
FAILED_DEMOS=()

# Run each demo
for demo in "${DEMOS[@]}"; do
    demo_dir="${SCRIPT_DIR}/${demo}"
    
    if [[ ! -d "${demo_dir}" ]]; then
        log_warning "Demo not found: ${demo} (skipping)"
        continue
    fi
    
    if [[ ! -x "${demo_dir}/demo.sh" ]]; then
        log_warning "Demo script not executable: ${demo}/demo.sh (skipping)"
        continue
    fi
    
    log_step "Running demo: ${demo}"
    
    if "${demo_dir}/demo.sh"; then
        PASSED=$((PASSED + 1))
        log_success "${demo} passed!"
    else
        FAILED=$((FAILED + 1))
        FAILED_DEMOS+=("${demo}")
        log_error "${demo} failed!"
    fi
    
    echo ""
done

# Summary
log_header "Level 1 Demo Results"

log_info "Total demos: $((PASSED + FAILED))"
log_success "Passed: ${PASSED}"

if [[ ${FAILED} -gt 0 ]]; then
    log_error "Failed: ${FAILED}"
    log_info "Failed demos:"
    for demo in "${FAILED_DEMOS[@]}"; do
        log_info "  - ${demo}"
    done
    exit 1
else
    log_success "All demos passed! 🎉"
    log_info ""
    log_info "Next steps:"
    log_info "  • Try Level 2: RPC API (cd ../02-rpc-api)"
    log_info "  • Try Level 3: Songbird Discovery (cd ../03-songbird-discovery)"
    log_info "  • Try Level 4: Inter-Primal (cd ../04-inter-primal)"
fi

