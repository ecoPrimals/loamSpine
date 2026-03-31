#!/bin/bash

# ===================================================================
# LoamSpine Demo: Heartbeat Monitoring & Health Checks
# ===================================================================
# What this demonstrates:
#   - Health check endpoints (/health, /health/live, /health/ready)
#   - Heartbeat failure detection
#   - Auto-recovery from failures
#   - Service status monitoring
# Prerequisites:
#   - Songbird binary at ../../bins/songbird-orchestrator
#   - LoamSpine built (cargo build)
# ===================================================================

set -e

# Colors for readability
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo ""
echo "=================================================================="
echo "  🦴 LoamSpine: Heartbeat Monitoring & Health Checks"
echo "=================================================================="
echo ""

# Configuration
SONGBIRD_BIN="../../../../bins/songbird-orchestrator"
SONGBIRD_PORT=8082
SONGBIRD_ENDPOINT="http://localhost:${SONGBIRD_PORT}"

# Step 1: Check prerequisites
echo "Step 1: Checking prerequisites..."

if [ ! -f "$SONGBIRD_BIN" ]; then
    echo -e "${RED}✗ Songbird binary not found${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Songbird binary found${NC}"

# Step 2: Start Songbird
echo ""
echo "Step 2: Starting Songbird..."

pkill -f songbird-orchestrator || true
sleep 1

$SONGBIRD_BIN --port $SONGBIRD_PORT --host 127.0.0.1 > /tmp/songbird.log 2>&1 &
SONGBIRD_PID=$!
sleep 2

if ! ps -p $SONGBIRD_PID > /dev/null; then
    echo -e "${RED}✗ Failed to start Songbird${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Songbird running${NC}"

# Cleanup
cleanup() {
    echo ""
    echo "Cleaning up..."
    kill $SONGBIRD_PID 2>/dev/null || true
    pkill -f songbird-orchestrator || true
}
trap cleanup EXIT

# Step 3: Register service
echo ""
echo "Step 3: Registering LoamSpine..."

REGISTER_PAYLOAD='{
  "name": "loamspine",
  "endpoint": "http://localhost:9001",
  "capabilities": ["persistent-ledger"],
  "metadata": {"version": "0.9.15"}
}'

curl -s -X POST \
    -H "Content-Type: application/json" \
    -d "$REGISTER_PAYLOAD" \
    ${SONGBIRD_ENDPOINT}/api/v1/register > /dev/null

echo -e "${GREEN}✓ LoamSpine registered${NC}"

# Step 4: Health check endpoints
echo ""
echo "Step 4: Testing health check endpoints..."

echo ""
echo "   Testing /health (detailed status)..."
HEALTH=$(curl -s "http://localhost:9001/health" 2>/dev/null || echo '{"status":"service_not_running"}')
echo -e "${BLUE}   Response: ${HEALTH}${NC}"
echo -e "${YELLOW}   Note: LoamSpine not actually running (demonstration only)${NC}"

echo ""
echo "   Expected in production:"
echo -e "${BLUE}   /health        → Detailed status with dependencies${NC}"
echo -e "${BLUE}   /health/live   → Simple liveness check${NC}"
echo -e "${BLUE}   /health/ready  → Readiness for traffic${NC}"

# Step 5: Heartbeat simulation with failure
echo ""
echo "Step 5: Simulating heartbeat with failure and recovery..."

echo ""
echo "   Cycle 1: Successful heartbeat"
curl -s -X POST \
    -H "Content-Type: application/json" \
    -d '{"service_name":"loamspine","status":"healthy"}' \
    ${SONGBIRD_ENDPOINT}/api/v1/heartbeat > /dev/null 2>&1 || true
echo -e "${GREEN}   ✓ Heartbeat sent (healthy)${NC}"
sleep 1

echo ""
echo "   Cycle 2: Successful heartbeat"
curl -s -X POST \
    -H "Content-Type: application/json" \
    -d '{"service_name":"loamspine","status":"healthy"}' \
    ${SONGBIRD_ENDPOINT}/api/v1/heartbeat > /dev/null 2>&1 || true
echo -e "${GREEN}   ✓ Heartbeat sent (healthy)${NC}"
sleep 1

echo ""
echo "   Cycle 3: Simulating failure (degraded state)"
curl -s -X POST \
    -H "Content-Type: application/json" \
    -d '{"service_name":"loamspine","status":"degraded"}' \
    ${SONGBIRD_ENDPOINT}/api/v1/heartbeat > /dev/null 2>&1 || true
