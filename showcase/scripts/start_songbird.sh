#!/usr/bin/env bash
# Start Songbird orchestrator for showcase demos
# This script manages the Songbird service lifecycle

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINS_DIR="$(cd "${SCRIPT_DIR}/../../bins" && pwd)"
SONGBIRD_BIN="${BINS_DIR}/songbird-orchestrator"
LOGS_DIR="${SCRIPT_DIR}/../logs"
PID_FILE="${LOGS_DIR}/songbird.pid"
LOG_FILE="${LOGS_DIR}/songbird.log"

# Port configuration
SONGBIRD_PORT="${SONGBIRD_PORT:-8082}"
SONGBIRD_HOST="${SONGBIRD_HOST:-127.0.0.1}"

mkdir -p "${LOGS_DIR}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ${NC} $*"; }
log_success() { echo -e "${GREEN}✅${NC} $*"; }
log_warning() { echo -e "${YELLOW}⚠${NC}  $*"; }
log_error() { echo -e "${RED}❌${NC} $*" >&2; }

# Check if Songbird binary exists
if [ ! -f "${SONGBIRD_BIN}" ]; then
    log_error "Songbird binary not found at: ${SONGBIRD_BIN}"
    log_info "Expected location: ${BINS_DIR}/songbird-orchestrator"
    log_info "Please ensure Phase 1 binaries are built and copied to ../bins/"
    exit 1
fi

if [ ! -x "${SONGBIRD_BIN}" ]; then
    log_warning "Songbird binary not executable, fixing..."
    chmod +x "${SONGBIRD_BIN}"
fi

# Check if already running
if [ -f "${PID_FILE}" ]; then
    OLD_PID=$(cat "${PID_FILE}")
    if kill -0 "$OLD_PID" 2>/dev/null; then
        log_warning "Songbird already running (PID: $OLD_PID)"
        log_info "Endpoint: http://${SONGBIRD_HOST}:${SONGBIRD_PORT}"
        exit 0
    else
        log_warning "Stale PID file found, removing..."
        rm "${PID_FILE}"
    fi
fi

# Start Songbird
log_info "Starting Songbird orchestrator..."
log_info "Port: ${SONGBIRD_PORT}"
log_info "Host: ${SONGBIRD_HOST}"
log_info "Logs: ${LOG_FILE}"

# Start in background
"${SONGBIRD_BIN}" \
    --port "${SONGBIRD_PORT}" \
    --host "${SONGBIRD_HOST}" \
    > "${LOG_FILE}" 2>&1 &

PID=$!
echo "$PID" > "${PID_FILE}"

# Wait for service to be ready
log_info "Waiting for Songbird to be ready..."
for i in {1..10}; do
    if curl -s -f "http://${SONGBIRD_HOST}:${SONGBIRD_PORT}/health" > /dev/null 2>&1; then
        log_success "Songbird started successfully!"
        log_success "PID: $PID"
        log_success "Endpoint: http://${SONGBIRD_HOST}:${SONGBIRD_PORT}"
        exit 0
    fi
    sleep 1
done

log_error "Songbird failed to start"
log_info "Check logs at: ${LOG_FILE}"
if kill -0 "$PID" 2>/dev/null; then
    log_warning "Process is running but not responding"
else
    log_error "Process died"
fi
exit 1

