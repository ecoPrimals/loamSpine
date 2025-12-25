#!/usr/bin/env bash
# Common utilities for LoamSpine showcase demos
# Source this file in demo scripts: source "$(dirname "$0")/../../scripts/common.sh"

# ============================================================================
# CONFIGURATION
# ============================================================================

# Project root (loamSpine directory)
COMMON_SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export SHOWCASE_ROOT="$(cd "${COMMON_SCRIPT_DIR}/.." && pwd)"
export PROJECT_ROOT="$(cd "${SHOWCASE_ROOT}/.." && pwd)"
export LOGS_DIR="${SHOWCASE_ROOT}/logs"
export RECEIPTS_DIR="${SHOWCASE_ROOT}/receipts"
export BINS_DIR="${PROJECT_ROOT}/../bins"

# Create directories
mkdir -p "${LOGS_DIR}" "${RECEIPTS_DIR}"

# ============================================================================
# COLORS
# ============================================================================

# Color codes
export RED='\033[0;31m'
export GREEN='\033[0;32m'
export YELLOW='\033[1;33m'
export BLUE='\033[0;34m'
export MAGENTA='\033[0;35m'
export CYAN='\033[0;36m'
export WHITE='\033[1;37m'
export BOLD='\033[1m'
export NC='\033[0m'  # No Color

# ============================================================================
# LOGGING FUNCTIONS
# ============================================================================

log_info() {
    echo -e "${BLUE}ℹ${NC} $*"
}

log_success() {
    echo -e "${GREEN}✅${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC}  $*"
}

log_error() {
    echo -e "${RED}❌${NC} $*" >&2
}

log_step() {
    echo -e "${CYAN}▶${NC}  ${BOLD}$*${NC}"
    echo ""
}

log_header() {
    echo ""
    echo "=================================================================="
    echo "  $*"
    echo "=================================================================="
    echo ""
}

log_section() {
    echo ""
    echo "──────────────────────────────────────────────────────────────────"
    echo "  $*"
    echo "──────────────────────────────────────────────────────────────────"
    echo ""
}

# ============================================================================
# DEMO UTILITIES
# ============================================================================

# Pause for user to read (unless FAST_MODE=1)
demo_pause() {
    if [ "${FAST_MODE:-0}" != "1" ]; then
        local msg="${1:-Press Enter to continue...}"
        echo ""
        read -r -p "$(echo -e "${CYAN}▶${NC}  ${msg}")" 
        echo ""
    fi
}

# Show code block with syntax highlighting (if bat is available)
show_code() {
    local file="$1"
    local title="${2:-Code}"
    
    echo ""
    echo -e "${BOLD}${title}:${NC}"
    echo "────────────────────────────────────────────────────────────"
    
    if command -v bat > /dev/null 2>&1; then
        bat --style=plain --color=always "$file"
    else
        cat "$file"
    fi
    
    echo "────────────────────────────────────────────────────────────"
    echo ""
}

# Generate receipt for demo execution
generate_receipt() {
    local demo_name="$1"
    local status="$2"
    shift 2
    local steps=("$@")
    
    local receipt_file="${RECEIPTS_DIR}/${demo_name}_$(date +%Y%m%d_%H%M%S).txt"
    
    cat > "$receipt_file" <<EOF
================================================================
LoamSpine Showcase Demo Receipt
================================================================

Demo:        $demo_name
Status:      $status
Timestamp:   $(date -Iseconds)
Hostname:    $(hostname)
User:        $(whoami)
Working Dir: $(pwd)

Steps Completed:
EOF
    
    for step in "${steps[@]}"; do
        echo "  ✓ $step" >> "$receipt_file"
    done
    
    cat >> "$receipt_file" <<EOF

Environment:
  PROJECT_ROOT: ${PROJECT_ROOT}
  LOGS_DIR:     ${LOGS_DIR}

================================================================
EOF
    
    log_success "Receipt generated: $receipt_file"
}

# ============================================================================
# SERVICE UTILITIES
# ============================================================================

# Check if a service is available
check_service() {
    local service_name="$1"
    local health_url="$2"
    local timeout="${3:-5}"
    
    if curl -s -f --max-time "$timeout" "$health_url" > /dev/null 2>&1; then
        log_success "$service_name is available at $health_url"
        return 0
    else
        log_warning "$service_name is NOT available at $health_url"
        return 1
    fi
}

