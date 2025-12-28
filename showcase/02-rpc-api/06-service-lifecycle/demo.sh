#!/usr/bin/env bash
# Demo: Service Lifecycle Management
# Complete service management: start, configure, monitor, shutdown

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SHOWCASE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
PROJECT_ROOT="$(cd "${SHOWCASE_ROOT}/.." && pwd)"
BINS_DIR="${PROJECT_ROOT}/../../primalBins"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Configuration
LOAMSPINE_SERVICE_BIN="${BINS_DIR}/loamspine-service"
JSONRPC_PORT="8085"
TARPC_PORT="9005"
JSONRPC_URL="http://localhost:${JSONRPC_PORT}"

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║${NC}  ${GREEN}🦴 LoamSpine Service Lifecycle Management${NC}             ${BLUE}║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "🎯 Complete Service Management Demonstration"
echo ""
echo "This demo covers:"
echo "  • Service startup and configuration"
echo "  • Health verification"
echo "  • Runtime operations"
echo "  • Monitoring and metrics"
echo "  • Graceful shutdown"
echo ""

# Check binary
if [ ! -f "${LOAMSPINE_SERVICE_BIN}" ]; then
    echo -e "${RED}❌ loamspine-service binary not found${NC}"
    exit 1
fi

echo -e "${YELLOW}✓ Binary located: ${LOAMSPINE_SERVICE_BIN}${NC}"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Stage 1: Pre-Start Checks
echo -e "${PURPLE}STAGE 1: Pre-Start Checks${NC}"
echo ""

echo "   Checking port availability..."
if lsof -Pi :${JSONRPC_PORT} -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "   ${YELLOW}⚠ Port ${JSONRPC_PORT} in use${NC}"
    echo "   Attempting to clear..."
    lsof -ti:${JSONRPC_PORT} | xargs kill -9 2>/dev/null || true
    sleep 1
fi

if ! lsof -Pi :${JSONRPC_PORT} -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "   ${GREEN}✓ Port ${JSONRPC_PORT} available${NC}"
else
    echo -e "   ${RED}✗ Port ${JSONRPC_PORT} still in use${NC}"
    exit 1
fi

echo ""
echo "   Preparing configuration..."
cat > /tmp/loamspine-lifecycle.env << EOF
JSONRPC_PORT=${JSONRPC_PORT}
TARPC_PORT=${TARPC_PORT}
LOG_LEVEL=info
SERVICE_NAME=loamspine-lifecycle-demo
EOF

echo -e "   ${GREEN}✓ Configuration prepared${NC}"
echo ""

# Stage 2: Service Start
echo -e "${PURPLE}STAGE 2: Service Startup${NC}"
echo ""

echo "   Starting loamspine-service..."
echo "   • JSON-RPC port: ${JSONRPC_PORT}"
echo "   • TARPC port: ${TARPC_PORT}"
echo ""

"${LOAMSPINE_SERVICE_BIN}" \
    --jsonrpc-port "${JSONRPC_PORT}" \
    --tarpc-port "${TARPC_PORT}" \
    &> /tmp/loamspine-lifecycle.log &

SERVICE_PID=$!
echo "${SERVICE_PID}" > /tmp/loamspine-lifecycle.pid

echo "   Process ID: ${SERVICE_PID}"
echo "   Log file: /tmp/loamspine-lifecycle.log"
echo ""

echo "   Waiting for service to become ready..."
READY=false
for i in {1..30}; do
    if curl -s "${JSONRPC_URL}/health" > /dev/null 2>&1; then
        READY=true
        break
    fi
    echo -n "."
    sleep 0.5
done
echo ""

if [ "${READY}" = "true" ]; then
    echo -e "   ${GREEN}✅ Service started successfully${NC}"
else
    echo -e "   ${RED}❌ Service failed to start${NC}"
    cat /tmp/loamspine-lifecycle.log
    exit 1
fi

echo ""

