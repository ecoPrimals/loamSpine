#!/usr/bin/env bash
# Demo: Concurrent RPC Operations

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

readonly DEMO_NAME="concurrent-rpc-ops"

cleanup() {
    log_info "Cleaning up..."
}

register_cleanup cleanup

main() {
    log_header "🦴 Concurrent RPC Operations"
    
    log_info "This demo shows:"
    log_info "  • Multiple simultaneous RPC calls"
    log_info "  • Tokio async concurrency"
    log_info "  • No blocking, full parallelism"
    echo ""
    
    demo_pause
    
    log_step "Running concurrent RPC demo..."
    cd "${PROJECT_ROOT}"
    
    if cargo run --example demo_concurrent_rpc 2>&1 | tee "${LOGS_DIR}/${DEMO_NAME}.log"; then
        log_success "Concurrent RPC demo completed!"
    else
        log_error "Demo failed"
        exit 1
    fi
    
    echo ""
    
    log_section "Concurrency Architecture"
    cat <<'EOF'

How LoamSpine handles concurrency:

1. Tokio Runtime
   • Native async/await
   • Work-stealing scheduler
   • No thread-per-request overhead

2. RPC Handlers
   • Each RPC call is a task
   • Non-blocking I/O
   • Parallel execution

3. Storage Layer
   • Arc<RwLock<>> for safe shared state
   • Read parallelism
   • Write serialization

Benefits:
  ✅ High throughput
  ✅ Low latency
  ✅ Efficient resource usage
  ✅ Production-grade concurrency

EOF
    
    demo_pause
    
    generate_receipt "${DEMO_NAME}" "success" \
        "Demonstrated concurrent RPC operations" \
        "Native async/await with Tokio" \
        "Production-grade concurrency"
    
    log_success "Concurrent RPC demo complete! 🦴"
}

main "$@"

