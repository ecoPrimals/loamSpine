#!/usr/bin/env bash
# 🦴 LoamSpine Showcase — QUICK START
# Master script to run all showcase demos in sequence

set -euo pipefail

# Colors
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly BOLD='\033[1m'
readonly NC='\033[0m'

# Directories
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# ============================================================================
# UTILITIES
# ============================================================================

log_header() {
    echo -e "\n${BOLD}$*${NC}"
    echo "=================================================================="
}

log_success() {
    echo -e "${GREEN}✅${NC} $*"
}

log_info() {
    echo -e "${BLUE}ℹ${NC}  $*"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC}  $*"
}

log_error() {
    echo -e "${RED}❌${NC} $*"
}

pause_if_interactive() {
    if [ -z "${FAST_MODE:-}" ]; then
        echo ""
        read -p "Press Enter to continue..."
        echo ""
    fi
}

# ============================================================================
# MAIN
# ============================================================================

main() {
    log_header "🦴 LoamSpine Showcase — QUICK START"
    
    log_info "This master script runs all showcase demos in sequence:"
    log_info "  • Level 1: Local Primal (7 demos)"
    log_info "  • Level 3: Songbird Discovery (1 demo)"
    log_info "  • Level 4: Inter-Primal (1 demo)"
    echo ""
    log_info "Total: 9 working demos"
    echo ""
    
    if [ -z "${FAST_MODE:-}" ]; then
        log_warning "Running in INTERACTIVE mode (pauses between demos)"
        log_info "Set FAST_MODE=1 to run without pauses"
        echo ""
        read -p "Press Enter to start, or Ctrl+C to cancel..."
    else
        log_info "Running in FAST mode (no pauses)"
    fi
    
    echo ""
    
    # Track results
    local total_demos=0
    local passed_demos=0
    local failed_demos=0
    local failed_list=()
    
    # ========================================================================
    # LEVEL 0: LOCAL PRIMAL DEMOS
    # ========================================================================
    
    log_header "📦 Level 1: Local Primal Capabilities"
    
    local level0_demos=(
        "01-hello-loamspine"
        "02-entry-types"
        "03-certificate-lifecycle"
        "04-proofs"
        "05-backup-restore"
        "06-storage-backends"
        "07-concurrent-ops"
    )
    
    for demo in "${level0_demos[@]}"; do
        ((total_demos++))
        
        log_info "Running demo: ${demo}..."
        
        if cd "${SCRIPT_DIR}/01-local-primal/${demo}" && FAST_MODE=1 ./demo.sh >/dev/null 2>&1; then
            log_success "Demo ${demo} passed"
            ((passed_demos++))
        else
            log_error "Demo ${demo} FAILED"
            ((failed_demos++))
            failed_list+=("01-local-primal/${demo}")
        fi
        
        pause_if_interactive
    done
    
    # ========================================================================
    # LEVEL 2: INTER-PRIMAL INTEGRATION
    # ========================================================================
    
    log_header "🔗 Level 3: Inter-Primal Integration"
    
    ((total_demos++))
    log_info "Running demo: full-ecosystem..."
    
    if cd "${SCRIPT_DIR}/03-inter-primal/05-full-ecosystem" && FAST_MODE=1 ./demo.sh >/dev/null 2>&1; then
        log_success "Demo full-ecosystem passed"
        ((passed_demos++))
    else
        log_error "Demo full-ecosystem FAILED"
        ((failed_demos++))
        failed_list+=("03-inter-primal/05-full-ecosystem")
    fi
    
    # ========================================================================
    # SUMMARY
    # ========================================================================
    
    echo ""
    log_header "📊 SHOWCASE SUMMARY"
    
    log_info "Total demos:  ${total_demos}"
    log_success "Passed:       ${passed_demos}"
    
    if [ ${failed_demos} -gt 0 ]; then
        log_error "Failed:       ${failed_demos}"
        echo ""
        log_warning "Failed demos:"
        for failed in "${failed_list[@]}"; do
            log_error "  ${failed}"
        done
        echo ""
        exit 1
    else
        log_success "All demos passed! 🎉"
        echo ""
        log_info "Receipts generated in: ${SCRIPT_DIR}/receipts/"
        log_info "Logs available in: ${SCRIPT_DIR}/logs/"
        echo ""
        log_success "LoamSpine showcase validation complete! 🦴"
        echo ""
    fi
}

# Run main
main "$@"
