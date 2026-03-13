#!/usr/bin/env bash
# Demo: JSON-RPC Basics with Real Service
# Uses actual loamspine binary - NO MOCKS!

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
NC='\033[0m'

# Configuration
LOAMSPINE_BIN="${BINS_DIR}/loamspine"
JSONRPC_PORT="8080"
JSONRPC_URL="http://localhost:${JSONRPC_PORT}"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🦴 LoamSpine JSON-RPC 2.0 Demo${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "🎯 Real Service Integration - NO MOCKS!"
echo ""
echo "This demo uses:"
echo "  • Real loamspine binary"
echo "  • JSON-RPC 2.0 protocol"
echo "  • Language-agnostic API"
echo "  • External client access"
echo ""

# Check if binary exists
if [ ! -f "${LOAMSPINE_BIN}" ]; then
    echo -e "${RED}❌ loamspine binary not found at: ${LOAMSPINE_BIN}${NC}"
    echo "   Please ensure loamspine is built in primalBins/"
    exit 1
fi

echo -e "${YELLOW}✓ loamspine binary found${NC}"
echo ""

# Check if service is already running
if pgrep -f "loamspine" > /dev/null; then
    echo -e "${YELLOW}⚠ LoamSpine service already running, using existing instance${NC}"
else
    echo "🚀 Starting loamspine..."
    "${LOAMSPINE_BIN}" server --jsonrpc-port "${JSONRPC_PORT}" &> /tmp/loamspine-demo.log &
    SERVICE_PID=$!
    echo "${SERVICE_PID}" > /tmp/loamspine-demo.pid
    
    # Wait for service to be ready
    echo "   Waiting for service to start..."
    for i in {1..30}; do
        if curl -s "${JSONRPC_URL}/health" > /dev/null 2>&1; then
            break
        fi
        sleep 0.5
    done
    
    echo -e "   ${GREEN}✅ Service started (PID: ${SERVICE_PID})${NC}"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Test 1: Health Check
echo -e "${CYAN}TEST 1: Health Check${NC}"
echo "   Endpoint: GET ${JSONRPC_URL}/health"
echo ""

HEALTH_RESPONSE=$(curl -s "${JSONRPC_URL}/health")
echo "   Response: ${HEALTH_RESPONSE}"

if echo "${HEALTH_RESPONSE}" | grep -q "ok"; then
    echo -e "   ${GREEN}✅ Service is healthy${NC}"
else
    echo -e "   ${RED}❌ Health check failed${NC}"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Test 2: Create Spine via JSON-RPC
echo -e "${CYAN}TEST 2: Create Spine (JSON-RPC 2.0)${NC}"
echo "   Method: create_spine"
echo "   Protocol: JSON-RPC 2.0"
echo ""

cat > /tmp/create_spine_request.json << 'EOF'
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "create_spine",
  "params": {
    "owner_did": "did:key:z6MkJsonRpcDemo",
    "name": "JSON-RPC Demo Spine"
  }
}
EOF

echo "   Request:"
cat /tmp/create_spine_request.json | jq '.' 2>/dev/null || cat /tmp/create_spine_request.json
echo ""

CREATE_RESPONSE=$(curl -s -X POST "${JSONRPC_URL}" \
    -H "Content-Type: application/json" \
    -d @/tmp/create_spine_request.json)

echo "   Response:"
echo "${CREATE_RESPONSE}" | jq '.' 2>/dev/null || echo "${CREATE_RESPONSE}"
echo ""

SPINE_ID=$(echo "${CREATE_RESPONSE}" | jq -r '.result.spine_id // empty')

if [ -n "${SPINE_ID}" ]; then
    echo -e "   ${GREEN}✅ Spine created: ${SPINE_ID}${NC}"
else
    echo -e "   ${RED}❌ Failed to create spine${NC}"
    echo "   Response: ${CREATE_RESPONSE}"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Test 3: Append Entry
echo -e "${CYAN}TEST 3: Append Entry${NC}"
echo "   Method: append_entry"
echo ""

cat > /tmp/append_entry_request.json << EOF
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "append_entry",
  "params": {
    "spine_id": "${SPINE_ID}",
    "entry_type": {
      "GenericData": {
        "data_type": "demo_entry",
        "content_hash": "demo-hash-001",
        "metadata": "eyJ0ZXN0IjoidmFsdWUifQ=="
      }
    }
  }
}
EOF

