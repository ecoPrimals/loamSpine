#!/usr/bin/env bash
# Demo: JSON-RPC Basics - Language-Agnostic API

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

readonly DEMO_NAME="jsonrpc-basics"

cleanup() {
    log_info "Cleaning up..."
}

register_cleanup cleanup

main() {
    log_header "🦴 JSON-RPC 2.0 Basics — Language-Agnostic API"
    
    log_info "This demo shows:"
    log_info "  • JSON-RPC 2.0 protocol"
    log_info "  • Language-agnostic access"
    log_info "  • Same API as tarpc"
    log_info "  • External client support"
    echo ""
    
    demo_pause
    
    log_step "Running JSON-RPC demo..."
    cd "${PROJECT_ROOT}"
    
    if cargo run -p loam-spine-api --example demo_jsonrpc_service 2>&1 | tee "${LOGS_DIR}/${DEMO_NAME}.log"; then
        log_success "JSON-RPC demo completed!"
    else
        log_error "Demo failed"
        exit 1
    fi
    
    echo ""
    
    log_section "Dual Protocol Architecture"
    cat <<'EOF'

LoamSpine exposes TWO protocols:

1. tarpc (Binary)
   • For primal-to-primal communication
   • Pure Rust, high performance
   • Native types, zero-copy

2. JSON-RPC 2.0 (Text)
   • For external clients (Python, JS, etc.)
   • Language-agnostic
   • Standard protocol

Same API, different transports!

EOF
    
    demo_pause
    
    generate_receipt "${DEMO_NAME}" "success" \
        "Demonstrated JSON-RPC 2.0 API" \
        "Language-agnostic access" \
        "Same 18 methods as tarpc"
    
    log_success "JSON-RPC demo complete! 🦴"
}

main "$@"

