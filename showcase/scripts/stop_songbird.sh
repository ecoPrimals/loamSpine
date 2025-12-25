#!/usr/bin/env bash
# Stop Songbird orchestrator

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOGS_DIR="${SCRIPT_DIR}/../logs"
PID_FILE="${LOGS_DIR}/songbird.pid"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ${NC} $*"; }
log_success() { echo -e "${GREEN}✅${NC} $*"; }
log_warning() { echo -e "${YELLOW}⚠${NC}  $*"; }

if [ ! -f "${PID_FILE}" ]; then
    log_warning "No PID file found - Songbird may not be running"
    exit 0
fi

PID=$(cat "${PID_FILE}")

if kill -0 "$PID" 2>/dev/null; then
    log_info "Stopping Songbird (PID: $PID)..."
    kill "$PID"
    
    # Wait for graceful shutdown
    for i in {1..5}; do
        if ! kill -0 "$PID" 2>/dev/null; then
            break
        fi
        sleep 1
    done
    
    # Force kill if still running
    if kill -0 "$PID" 2>/dev/null; then
        log_warning "Forcing shutdown..."
        kill -9 "$PID" 2>/dev/null || true
    fi
    
    log_success "Songbird stopped"
else
    log_warning "Songbird not running (stale PID)"
fi

rm "${PID_FILE}"

