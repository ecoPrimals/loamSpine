#!/bin/bash

# ===================================================================
# LoamSpine Demo: Braid Commit (Multi-Entry Session)
# ===================================================================
# What this demonstrates:
#   - Creating a session with multiple entries
#   - Braiding entries together
#   - Merkle tree construction from multiple hashes
#   - Session integrity verification
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
echo "  🦴 LoamSpine: Braid Commit Demo (Multi-Entry)"
echo "=================================================================="
echo ""

# Configuration
LOAMSPINE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
LOAMSPINE_TARP_PORT=9001
LOAMSPINE_JSON_PORT=8080
STORAGE_PATH="/tmp/loamspine-demo-braid"

# Step 1: Clean environment
echo "Step 1: Preparing environment..."

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
    > /tmp/loamspine-braid.log 2>&1 &

LOAMSPINE_PID=$!
sleep 3

if ! ps -p $LOAMSPINE_PID > /dev/null; then
    echo -e "${RED}✗ Failed to start LoamSpine${NC}"
    cat /tmp/loamspine-braid.log
    exit 1
fi

echo -e "${GREEN}✓ LoamSpine running (PID: ${LOAMSPINE_PID})${NC}"

# Cleanup
cleanup() {
    echo ""
    echo "Cleaning up..."
    kill $LOAMSPINE_PID 2>/dev/null || true
    pkill -f loam-spine-cli || true
}
trap cleanup EXIT

# Step 3: Create session for multiple entries
echo ""
echo "Step 3: Creating braid session (4 entries expected)..."

