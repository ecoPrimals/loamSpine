#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"
DEMO_NAME="$(basename "$SCRIPT_DIR")"
EXAMPLE_NAME="demo_${DEMO_NAME#??-}"
EXAMPLE_NAME="${EXAMPLE_NAME//-/_}"

main() {
    log_header "🦴 ${DEMO_NAME} Demo"
    log_info "Running example: ${EXAMPLE_NAME}"
    echo ""
    cd "${PROJECT_ROOT}"
    if cargo run --example "${EXAMPLE_NAME}" 2>&1 | tee "${LOGS_DIR}/${DEMO_NAME}.log"; then
        log_success "Demo completed!"
        generate_receipt "${DEMO_NAME}" "success" "Ran ${EXAMPLE_NAME} example"
    else
        log_error "Demo failed - check logs"
        exit 1
    fi
}
main "$@"
