#!/bin/bash
# Stop LoamSpine service

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SHOWCASE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PID_FILE="${SHOWCASE_ROOT}/logs/loamspine-service.pid"

if [ ! -f "${PID_FILE}" ]; then
    echo "⚠️  LoamSpine service is not running (no PID file)"
    exit 0
fi

PID=$(cat "${PID_FILE}")

if ! kill -0 "${PID}" 2>/dev/null; then
    echo "⚠️  LoamSpine service not running (stale PID file)"
    rm -f "${PID_FILE}"
    exit 0
fi

echo "🛑 Stopping LoamSpine service (PID: ${PID})..."
kill "${PID}"

# Wait for graceful shutdown
for i in {1..10}; do
    if ! kill -0 "${PID}" 2>/dev/null; then
        echo "✅ LoamSpine service stopped"
        rm -f "${PID_FILE}"
        exit 0
    fi
    sleep 1
done

# Force kill if still running
echo "⚠️  Force killing service..."
kill -9 "${PID}" 2>/dev/null || true
rm -f "${PID_FILE}"
echo "✅ LoamSpine service stopped (forced)"

