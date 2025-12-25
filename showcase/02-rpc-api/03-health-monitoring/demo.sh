#!/usr/bin/env bash
# Demo: Health Monitoring via RPC

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

readonly DEMO_NAME="health-monitoring"

cleanup() {
    log_info "Cleaning up..."
}

register_cleanup cleanup

main() {
    log_header "🦴 Health Monitoring via RPC"
    
    log_info "This demo shows:"
    log_info "  • health_check RPC method"
    log_info "  • Service health reporting"
    log_info "  • Ready for monitoring systems"
    echo ""
    
    demo_pause
    
    log_step "Running health demo..."
    cd "${PROJECT_ROOT}"
    
    if cargo run --example demo_health_check 2>&1 | tee "${LOGS_DIR}/${DEMO_NAME}.log"; then
        log_success "Health demo completed!"
    else
        log_error "Demo failed"
        exit 1
    fi
    
    echo ""
    
    log_section "Production Health Checks"
    cat <<'EOF'

Health check integration:

Kubernetes:
  livenessProbe:
    exec:
      command: ["curl", "localhost:8080/health"]
    periodSeconds: 10

Docker Compose:
  healthcheck:
    test: ["CMD", "curl", "-f", "localhost:8080/health"]
    interval: 30s
    timeout: 3s

Prometheus:
  - job_name: 'loamspine'
    metrics_path: '/metrics'
    static_configs:
      - targets: ['localhost:8080']

EOF
    
    demo_pause
    
    generate_receipt "${DEMO_NAME}" "success" \
        "Demonstrated health monitoring" \
        "Production-ready health checks" \
        "Integration with monitoring systems"
    
    log_success "Health demo complete! 🦴"
}

main "$@"