# Stage 3: Health Verification
echo -e "${PURPLE}STAGE 3: Health Verification${NC}"
echo ""

echo "   Verifying service health..."
HEALTH_RESPONSE=$(curl -s "${JSONRPC_URL}/health")
echo "   Health: ${HEALTH_RESPONSE}"

if echo "${HEALTH_RESPONSE}" | grep -q "ok"; then
    echo -e "   ${GREEN}✅ Health check passed${NC}"
else
    echo -e "   ${RED}❌ Health check failed${NC}"
    exit 1
fi

echo ""
echo "   Checking process status..."
if kill -0 "${SERVICE_PID}" 2>/dev/null; then
    echo -e "   ${GREEN}✓ Process running (PID: ${SERVICE_PID})${NC}"
else
    echo -e "   ${RED}✗ Process not running${NC}"
    exit 1
fi

echo ""

# Stage 4: Runtime Operations
echo -e "${PURPLE}STAGE 4: Runtime Operations${NC}"
echo ""

echo "   Performing operations to generate load..."

# Create spines
SPINES_CREATED=0
for i in {1..5}; do
    RESPONSE=$(curl -s -X POST "${JSONRPC_URL}" \
        -H "Content-Type: application/json" \
        -d "{
            \"jsonrpc\": \"2.0\",
            \"id\": ${i},
            \"method\": \"create_spine\",
            \"params\": {
                \"owner_did\": \"did:key:z6MkLifecycle${i}\",
                \"name\": \"Lifecycle Spine ${i}\"
            }
        }")
    
    if echo "${RESPONSE}" | grep -q "result"; then
        SPINES_CREATED=$((SPINES_CREATED + 1))
    fi
done

echo "   Created ${SPINES_CREATED}/5 spines"

if [ ${SPINES_CREATED} -eq 5 ]; then
    echo -e "   ${GREEN}✅ All operations successful${NC}"
else
    echo -e "   ${YELLOW}⚠ Some operations failed${NC}"
fi

echo ""

# Stage 5: Monitoring
echo -e "${PURPLE}STAGE 5: Runtime Monitoring${NC}"
echo ""

echo "   Collecting service metrics..."
echo ""

# Memory
if [ -f /proc/${SERVICE_PID}/status ]; then
    MEM_KB=$(grep VmRSS /proc/${SERVICE_PID}/status | awk '{print $2}')
    MEM_MB=$((MEM_KB / 1024))
    echo "   Memory: ${MEM_MB} MB"
fi

# Uptime
if [ -f /proc/${SERVICE_PID}/stat ]; then
    START_TIME=$(awk '{print $22}' /proc/${SERVICE_PID}/stat)
    SYSTEM_UPTIME=$(awk '{print $1}' /proc/uptime)
    CLK_TCK=$(getconf CLK_TCK 2>/dev/null || echo 100)
    UPTIME_SECONDS=$(echo "scale=2; ${SYSTEM_UPTIME} - (${START_TIME} / ${CLK_TCK})" | bc 2>/dev/null || echo "N/A")
    if [ "${UPTIME_SECONDS}" != "N/A" ]; then
        echo "   Uptime: ${UPTIME_SECONDS}s"
    fi
fi

# Connection test
echo ""
echo "   Testing endpoints..."
if curl -s "${JSONRPC_URL}/health" > /dev/null; then
    echo -e "   ${GREEN}✓ JSON-RPC endpoint responsive${NC}"
else
    echo -e "   ${RED}✗ JSON-RPC endpoint not responding${NC}"
fi

echo ""

# Stage 6: Log Analysis
echo -e "${PURPLE}STAGE 6: Log Analysis${NC}"
echo ""

echo "   Analyzing service logs..."
if [ -f /tmp/loamspine-lifecycle.log ]; then
    LOG_LINES=$(wc -l < /tmp/loamspine-lifecycle.log)
    echo "   Log lines: ${LOG_LINES}"
    
    if [ ${LOG_LINES} -gt 0 ]; then
        echo ""
        echo "   Recent log entries:"
        tail -5 /tmp/loamspine-lifecycle.log | sed 's/^/   /'
    fi
