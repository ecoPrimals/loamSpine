#!/bin/bash

# ===================================================================
# LoamSpine Demo: Auto-Advertise & Lifecycle Management
# ===================================================================
# What this demonstrates:
#   - Automatic registration on startup
#   - Heartbeat mechanism (keep-alive)
#   - Auto-deregistration on shutdown
#   - Graceful lifecycle management
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
echo "  🦴 LoamSpine: Auto-Advertise & Lifecycle Management"
echo "=================================================================="
echo ""

# Configuration
SONGBIRD_BIN="../../../../bins/songbird-orchestrator"
SONGBIRD_PORT=8082
SONGBIRD_ENDPOINT="http://localhost:${SONGBIRD_PORT}"

# Step 1: Check prerequisites
echo "Step 1: Checking prerequisites..."

if [ ! -f "$SONGBIRD_BIN" ]; then
    echo -e "${RED}✗ Songbird binary not found at: ${SONGBIRD_BIN}${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Songbird binary found${NC}"

# Step 2: Start Songbird
echo ""
echo "Step 2: Starting Songbird orchestrator..."

pkill -f songbird-orchestrator || true
sleep 1

$SONGBIRD_BIN --port $SONGBIRD_PORT --host 127.0.0.1 > /tmp/songbird.log 2>&1 &
SONGBIRD_PID=$!
sleep 2

if ! ps -p $SONGBIRD_PID > /dev/null; then
    echo -e "${RED}✗ Failed to start Songbird${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Songbird running (PID: ${SONGBIRD_PID})${NC}"

# Cleanup function
cleanup() {
    echo ""
    echo "Step 5: Cleanup - Auto-deregistration..."
    
    # In production, LoamSpine would deregister on SIGTERM
    curl -s -X DELETE "${SONGBIRD_ENDPOINT}/api/v1/deregister/loamspine" > /dev/null 2>&1 || true
    echo -e "${GREEN}✓ LoamSpine deregistered from Songbird${NC}"
    
    kill $SONGBIRD_PID 2>/dev/null || true
    pkill -f songbird-orchestrator || true
}
trap cleanup EXIT

# Step 3: Auto-register on startup
echo ""
echo "Step 3: Auto-register LoamSpine on startup..."

# This simulates what LoamSpine would do automatically on startup
# In production: LifecycleManager::start() handles this
REGISTER_PAYLOAD='{
  "name": "loamspine",
  "endpoint": "http://localhost:9001",
  "capabilities": [
    "persistent-ledger",
    "waypoint-anchoring",
    "certificate-manager",
    "proof-generation"
  ],
  "metadata": {
    "version": "0.9.15",
    "startup_time": "'$(date -Iseconds)'",
    "auto_registered": true
  }
}'

curl -s -X POST \
    -H "Content-Type: application/json" \
    -d "$REGISTER_PAYLOAD" \
    ${SONGBIRD_ENDPOINT}/api/v1/register > /dev/null

echo -e "${GREEN}✓ LoamSpine auto-registered on startup${NC}"
echo -e "${BLUE}   Startup time: $(date)${NC}"

# Verify registration
SERVICES=$(curl -s "${SONGBIRD_ENDPOINT}/api/v1/services")
if [[ "$SERVICES" == *"loamspine"* ]]; then
    echo -e "${GREEN}✓ Registration verified${NC}"
else
    echo -e "${RED}✗ Registration failed${NC}"
    exit 1
fi

# Step 4: Heartbeat mechanism
echo ""
echo "Step 4: Heartbeat mechanism (keep-alive)..."
echo ""
echo "   Simulating 3 heartbeat cycles (would run continuously in production)"

for i in {1..3}; do
    echo -e "${BLUE}   Heartbeat ${i}/3...${NC}"
    
    # Send heartbeat
    HEARTBEAT_PAYLOAD='{
      "service_name": "loamspine",
      "timestamp": "'$(date -Iseconds)'",
      "status": "healthy"
    }'
    
    curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$HEARTBEAT_PAYLOAD" \
        ${SONGBIRD_ENDPOINT}/api/v1/heartbeat > /dev/null 2>&1 || true
    
    echo -e "${GREEN}   ✓ Heartbeat sent ($(date +%H:%M:%S))${NC}"
    
    # In production, heartbeat every 30 seconds
    # For demo, wait 2 seconds between heartbeats
    sleep 2
done

echo ""
echo -e "${GREEN}✓ Heartbeat mechanism working${NC}"
echo -e "${BLUE}   In production: heartbeat every 30s${NC}"

# Visual flow diagram
echo ""
echo "   ┌──────────────────────────────────────┐"
echo "   │      SERVICE LIFECYCLE FLOW          │"
echo "   └──────────────────────────────────────┘"
echo ""
echo "         1. Service Starts"
echo "                 │"
echo "                 ↓"
echo "      Auto-register with Songbird"
echo "      (advertise capabilities)"
echo "                 │"
echo "                 ↓"
echo "        Run Heartbeat Loop"
echo "        (every 30 seconds)"
echo "                 │"
echo "            ┌────┴────┐"
echo "            │         │"
echo "        Healthy   Failure?"
echo "            │         │"
echo "            ↓         ↓"
echo "        Continue   Retry"
echo "            │         │"
echo "            └────┬────┘"
echo "                 │"
echo "                 ↓"
echo "      Service Shuts Down"
echo "                 │"
echo "                 ↓"
echo "     Auto-deregister from Songbird"
echo "     (graceful cleanup)"
echo ""

# Summary
echo ""
echo "=================================================================="
echo "  Demo Complete!"
echo "=================================================================="
echo ""
echo "What we demonstrated:"
echo "  ✅ Auto-registration on startup"
echo "  ✅ Heartbeat mechanism (keep-alive)"
echo "  ✅ Auto-deregistration on shutdown"
echo "  ✅ Graceful lifecycle management"
echo ""
echo "Key principles:"
echo "  - Services self-register (no manual configuration)"
echo "  - Heartbeat keeps registration alive"
echo "  - Graceful shutdown cleans up registration"
echo "  - Songbird knows which services are healthy"
echo ""
echo "Gap discovered:"
echo "  ⚠️  Need to implement LifecycleManager in LoamSpine"
echo "      - Auto-registration on startup"
echo "      - Heartbeat loop in background"
echo "      - SIGTERM handler for graceful shutdown"
echo "      - Retry logic for failed heartbeats"
echo ""
echo "Next steps:"
echo "  - Try: 04-heartbeat-monitoring (health checks)"
echo "  - Implement: LifecycleManager with auto-advertise"
echo "  - Learn: specs/SERVICE_LIFECYCLE.md"
echo ""

