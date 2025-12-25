#!/bin/bash
# 🦴 LoamSpine Showcase - Stop Phase 1 Primals
#
# This script stops any Phase 1 primal services started by start_primals.sh

LOGS_DIR="$(cd "$(dirname "$0")/.." && pwd)/logs"
PIDS_FILE="$LOGS_DIR/primal_pids.txt"

echo "═══════════════════════════════════════════════════════════════════════════════"
echo "  🦴 LOAMSPINE SHOWCASE - STOPPING PHASE 1 PRIMALS"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""

if [ ! -f "$PIDS_FILE" ]; then
    echo "No primal PIDs file found. Nothing to stop."
    exit 0
fi

while read line; do
    name=$(echo "$line" | cut -d: -f1)
    pid=$(echo "$line" | cut -d: -f2)
    
    if kill -0 $pid 2>/dev/null; then
        echo "→ Stopping $name (PID: $pid)..."
        kill $pid 2>/dev/null || true
        sleep 0.5
        if kill -0 $pid 2>/dev/null; then
            echo "  Force killing..."
            kill -9 $pid 2>/dev/null || true
        fi
        echo "  ✓ Stopped"
    else
        echo "→ $name (PID: $pid) already stopped"
    fi
done < "$PIDS_FILE"

rm -f "$PIDS_FILE"
echo ""
echo "All primals stopped."