echo "   Request:"
cat /tmp/append_entry_request.json | jq '.' 2>/dev/null || cat /tmp/append_entry_request.json
echo ""

APPEND_RESPONSE=$(curl -s -X POST "${JSONRPC_URL}" \
    -H "Content-Type: application/json" \
    -d @/tmp/append_entry_request.json)

echo "   Response:"
echo "${APPEND_RESPONSE}" | jq '.' 2>/dev/null || echo "${APPEND_RESPONSE}"
echo ""

ENTRY_HASH=$(echo "${APPEND_RESPONSE}" | jq -r '.result.entry_hash // empty')

if [ -n "${ENTRY_HASH}" ]; then
    echo -e "   ${GREEN}✅ Entry appended${NC}"
else
    echo -e "   ${RED}❌ Failed to append entry${NC}"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Test 4: Get Spine Info
echo -e "${CYAN}TEST 4: Get Spine Info${NC}"
echo "   Method: get_spine"
echo ""

cat > /tmp/get_spine_request.json << EOF
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "get_spine",
  "params": {
    "spine_id": "${SPINE_ID}"
  }
}
EOF

GET_RESPONSE=$(curl -s -X POST "${JSONRPC_URL}" \
    -H "Content-Type: application/json" \
    -d @/tmp/get_spine_request.json)

echo "   Response:"
echo "${GET_RESPONSE}" | jq '.' 2>/dev/null || echo "${GET_RESPONSE}"
echo ""

HEIGHT=$(echo "${GET_RESPONSE}" | jq -r '.result.height // empty')

if [ -n "${HEIGHT}" ]; then
    echo -e "   ${GREEN}✅ Spine retrieved (height: ${HEIGHT})${NC}"
else
    echo -e "   ${RED}❌ Failed to get spine${NC}"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Test 5: Verify Spine
echo -e "${CYAN}TEST 5: Verify Spine Integrity${NC}"
echo "   Method: verify_spine"
echo ""

cat > /tmp/verify_spine_request.json << EOF
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "verify_spine",
  "params": {
    "spine_id": "${SPINE_ID}"
  }
}
EOF

VERIFY_RESPONSE=$(curl -s -X POST "${JSONRPC_URL}" \
    -H "Content-Type: application/json" \
    -d @/tmp/verify_spine_request.json)

echo "   Response:"
echo "${VERIFY_RESPONSE}" | jq '.' 2>/dev/null || echo "${VERIFY_RESPONSE}"
echo ""

VALID=$(echo "${VERIFY_RESPONSE}" | jq -r '.result.valid // empty')

if [ "${VALID}" = "true" ]; then
    echo -e "   ${GREEN}✅ Spine verification passed${NC}"
else
    echo -e "   ${RED}❌ Spine verification failed${NC}"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${GREEN}✅ All JSON-RPC Tests Passed!${NC}"
echo ""
echo "🎓 What we demonstrated:"
echo "   1. Health check endpoint"
echo "   2. Created spine via JSON-RPC"
echo "   3. Appended entry via JSON-RPC"
echo "   4. Retrieved spine info"
echo "   5. Verified spine integrity"
echo ""
echo "💡 Key Features:"
echo "   • Language-agnostic protocol (JSON-RPC 2.0)"
echo "   • RESTful HTTP endpoints"
echo "   • Same API as tarpc (19 methods)"
echo "   • External client support"
echo "   • Production-ready service"
echo ""
echo "🌐 Use cases:"
echo "   • Python/JavaScript clients"
echo "   • Web frontends"
echo "   • Mobile apps"
echo "   • CLI tools (curl, httpie)"
echo "   • Any language with HTTP support"
echo ""

# Cleanup
rm -f /tmp/create_spine_request.json
rm -f /tmp/append_entry_request.json
rm -f /tmp/get_spine_request.json
rm -f /tmp/verify_spine_request.json

# Stop service if we started it
if [ -f /tmp/loamspine-demo.pid ]; then
    SERVICE_PID=$(cat /tmp/loamspine-demo.pid)
    if kill "${SERVICE_PID}" 2>/dev/null; then
        echo "🛑 Stopped loamspine (PID: ${SERVICE_PID})"
    fi
    rm -f /tmp/loamspine-demo.pid
    rm -f /tmp/loamspine-demo.log
fi

echo ""
echo "🦴 JSON-RPC 2.0 Demo Complete!"
echo ""
