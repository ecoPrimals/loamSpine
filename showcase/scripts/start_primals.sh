#!/bin/bash
# 🦴 LoamSpine Showcase - Start Phase 1 Primals
#
# This script starts the Phase 1 primal services needed for inter-primal demos.
#
# Usage:
#   cd /path/to/loamSpine/showcase/scripts
#   ./start_primals.sh
#
# Services Started:
#   - BearDog (port 8091) - Signing/Security
#   - NestGate (port 8092) - Storage
#   - Songbird (port 8081) - Discovery

set -e

BINS_DIR="$(cd "$(dirname "$0")/../../../bins" && pwd)"
LOGS_DIR="$(cd "$(dirname "$0")/.." && pwd)/logs"
PIDS_FILE="$LOGS_DIR/primal_pids.txt"

echo "═══════════════════════════════════════════════════════════════════════════════"
echo "  🦴 LOAMSPINE SHOWCASE - STARTING PHASE 1 PRIMALS"
echo "═══════════════════════════════════════════════════════════════════════════════"

# Create logs directory
mkdir -p "$LOGS_DIR"
> "$PIDS_FILE"

echo ""
echo "Bins directory: $BINS_DIR"
echo "Logs directory: $LOGS_DIR"
echo ""

# Check binaries exist
check_binary() {
    if [ ! -x "$BINS_DIR/$1" ]; then
        echo "⚠ Binary not found: $1"
        return 1
    fi
    return 0
}

# Start a primal in the background
start_primal() {
    local name="$1"
    local binary="$2"
    local args="$3"
    local port="$4"
    
    echo "→ Starting $name..."
    
    if ! check_binary "$binary"; then
        echo "  ✗ Skipping $name (binary not available)"
        return 0
    fi
    
    # Check if port is already in use
    if command -v lsof >/dev/null 2>&1; then
        if lsof -i ":$port" >/dev/null 2>&1; then
            echo "  ⚠ Port $port already in use, $name may already be running"
            return 0
        fi
    fi
    
    # Start the service
    "$BINS_DIR/$binary" $args > "$LOGS_DIR/${name}.log" 2>&1 &
    local pid=$!
    echo "$name:$pid" >> "$PIDS_FILE"
    
    # Wait a moment for startup
    sleep 1
    
    if kill -0 $pid 2>/dev/null; then
        echo "  ✓ $name started (PID: $pid, Port: $port)"
    else
        echo "  ✗ $name failed to start (check $LOGS_DIR/${name}.log)"
    fi
}

# Start BearDog
echo ""
echo "Phase 1 Primals"
echo "───────────────────────────────────────────────────────────────────────────────"

# BearDog - Check if it has a serve command
if "$BINS_DIR/beardog" --help 2>&1 | grep -q "serve\|status"; then
    start_primal "beardog" "beardog" "status" "8091"
else
    echo "→ BearDog: CLI tool (no background service needed)"
    echo "  Use: $BINS_DIR/beardog key generate --type ed25519"
fi

# NestGate
if "$BINS_DIR/nestgate" --help 2>&1 | grep -q "service"; then
    start_primal "nestgate" "nestgate" "service start --port 8092" "8092"
else
    echo "→ NestGate: Checking status..."
    "$BINS_DIR/nestgate" --help 2>&1 | head -5 || true
fi

# Songbird Rendezvous (discovery)
if [ -x "$BINS_DIR/songbird-rendezvous" ]; then
    start_primal "songbird" "songbird-rendezvous" "" "8081"
else
    echo "→ Songbird: Binary not available"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo "  STATUS"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""

# Show what's running
if [ -s "$PIDS_FILE" ]; then
    echo "Running primals (PIDs in $PIDS_FILE):"
    cat "$PIDS_FILE" | while read line; do
        name=$(echo "$line" | cut -d: -f1)
        pid=$(echo "$line" | cut -d: -f2)
        if kill -0 $pid 2>/dev/null; then
            echo "  ✓ $name (PID: $pid)"
        else
            echo "  ✗ $name (PID: $pid - not running)"
        fi
    done
else
    echo "No background primals started."
fi

echo ""
echo "Available CLI tools:"
echo "  • BearDog:   $BINS_DIR/beardog --help"
echo "  • NestGate:  $BINS_DIR/nestgate --help"
echo "  • Songbird:  $BINS_DIR/songbird-cli --help"
echo "  • Squirrel:  $BINS_DIR/squirrel (starts automatically)"
echo "  • ToadStool: $BINS_DIR/toadstool-cli --help"
echo ""
echo "To stop primals: ./stop_primals.sh"
echo ""

