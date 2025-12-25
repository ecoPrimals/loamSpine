#!/bin/bash

# ===================================================================
# LoamSpine Demo: Session Commit (Local Single-Entry)
# ===================================================================
# What this demonstrates:
#   - Creating a session with LoamSpine
#   - Committing a single entry
#   - Verifying the commit through hash
#   - Real JSON-RPC interaction (no mocks)
# Prerequisites:
#   - LoamSpine built (cargo build)
# ===================================================================

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo ""
echo "=================================================================="
echo "  🦴 LoamSpine: Session Commit Demo"
echo "=================================================================="
echo ""

# Configuration
LOAMSPINE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
LOAMSPINE_TARP_PORT=9001
LOAMSPINE_JSON_PORT=8080
STORAGE_PATH="/tmp/loamspine-demo-session"

# Step 1: Clean environment
echo "Step 1: Preparing environment..."

# Kill any existing LoamSpine instance
pkill -f loam-spine-cli || true
rm -rf "$STORAGE_PATH"
mkdir -p "$STORAGE_PATH"

echo -e "${GREEN}✓ Environment ready${NC}"

# Step 2: Start LoamSpine
echo ""
echo "Step 2: Starting LoamSpine server..."

cd "$LOAMSPINE_ROOT"
cargo run --bin loam-spine-cli -- \
    --storage "$STORAGE_PATH" \
    --json-rpc-port $LOAMSPINE_JSON_PORT \
    --tarp-port $LOAMSPINE_TARP_PORT \
    > /tmp/loamspine-demo.log 2>&1 &

LOAMSPINE_PID=$!
sleep 3

if ! ps -p $LOAMSPINE_PID > /dev/null; then
    echo -e "${RED}✗ Failed to start LoamSpine${NC}"
    cat /tmp/loamspine-demo.log
    exit 1
fi

echo -e "${GREEN}✓ LoamSpine running (PID: ${LOAMSPINE_PID})${NC}"
echo -e "${BLUE}   TARP endpoint: http://localhost:${LOAMSPINE_TARP_PORT}${NC}"
echo -e "${BLUE}   JSON-RPC endpoint: http://localhost:${LOAMSPINE_JSON_PORT}${NC}"

# Cleanup function
cleanup() {
    echo ""
    echo "Cleaning up..."
    kill $LOAMSPINE_PID 2>/dev/null || true
    pkill -f loam-spine-cli || true
}
trap cleanup EXIT

# Step 3: Create session
echo ""
echo "Step 3: Creating session..."

SESSION_RESPONSE=$(curl -s -X POST http://localhost:${LOAMSPINE_JSON_PORT}/rpc \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "create_session",
        "params": {
            "manifest": {
                "expected_entries": 1,
                "metadata": {
                    "demo": "session-commit",
                    "timestamp": "'$(date -Iseconds)'"
                }
            }
        },
        "id": 1
    }')

SESSION_ID=$(echo "$SESSION_RESPONSE" | jq -r '.result.session_id')

if [ -z "$SESSION_ID" ] || [ "$SESSION_ID" = "null" ]; then
    echo -e "${RED}✗ Failed to create session${NC}"
    echo "Response: $SESSION_RESPONSE"
    exit 1
fi

echo -e "${GREEN}✓ Session created${NC}"
echo -e "${BLUE}   Session ID: ${SESSION_ID}${NC}"

# Step 4: Commit entry
echo ""
echo "Step 4: Committing entry to session..."

# Create entry content
ENTRY_CONTENT="Hello from LoamSpine Session Demo!"
ENTRY_HASH=$(echo -n "$ENTRY_CONTENT" | sha256sum | awk '{print $1}')

COMMIT_RESPONSE=$(curl -s -X POST http://localhost:${LOAMSPINE_JSON_PORT}/rpc \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "commit_entry",
        "params": {
            "session_id": "'$SESSION_ID'",
            "entry": {
                "hash": "'$ENTRY_HASH'",
                "content_type": "text/plain",
                "data": "'$(echo -n "$ENTRY_CONTENT" | base64)'",
                "metadata": {
                    "author": "demo-user",
                    "description": "Single session entry"
                }
            }
        },
        "id": 2
    }')

COMMIT_HASH=$(echo "$COMMIT_RESPONSE" | jq -r '.result.entry_hash')