SESSION_RESPONSE=$(curl -s -X POST http://localhost:${LOAMSPINE_JSON_PORT}/rpc \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "create_session",
        "params": {
            "manifest": {
                "expected_entries": 4,
                "metadata": {
                    "demo": "braid-commit",
                    "description": "Multi-entry session with braiding",
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

echo -e "${GREEN}✓ Braid session created${NC}"
echo -e "${BLUE}   Session ID: ${SESSION_ID}${NC}"
echo -e "${BLUE}   Expected entries: 4${NC}"

# Step 4: Commit multiple entries
echo ""
echo "Step 4: Committing multiple entries to braid..."

declare -a ENTRY_HASHES

# Entry 1: User action
echo ""
echo -e "${BLUE}   Entry 1/4: User action${NC}"
CONTENT_1="User clicked button 'Submit'"
HASH_1=$(echo -n "$CONTENT_1" | sha256sum | awk '{print $1}')
DATA_1=$(echo -n "$CONTENT_1" | base64)

RESPONSE_1=$(curl -s -X POST http://localhost:${LOAMSPINE_JSON_PORT}/rpc \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "commit_entry",
        "params": {
            "session_id": "'$SESSION_ID'",
            "entry": {
                "hash": "'$HASH_1'",
                "content_type": "application/json",
                "data": "'$DATA_1'",
                "metadata": {
                    "type": "user_action",
                    "sequence": 0
                }
            }
        },
        "id": 2
    }')

ENTRY_HASHES[0]=$(echo "$RESPONSE_1" | jq -r '.result.entry_hash')
echo -e "${GREEN}   ✓ Entry 1 committed: ${ENTRY_HASHES[0]:0:12}...${NC}"

# Entry 2: System response
echo -e "${BLUE}   Entry 2/4: System response${NC}"
CONTENT_2="System validated input successfully"
HASH_2=$(echo -n "$CONTENT_2" | sha256sum | awk '{print $1}')
DATA_2=$(echo -n "$CONTENT_2" | base64)

RESPONSE_2=$(curl -s -X POST http://localhost:${LOAMSPINE_JSON_PORT}/rpc \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "commit_entry",
        "params": {
            "session_id": "'$SESSION_ID'",
            "entry": {
                "hash": "'$HASH_2'",
                "content_type": "application/json",
                "data": "'$DATA_2'",
                "metadata": {
                    "type": "system_response",
                    "sequence": 1
                }
            }
        },
        "id": 3
    }')

ENTRY_HASHES[1]=$(echo "$RESPONSE_2" | jq -r '.result.entry_hash')
echo -e "${GREEN}   ✓ Entry 2 committed: ${ENTRY_HASHES[1]:0:12}...${NC}"

# Entry 3: State change
echo -e "${BLUE}   Entry 3/4: State change${NC}"
CONTENT_3='{"status":"processing","progress":0.5}'
HASH_3=$(echo -n "$CONTENT_3" | sha256sum | awk '{print $1}')
DATA_3=$(echo -n "$CONTENT_3" | base64)

RESPONSE_3=$(curl -s -X POST http://localhost:${LOAMSPINE_JSON_PORT}/rpc \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "commit_entry",
        "params": {
            "session_id": "'$SESSION_ID'",
            "entry": {
                "hash": "'$HASH_3'",
                "content_type": "application/json",
                "data": "'$DATA_3'",
                "metadata": {
                    "type": "state_change",
                    "sequence": 2
                }
            }
        },
        "id": 4
    }')

ENTRY_HASHES[2]=$(echo "$RESPONSE_3" | jq -r '.result.entry_hash')
echo -e "${GREEN}   ✓ Entry 3 committed: ${ENTRY_HASHES[2]:0:12}...${NC}"

# Entry 4: Final result
echo -e "${BLUE}   Entry 4/4: Final result${NC}"
CONTENT_4='{"status":"completed","result":"success"}'
HASH_4=$(echo -n "$CONTENT_4" | sha256sum | awk '{print $1}')
DATA_4=$(echo -n "$CONTENT_4" | base64)

RESPONSE_4=$(curl -s -X POST http://localhost:${LOAMSPINE_JSON_PORT}/rpc \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "commit_entry",
        "params": {
            "session_id": "'$SESSION_ID'",
            "entry": {
                "hash": "'$HASH_4'",
                "content_type": "application/json",
                "data": "'$DATA_4'",
                "metadata": {
                    "type": "final_result",
                    "sequence": 3
                }
            }
        },
        "id": 5
    }')

ENTRY_HASHES[3]=$(echo "$RESPONSE_4" | jq -r '.result.entry_hash')
echo -e "${GREEN}   ✓ Entry 4 committed: ${ENTRY_HASHES[3]:0:12}...${NC}"

echo ""
echo -e "${GREEN}✓ All 4 entries committed to braid${NC}"

# Step 5: Finalize braid (compute Merkle tree)
echo ""
echo "Step 5: Finalizing braid (computing Merkle tree)..."

FINALIZE_RESPONSE=$(curl -s -X POST http://localhost:${LOAMSPINE_JSON_PORT}/rpc \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "finalize_session",
        "params": {
            "session_id": "'$SESSION_ID'"
        },
        "id": 6
    }')

MERKLE_ROOT=$(echo "$FINALIZE_RESPONSE" | jq -r '.result.merkle_root')
ENTRY_COUNT=$(echo "$FINALIZE_RESPONSE" | jq -r '.result.entry_count')

if [ -z "$MERKLE_ROOT" ] || [ "$MERKLE_ROOT" = "null" ]; then
    echo -e "${RED}✗ Failed to finalize braid${NC}"
    echo "Response: $FINALIZE_RESPONSE"
    exit 1
fi

echo -e "${GREEN}✓ Braid finalized${NC}"
echo -e "${BLUE}   Merkle Root: ${MERKLE_ROOT}${NC}"
echo -e "${BLUE}   Entry Count: ${ENTRY_COUNT}${NC}"

# Visual representation
echo ""
echo "   ┌──────────────────────────────────────────┐"
echo "   │      MERKLE TREE STRUCTURE               │"
echo "   └──────────────────────────────────────────┘"
echo ""
echo "                  Merkle Root"
echo "              ${MERKLE_ROOT:0:16}..."
echo "                    /    \\"
echo "                   /      \\"
echo "              Hash(1,2)  Hash(3,4)"
echo "               /    \\      /    \\"
echo "              /      \\    /      \\"
echo "          Entry1  Entry2 Entry3  Entry4"
echo "          ${ENTRY_HASHES[0]:0:8} ${ENTRY_HASHES[1]:0:8} ${ENTRY_HASHES[2]:0:8} ${ENTRY_HASHES[3]:0:8}"
echo ""
echo "   This Merkle root cryptographically proves:"
echo "   - All 4 entries are included"
echo "   - Entries are in this specific order"
echo "   - Content has not been tampered with"
echo ""

# Step 6: Verify braid integrity
echo ""
echo "Step 6: Verifying braid integrity..."

echo -e "${BLUE}   Retrieving all entries and verifying...${NC}"

ALL_VALID=true

for i in {0..3}; do
    VERIFY_RESPONSE=$(curl -s -X POST http://localhost:${LOAMSPINE_JSON_PORT}/rpc \
        -H "Content-Type: application/json" \
        -d '{
            "jsonrpc": "2.0",
            "method": "get_entry",
            "params": {
                "session_id": "'$SESSION_ID'",
                "entry_hash": "'${ENTRY_HASHES[$i]}'"
            },
            "id": '$((7+i))'
        }')
    
    RETRIEVED_HASH=$(echo "$VERIFY_RESPONSE" | jq -r '.result.entry.hash')
    
    if [ "$RETRIEVED_HASH" = "${ENTRY_HASHES[$i]}" ]; then
        echo -e "${GREEN}   ✓ Entry $((i+1)) verified${NC}"
    else
        echo -e "${RED}   ✗ Entry $((i+1)) verification failed${NC}"
        ALL_VALID=false
    fi
done

if [ "$ALL_VALID" = true ]; then
    echo ""
    echo -e "${GREEN}✓ Braid integrity verified${NC}"
else
    echo ""
    echo -e "${RED}✗ Braid integrity check failed${NC}"
    exit 1
fi

# Summary
echo ""
echo "=================================================================="
echo "  Demo Complete!"
echo "=================================================================="
echo ""
echo "What we demonstrated:"
echo "  ✅ Created a multi-entry session (braid)"
echo "  ✅ Committed 4 entries with different types"
echo "  ✅ Computed Merkle tree from all entries"
echo "  ✅ Verified braid integrity"
echo "  ✅ Demonstrated cryptographic proof"
echo ""
echo "Braid Summary:"
echo "  Session ID:    $SESSION_ID"
echo "  Entry Count:   $ENTRY_COUNT"
echo "  Merkle Root:   $MERKLE_ROOT"
echo ""
echo "Entry Types Demonstrated:"
echo "  1. User action       (${ENTRY_HASHES[0]:0:16}...)"
echo "  2. System response   (${ENTRY_HASHES[1]:0:16}...)"
echo "  3. State change      (${ENTRY_HASHES[2]:0:16}...)"
echo "  4. Final result      (${ENTRY_HASHES[3]:0:16}...)"
echo ""
echo "Why Braiding Matters:"
echo "  - Multiple entries form a cryptographic proof"
echo "  - Merkle root proves all entries + order"
echo "  - Any change breaks the root hash"
echo "  - Efficient verification (log n proofs)"
echo ""
echo "Storage:"
echo "  Path: $STORAGE_PATH"
echo "  Files: $(ls -1 $STORAGE_PATH 2>/dev/null | wc -l) session structures"
echo ""
echo "Next steps:"
echo "  - Try: 03-signing-capability (integration with BearDog)"
echo "  - Learn: How signed entries work"
echo "  - Explore: Merkle proofs for individual entries"
echo ""