else
    echo "   No log file found"
fi

echo ""

# Stage 7: Graceful Shutdown
echo -e "${PURPLE}STAGE 7: Graceful Shutdown${NC}"
echo ""

echo "   Initiating graceful shutdown..."
echo "   Sending SIGTERM to PID ${SERVICE_PID}..."

if kill -TERM "${SERVICE_PID}" 2>/dev/null; then
    echo "   Waiting for process to terminate..."
    
    SHUTDOWN_TIMEOUT=10
    for i in $(seq 1 ${SHUTDOWN_TIMEOUT}); do
        if ! kill -0 "${SERVICE_PID}" 2>/dev/null; then
            echo -e "   ${GREEN}✅ Service shut down gracefully (${i}s)${NC}"
            break
        fi
        echo -n "."
        sleep 1
    done
    echo ""
    
    # Force kill if still running
    if kill -0 "${SERVICE_PID}" 2>/dev/null; then
        echo -e "   ${YELLOW}⚠ Forcing shutdown after ${SHUTDOWN_TIMEOUT}s timeout${NC}"
        kill -9 "${SERVICE_PID}" 2>/dev/null || true
        sleep 1
    fi
else
    echo -e "   ${RED}❌ Failed to signal process${NC}"
fi

# Verify shutdown
if ! kill -0 "${SERVICE_PID}" 2>/dev/null; then
    echo -e "   ${GREEN}✓ Process terminated${NC}"
else
    echo -e "   ${RED}✗ Process still running${NC}"
fi

echo ""

# Stage 8: Cleanup
echo -e "${PURPLE}STAGE 8: Cleanup${NC}"
echo ""

echo "   Cleaning up resources..."

rm -f /tmp/loamspine-lifecycle.pid
rm -f /tmp/loamspine-lifecycle.env

if [ -f /tmp/loamspine-lifecycle.log ]; then
    echo "   Log archived to: /tmp/loamspine-lifecycle.log"
    echo "   (Keeping for review)"
fi

echo -e "   ${GREEN}✓ Cleanup complete${NC}"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${GREEN}✅ Service Lifecycle Demo Complete!${NC}"
echo ""
echo "🎓 Lifecycle Stages Demonstrated:"
echo "   1. Pre-start checks (port availability)"
echo "   2. Service startup (with configuration)"
echo "   3. Health verification (endpoints ready)"
echo "   4. Runtime operations (normal workload)"
echo "   5. Monitoring (metrics collection)"
echo "   6. Log analysis (debugging insights)"
echo "   7. Graceful shutdown (clean termination)"
echo "   8. Cleanup (resource management)"
echo ""
echo "💡 Production Best Practices:"
echo ""
echo "   ${PURPLE}Startup${NC}"
echo "   • Verify port availability before starting"
echo "   • Use configuration files/environment vars"
echo "   • Wait for health checks before routing traffic"
echo "   • Log startup sequence for debugging"
echo ""
echo "   ${PURPLE}Runtime${NC}"
echo "   • Continuous health monitoring"
echo "   • Collect and export metrics"
echo "   • Rotate logs to prevent disk fill"
echo "   • Handle signals gracefully (SIGTERM, SIGHUP)"
echo ""
echo "   ${PURPLE}Shutdown${NC}"
echo "   • Drain connections before stopping"
echo "   • Save state if needed"
echo "   • Use graceful shutdown with timeout"
echo "   • Archive logs for post-mortem"
echo ""
echo "🔧 Integration Points:"
echo "   • systemd (Linux service management)"
echo "   • Docker (containerized deployment)"
echo "   • Kubernetes (orchestration)"
echo "   • Supervisor (process control)"
echo "   • PM2 (Node.js-style process management)"
echo ""
echo "🦴 LoamSpine: Production-ready service management!"
echo ""

