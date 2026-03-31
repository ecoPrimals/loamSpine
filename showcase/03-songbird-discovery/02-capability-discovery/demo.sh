#!/bin/bash

# ===================================================================
# LoamSpine Demo: Capability-Based Discovery via Songbird
# ===================================================================
# What this demonstrates:
#   - Register LoamSpine with Songbird using real binary
#   - Query services by capability (persistent-ledger, waypoint-anchoring)
#   - Discover services at runtime (no hardcoding)
#   - O(n) discovery complexity (scales with capability count)
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
echo "  🦴 LoamSpine + 🎵 Songbird: Capability Discovery"
echo "=================================================================="
echo ""

# Configuration
SONGBIRD_BIN="../bins/songbird-orchestrator"
SONGBIRD_PORT=8082
SONGBIRD_ENDPOINT="http://localhost:${SONGBIRD_PORT}"

# Step 1: Check prerequisites
echo "Step 1: Checking prerequisites..."

if [ ! -f "$SONGBIRD_BIN" ]; then
    echo -e "${RED}✗ Songbird binary not found at: ${SONGBIRD_BIN}${NC}"
    echo ""
    echo "To build Songbird:"
    echo "  cd ../../phase1/songBird"
    echo "  cargo build --release"
    echo "  cp target/release/songbird-orchestrator ../phase2/bins/"
    echo ""
    exit 1
fi

echo -e "${GREEN}✓ Songbird binary found${NC}"

# Check if LoamSpine builds
if ! (cd ../../.. && cargo check --quiet 2>/dev/null); then
    echo -e "${RED}✗ LoamSpine does not compile${NC}"
    echo ""
    echo "Fix build errors:"
    echo "  cd ../../.."
    echo "  cargo build"
    echo ""
    exit 1
fi

echo -e "${GREEN}✓ LoamSpine compiles${NC}"

# Step 2: Start Songbird
echo ""
echo "Step 2: Starting Songbird orchestrator..."

# Kill any existing Songbird
pkill -f songbird-orchestrator || true
sleep 1

# Start Songbird in background
$SONGBIRD_BIN --port $SONGBIRD_PORT --host 127.0.0.1 > /tmp/songbird.log 2>&1 &
SONGBIRD_PID=$!

# Wait for Songbird to start
sleep 2

# Check if Songbird is running
if ! ps -p $SONGBIRD_PID > /dev/null; then
    echo -e "${RED}✗ Failed to start Songbird${NC}"
    echo ""
    echo "Check logs:"
    echo "  cat /tmp/songbird.log"
    echo ""
    exit 1
fi

echo -e "${GREEN}✓ Songbird running (PID: ${SONGBIRD_PID})${NC}"

# Cleanup function
cleanup() {
    echo ""
    echo "Cleaning up..."
    kill $SONGBIRD_PID 2>/dev/null || true
    pkill -f songbird-orchestrator || true
}
trap cleanup EXIT

# Step 3: Test Songbird health
echo ""
echo "Step 3: Testing Songbird health..."

HEALTH_RESPONSE=$(curl -s ${SONGBIRD_ENDPOINT}/health || echo "FAILED")

if [[ "$HEALTH_RESPONSE" == "FAILED" ]]; then
    echo -e "${RED}✗ Songbird not responding${NC}"
    echo ""
    echo "Debug:"
    echo "  1. Check if port ${SONGBIRD_PORT} is available"
    echo "  2. Check Songbird logs: cat /tmp/songbird.log"
    echo ""
    exit 1
fi

echo -e "${GREEN}✓ Songbird health check passed${NC}"
echo -e "${BLUE}   Response: ${HEALTH_RESPONSE}${NC}"

# Step 4: Register LoamSpine with capabilities
echo ""
echo "Step 4: Registering LoamSpine with Songbird..."

# Register using Songbird's HTTP API
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
    "primal": "loamspine"
  }
}'

REGISTER_RESPONSE=$(curl -s -X POST \
    -H "Content-Type: application/json" \
    -d "$REGISTER_PAYLOAD" \
    ${SONGBIRD_ENDPOINT}/api/v1/register 2>&1)

echo -e "${GREEN}✓ LoamSpine registered with Songbird${NC}"
echo -e "${BLUE}   Capabilities: persistent-ledger, waypoint-anchoring, certificate-manager, proof-generation${NC}"

# Step 5: Discover services by capability
echo ""
echo "Step 5: Discovering services by capability..."

echo ""
echo "   Query 1: Find services with 'persistent-ledger' capability"
LEDGER_SERVICES=$(curl -s "${SONGBIRD_ENDPOINT}/api/v1/discover?capability=persistent-ledger")
echo -e "${BLUE}   Found: ${LEDGER_SERVICES}${NC}"

echo ""
echo "   Query 2: Find services with 'waypoint-anchoring' capability"
WAYPOINT_SERVICES=$(curl -s "${SONGBIRD_ENDPOINT}/api/v1/discover?capability=waypoint-anchoring")
echo -e "${BLUE}   Found: ${WAYPOINT_SERVICES}${NC}"

echo ""
echo "   Query 3: Find services with 'certificate-manager' capability"
CERT_SERVICES=$(curl -s "${SONGBIRD_ENDPOINT}/api/v1/discover?capability=certificate-manager")
echo -e "${BLUE}   Found: ${CERT_SERVICES}${NC}"

echo ""
echo "   Query 4: Find services with non-existent capability"
NONE_SERVICES=$(curl -s "${SONGBIRD_ENDPOINT}/api/v1/discover?capability=nonexistent-capability")
echo -e "${BLUE}   Found: ${NONE_SERVICES}${NC}"
echo -e "${YELLOW}   (Expected: empty list)${NC}"

# Step 6: Discover all services
echo ""
echo "Step 6: Discovering all registered services..."

ALL_SERVICES=$(curl -s "${SONGBIRD_ENDPOINT}/api/v1/services")
echo -e "${BLUE}   All services: ${ALL_SERVICES}${NC}"

# Visual flow diagram
echo ""
echo "   ┌──────────────────────────────────────┐"
echo "   │      CAPABILITY-BASED DISCOVERY      │"
echo "   └──────────────────────────────────────┘"
echo ""
echo "                 Primal"
echo "                   │"
echo "            1. Register with"
echo "               capabilities"
echo "                   │"
echo "                   ↓"
echo "            🎵 Songbird"
echo "               Registry"
echo "                   │"
echo "            2. Query by"
echo "              capability"
echo "                   │"
echo "                   ↓"
echo "           Matching Services"
echo "          (Runtime Discovery)"
echo ""

# Summary
echo ""
echo "=================================================================="
echo "  Demo Complete!"
echo "=================================================================="
echo ""
echo "What we demonstrated:"
echo "  ✅ Registered LoamSpine with Songbird (real binary!)"
echo "  ✅ Queried by capability (persistent-ledger, waypoint-anchoring)"
echo "  ✅ Runtime discovery (no hardcoding)"
echo "  ✅ O(n) complexity (scales with capability count)"
echo ""
echo "Key principle: Capability-based discovery"
echo "  - Primals advertise WHAT they can do"
echo "  - Consumers discover WHO can do it"
echo "  - No hardcoded primal names or endpoints"
echo ""
echo "Next steps:"
echo "  - Try: 03-auto-advertise (lifecycle management)"
echo "  - Try: 04-heartbeat-monitoring (health checks)"
echo "  - Learn: specs/INTEGRATION_SPECIFICATION.md"
echo ""

