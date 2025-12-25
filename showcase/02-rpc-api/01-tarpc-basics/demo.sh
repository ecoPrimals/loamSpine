#!/usr/bin/env bash
# Demo: tarpc Basics - Pure Rust Binary RPC

set -euo pipefail

# Load common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

readonly DEMO_NAME="tarpc-basics"

cleanup() {
    log_info "Cleaning up..."
}

register_cleanup cleanup

main() {
    log_header "🦴 tarpc Basics — Pure Rust Binary RPC"
    
    log_info "This demo shows:"
    log_info "  • Pure Rust tarpc (no gRPC, no protobuf!)"
    log_info "  • Binary protocol for primal-to-primal"
    log_info "  • High performance RPC"
    log_info "  • 18 RPC methods available"
    echo ""
    
    demo_pause
    
    log_step "Running tarpc demo..."
    cd "${PROJECT_ROOT}"
    
    if cargo run -p loam-spine-api --example demo_rpc_service 2>&1 | tee "${LOGS_DIR}/${DEMO_NAME}.log"; then
        log_success "tarpc demo completed!"
    else
        log_error "Demo failed"
        exit 1
    fi
    
    echo ""
    
    log_section "tarpc Benefits"
    cat <<'EOF'

Why tarpc over gRPC:
  ✅ Pure Rust (no C++ toolchain)
  ✅ Native serde serialization  
  ✅ No protobuf complexity
  ✅ Toolchain sovereignty
  ✅ Zero foreign dependencies
  ✅ Native async/await

Performance:
  • Binary protocol (compact)
  • Zero-copy where possible
  • Native Rust types
  • Direct primal-to-primal

EOF
    
    demo_pause
    
    generate_receipt "${DEMO_NAME}" "success" \
        "Demonstrated tarpc pure Rust RPC" \
        "18 RPC methods available" \
        "No gRPC, no protobuf, pure Rust!"
    
    log_success "tarpc demo complete! 🦴"
}

main "$@"

