#!/usr/bin/env bash
# Demo: Error Handling in RPC

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

readonly DEMO_NAME="error-handling"

cleanup() {
    log_info "Cleaning up..."
}

register_cleanup cleanup

main() {
    log_header "🦴 Error Handling in RPC"
    
    log_info "This demo shows:"
    log_info "  • Comprehensive error types"
    log_info "  • Error propagation via RPC"
    log_info "  • Client error handling"
    echo ""
    
    demo_pause
    
    log_step "Running error handling demo..."
    cd "${PROJECT_ROOT}"
    
    if cargo run --example demo_error_handling 2>&1 | tee "${LOGS_DIR}/${DEMO_NAME}.log"; then
        log_success "Error handling demo completed!"
    else
        log_error "Demo failed"
        exit 1
    fi
    
    echo ""
    
    log_section "Error Architecture"
    cat <<'EOF'

LoamSpine error handling:

Error Types (ApiError):
  • NotFound
  • InvalidInput
  • SpineSealed
  • InvalidState
  • StorageError
  • CertificateError
  • ProofError
  • SerializationError

Error Propagation:
  1. Core error → ApiError
  2. ApiError → RPC result
  3. RPC transport (tarpc/JSON-RPC)
  4. Client receives typed error

Benefits:
  ✅ Type-safe errors
  ✅ No panic in production
  ✅ Clear error messages
  ✅ Debuggable failures

EOF
    
    demo_pause
    
    generate_receipt "${DEMO_NAME}" "success" \
        "Demonstrated RPC error handling" \
        "Type-safe error propagation" \
        "Production-grade error management"
    
    log_success "Error handling demo complete! 🦴"
}

main "$@"