if [ -z "$COMMIT_HASH" ] || [ "$COMMIT_HASH" = "null" ]; then
    echo -e "${RED}✗ Failed to commit entry${NC}"
    echo "Response: $COMMIT_RESPONSE"
    exit 1
fi

echo -e "${GREEN}✓ Entry committed${NC}"
echo -e "${BLUE}   Entry Hash: ${COMMIT_HASH}${NC}"
echo -e "${BLUE}   Content: ${ENTRY_CONTENT}${NC}"

# Step 5: Verify entry
echo ""
echo "Step 5: Verifying entry..."

VERIFY_RESPONSE=$(curl -s -X POST http://localhost:${LOAMSPINE_JSON_PORT}/rpc \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "get_entry",
        "params": {
            "session_id": "'$SESSION_ID'",
            "entry_hash": "'$COMMIT_HASH'"
        },
        "id": 3
    }')

RETRIEVED_CONTENT=$(echo "$VERIFY_RESPONSE" | jq -r '.result.entry.data' | base64 -d)

if [ "$RETRIEVED_CONTENT" = "$ENTRY_CONTENT" ]; then
    echo -e "${GREEN}✓ Entry verified successfully${NC}"
    echo -e "${BLUE}   Retrieved: ${RETRIEVED_CONTENT}${NC}"
else
    echo -e "${RED}✗ Entry verification failed${NC}"
    echo "Expected: $ENTRY_CONTENT"
    echo "Got: $RETRIEVED_CONTENT"
    exit 1
fi

# Step 6: Finalize session
echo ""
echo "Step 6: Finalizing session..."

FINALIZE_RESPONSE=$(curl -s -X POST http://localhost:${LOAMSPINE_JSON_PORT}/rpc \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "finalize_session",
        "params": {
            "session_id": "'$SESSION_ID'"
        },
        "id": 4
    }')

MERKLE_ROOT=$(echo "$FINALIZE_RESPONSE" | jq -r '.result.merkle_root')

if [ -z "$MERKLE_ROOT" ] || [ "$MERKLE_ROOT" = "null" ]; then
    echo -e "${RED}✗ Failed to finalize session${NC}"
    echo "Response: $FINALIZE_RESPONSE"
    exit 1
fi

echo -e "${GREEN}✓ Session finalized${NC}"
echo -e "${BLUE}   Merkle Root: ${MERKLE_ROOT}${NC}"

# Visual flow
echo ""
echo "   ┌──────────────────────────────────────┐"
echo "   │        SESSION COMMIT FLOW           │"
echo "   └──────────────────────────────────────┘"
echo ""
echo "         1. Create Session"
echo "                 │"
echo "                 ↓"
echo "            Session ID"
echo "           ($SESSION_ID)"
echo "                 │"
echo "                 ↓"
echo "         2. Commit Entry"
echo "      (content + metadata)"
echo "                 │"
echo "                 ↓"
echo "            Entry Hash"
echo "           (${COMMIT_HASH:0:12}...)"
echo "                 │"
echo "                 ↓"
echo "         3. Verify Entry"
echo "       (retrieve by hash)"
echo "                 │"
echo "                 ↓"
echo "      4. Finalize Session"
echo "        (compute Merkle)"
echo "                 │"
echo "                 ↓"
echo "           Merkle Root"
echo "           (${MERKLE_ROOT:0:12}...)"
echo ""

# Summary
echo ""
echo "=================================================================="
echo "  Demo Complete!"
echo "=================================================================="
echo ""
echo "What we demonstrated:"
echo "  ✅ Created a session with LoamSpine"
echo "  ✅ Committed a single entry with metadata"
echo "  ✅ Verified the entry through its hash"
echo "  ✅ Finalized the session with Merkle root"
echo "  ✅ Used real JSON-RPC calls (no mocks)"
echo ""
echo "Session Summary:"
echo "  Session ID:   $SESSION_ID"
echo "  Entry Hash:   $COMMIT_HASH"
echo "  Merkle Root:  $MERKLE_ROOT"
echo "  Content:      \"$ENTRY_CONTENT\""
echo ""
echo "Storage Location:"
echo "  Path: $STORAGE_PATH"
echo "  Files: $(ls -1 $STORAGE_PATH | wc -l) files created"
echo ""
echo "Next steps:"
echo "  - Try: 02-braid-commit (multi-entry session)"
echo "  - Learn: How braiding works with multiple entries"
echo "  - Explore: Storage internals at $STORAGE_PATH"
echo ""