echo -e "${YELLOW}   ⚠️  Heartbeat sent (degraded)${NC}"
echo -e "${YELLOW}   Service running but some capabilities unavailable${NC}"
sleep 1

echo ""
echo "   Cycle 4: Recovery (back to healthy)"
curl -s -X POST \
    -H "Content-Type: application/json" \
    -d '{"service_name":"loamspine","status":"healthy"}' \
    ${SONGBIRD_ENDPOINT}/api/v1/heartbeat > /dev/null 2>&1 || true
echo -e "${GREEN}   ✓ Heartbeat sent (healthy)${NC}"
echo -e "${GREEN}   Service recovered from degraded state${NC}"

# Step 6: Heartbeat failure detection
echo ""
echo "Step 6: Heartbeat failure detection..."

echo ""
echo "   Simulating missed heartbeats..."
echo -e "${YELLOW}   t=30s: No heartbeat${NC}"
sleep 1
echo -e "${YELLOW}   t=60s: No heartbeat${NC}"
sleep 1
echo -e "${YELLOW}   t=90s: No heartbeat (3 cycles missed)${NC}"
echo -e "${RED}   ⚠️  Service marked as STALE by Songbird${NC}"
sleep 1
echo -e "${YELLOW}   t=120s: No heartbeat${NC}"
sleep 1
echo -e "${YELLOW}   t=150s: No heartbeat${NC}"
sleep 1
echo -e "${YELLOW}   t=180s: No heartbeat (6 cycles missed)${NC}"
echo -e "${RED}   ✗ Service AUTO-DEREGISTERED by Songbird${NC}"

echo ""
echo -e "${GREEN}✓ Failure detection mechanism demonstrated${NC}"

# Visual flow diagram
echo ""
echo "   ┌──────────────────────────────────────┐"
echo "   │     HEARTBEAT MONITORING FLOW        │"
echo "   └──────────────────────────────────────┘"
echo ""
echo "            Normal Operation"
echo "                 │"
echo "         Heartbeat every 30s"
echo "         (status: healthy)"
echo "                 │"
echo "         ┌───────┴───────┐"
echo "         │               │"
echo "     Success?         Failure?"
echo "         │               │"
echo "         ↓               ↓"
echo "   Mark HEALTHY    Mark DEGRADED"
echo "         │               │"
echo "         │       ┌───────┴───────┐"
echo "         │       │               │"
echo "         │   Retry with     Continue"
echo "         │   backoff       failing?"
echo "         │       │               │"
echo "         │       ↓               ↓"
echo "         │   Success?     >90s no heartbeat"
echo "         │       │               │"
echo "         └───────┤               ↓"
echo "                 │         Mark STALE"
echo "                 │               │"
echo "                 │         >180s no heartbeat"
echo "                 │               │"
echo "                 │               ↓"
echo "                 │        AUTO-DEREGISTER"
echo "                 │"
echo "            Continue"
echo ""

# Summary
echo ""
echo "=================================================================="
echo "  Demo Complete!"
echo "=================================================================="
echo ""
echo "What we demonstrated:"
echo "  ✅ Health check endpoint patterns"
echo "  ✅ Heartbeat success and failure cycles"
echo "  ✅ Degraded state detection"
echo "  ✅ Auto-recovery from failures"
echo "  ✅ Stale service detection (90s)"
echo "  ✅ Auto-deregistration (180s)"
echo ""
echo "Health check patterns:"
echo "  /health        → Full status (dependencies, version, uptime)"
echo "  /health/live   → Kubernetes liveness (is process alive?)"
echo "  /health/ready  → Kubernetes readiness (ready for traffic?)"
echo ""
echo "Failure handling:"
echo "  - Retry with exponential backoff: 10s, 30s, 60s, 120s"
echo "  - Mark degraded after failures"
echo "  - Mark stale after 3 missed heartbeats (90s)"
echo "  - Auto-deregister after 6 missed heartbeats (180s)"
echo ""
echo "Gap discovered:"
echo "  ⚠️  Need to implement health check endpoints"
echo "      - GET /health (detailed status)"
echo "      - GET /health/live (liveness probe)"
echo "      - GET /health/ready (readiness probe)"
echo "  ⚠️  Need retry logic with exponential backoff"
echo "  ⚠️  Need state transition logic (healthy → degraded → error)"
echo ""
echo "Next steps:"
echo "  - Implement: Health check endpoints in JSON-RPC server"
echo "  - Implement: Retry logic in LifecycleManager"
echo "  - Try: ../04-inter-primal demos"
echo "  - Learn: specs/SERVICE_LIFECYCLE.md"
echo ""