# Check if binary exists in ../bins
check_binary() {
    local binary_name="$1"
    local binary_path="${BINS_DIR}/${binary_name}"
    
    if [ -f "$binary_path" ] && [ -x "$binary_path" ]; then
        log_success "Binary found: $binary_name"
        echo "$binary_path"
        return 0
    else
        log_warning "Binary NOT found: $binary_name"
        log_info "Expected location: $binary_path"
        return 1
    fi
}

# Start service from ../bins if not already running
start_binary_service() {
    local binary_name="$1"
    local port="$2"
    shift 2
    local args=("$@")
    
    # Check if binary exists
    local binary_path
    if ! binary_path=$(check_binary "$binary_name"); then
        log_error "Cannot start $binary_name: binary not found"
        return 1
    fi
    
    # Check if already running
    if lsof -i ":$port" > /dev/null 2>&1; then
        log_info "$binary_name appears to be running on port $port"
        return 0
    fi
    
    # Start service in background
    log_info "Starting $binary_name on port $port..."
    "$binary_path" "${args[@]}" > "${LOGS_DIR}/${binary_name}.log" 2>&1 &
    local pid=$!
    echo "$pid" > "${LOGS_DIR}/${binary_name}.pid"
    
    # Wait for service to be ready
    sleep 2
    
    if kill -0 "$pid" 2> /dev/null; then
        log_success "$binary_name started (PID: $pid)"
        return 0
    else
        log_error "$binary_name failed to start"
        return 1
    fi
}

# Stop service by PID file
stop_binary_service() {
    local binary_name="$1"
    local pid_file="${LOGS_DIR}/${binary_name}.pid"
    
    if [ -f "$pid_file" ]; then
        local pid
        pid=$(cat "$pid_file")
        if kill -0 "$pid" 2> /dev/null; then
            log_info "Stopping $binary_name (PID: $pid)..."
            kill "$pid"
            rm "$pid_file"
            log_success "$binary_name stopped"
        else
            log_warning "$binary_name PID $pid not running"
            rm "$pid_file"
        fi
    else
        log_warning "No PID file for $binary_name"
    fi
}

# ============================================================================
# DATA UTILITIES
# ============================================================================

# Create random DID for testing
random_did() {
    local prefix="${1:-did:example}"
    local random_id=$(cat /dev/urandom | tr -dc 'a-z0-9' | fold -w 16 | head -n 1)
    echo "${prefix}:${random_id}"
}

# Format timestamp
timestamp_iso() {
    date -Iseconds
}

timestamp_unix() {
    date +%s
}

# ============================================================================
# VALIDATION UTILITIES
# ============================================================================

# Validate JSON output
validate_json() {
    local json="$1"
    if echo "$json" | jq empty > /dev/null 2>&1; then
        return 0
    else
        log_error "Invalid JSON output"
        return 1
    fi
}

# Check if command exists
require_command() {
    local cmd="$1"
    local install_hint="${2:-}"
    
    if ! command -v "$cmd" > /dev/null 2>&1; then
        log_error "Required command not found: $cmd"
        [ -n "$install_hint" ] && log_info "Install with: $install_hint"
        return 1
    fi
    return 0
}

# ============================================================================
# CLEANUP
# ============================================================================

# Register cleanup function
register_cleanup() {
    local cleanup_func="$1"
    trap "$cleanup_func" EXIT INT TERM
}

# ============================================================================
# DEMO MODE
# ============================================================================

# Check if we're in demo mode (services not available)
is_demo_mode() {
    [ "${DEMO_MODE:-0}" = "1" ]
}

# Simulate operation in demo mode
simulate_operation() {
    local operation="$1"
    local duration="${2:-1}"
    
    if is_demo_mode; then
        log_warning "[DEMO MODE] Simulating: $operation"
        sleep "$duration"
        return 0
    else
        return 1
    fi
}

# ============================================================================
# INITIALIZATION
# ============================================================================

log_info "LoamSpine showcase utilities loaded"
log_info "Project root: ${PROJECT_ROOT}"
log_info "Bins directory: ${BINS_DIR}"
