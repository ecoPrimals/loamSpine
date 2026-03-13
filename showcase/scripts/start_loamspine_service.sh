#!/bin/bash
# Start LoamSpine service using real binary from primalBins

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SHOWCASE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PROJECT_ROOT="$(cd "${SHOWCASE_ROOT}/.." && pwd)"
BINS_DIR="${PROJECT_ROOT}/../../primalBins"

# Configuration
LOAM_BIN="${BINS_DIR}/loamspine"
TARPC_PORT="${TARPC_PORT:-9001}"
JSONRPC_PORT="${JSONRPC_PORT:-8080}"
LOG_FILE="${SHOWCASE_ROOT}/logs/loamspine-service.log"
PID_FILE="${SHOWCASE_ROOT}/logs/loamspine-service.pid"

# Create logs directory
mkdir -p "${SHOWCASE_ROOT}/logs"

# Check if binary exists
if [ ! -f "${LOAM_BIN}" ]; then
    echo "❌ LoamSpine binary not found at: ${LOAM_BIN}"
    echo "   Please ensure loamspine is built in primalBins/"
    exit 1
fi

# Check if already running
if [ -f "${PID_FILE}" ]; then
    OLD_PID=$(cat "${PID_FILE}")
    if kill -0 "${OLD_PID}" 2>/dev/null; then
        echo "⚠️  LoamSpine service already running (PID: ${OLD_PID})"
        echo "   Use stop_loamspine_service.sh to stop it first"
        exit 1
    else
        rm -f "${PID_FILE}"
    fi
fi

# Start service
echo "🚀 Starting LoamSpine service..."
echo "   Binary: ${LOAM_BIN}"
echo "   tarpc port: ${TARPC_PORT}"
echo "   JSON-RPC port: ${JSONRPC_PORT}"
echo "   Logs: ${LOG_FILE}"

"${LOAM_BIN}" server \
  --tarpc-port "${TARPC_PORT}" \
  --jsonrpc-port "${JSONRPC_PORT}" \
  > "${LOG_FILE}" 2>&1 &

SERVICE_PID=$!
echo "${SERVICE_PID}" > "${PID_FILE}"

# Wait for service to be ready
echo -n "   Waiting for service"
for i in {1..30}; do
    if curl -s "http://localhost:${JSONRPC_PORT}/health" > /dev/null 2>&1; then
        echo ""
        echo "✅ LoamSpine service started successfully!"
        echo "   PID: ${SERVICE_PID}"
        echo "   tarpc: localhost:${TARPC_PORT}"
        echo "   JSON-RPC: http://localhost:${JSONRPC_PORT}"
        echo "   Health: http://localhost:${JSONRPC_PORT}/health"
        exit 0
    fi
    echo -n "."
    sleep 1
done

echo ""
echo "❌ Service failed to start within 30 seconds"
echo "   Check logs: ${LOG_FILE}"
kill "${SERVICE_PID}" 2>/dev/null || true
rm -f "${PID_FILE}"
exit 1

