#!/bin/bash

# ===================================================================
# LoamSpine Demo: Storage Capability (Conceptual NestGate Integration)
# ===================================================================
# What this demonstrates:
#   - Conceptual integration with NestGate for persistent storage
#   - LoamSpine discovers storage capability
#   - Backing up sessions to external storage
#   - Restoring sessions from external storage
#   - Gap: NestGate binary not yet available
# Prerequisites:
#   - LoamSpine built (cargo build)
# Note: This demo is conceptual, showing the pattern for future integration
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
echo "  🦴 LoamSpine + 🏠 NestGate: Storage Capability Demo"
echo "  (Conceptual — NestGate binary not yet available)"
echo "=================================================================="
echo ""

# Configuration
LOAMSPINE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
LOAMSPINE_TARP_PORT=9001
LOAMSPINE_JSON_PORT=8080
STORAGE_PATH="/tmp/loamspine-demo-storage"
NESTGATE_BACKUP_PATH="/tmp/nestgate-backups"

# Step 1: Explain the concept
echo "Step 1: Understanding Storage Capability Integration..."
echo ""
echo -e "${BLUE}What is NestGate?${NC}"
echo "  - Distributed storage primal"
echo "  - Provides persistent, redundant storage"
echo "  - Capability: 'persistent-storage'"
echo ""
echo -e "${BLUE}Why integrate LoamSpine + NestGate?${NC}"
echo "  - LoamSpine: Fast local ledger"
echo "  - NestGate: Durable remote backup"
echo "  - Together: Best of both worlds"
echo ""
echo -e "${YELLOW}Note: NestGate binary not yet in ../../bins/${NC}"
echo -e "${YELLOW}This demo shows the pattern using local filesystem${NC}"
echo ""

# Step 2: Prepare environment
echo "Step 2: Preparing environment..."

pkill -f loam-spine-cli || true
rm -rf "$STORAGE_PATH" "$NESTGATE_BACKUP_PATH"
mkdir -p "$STORAGE_PATH" "$NESTGATE_BACKUP_PATH"

echo -e "${GREEN}✓ Environment ready${NC}"

# Step 3: Start LoamSpine
echo ""
echo "Step 3: Starting LoamSpine server..."

cd "$LOAMSPINE_ROOT"
cargo run --bin loam-spine-cli -- \
    --storage "$STORAGE_PATH" \
    --json-rpc-port $LOAMSPINE_JSON_PORT \
    --tarp-port $LOAMSPINE_TARP_PORT \
    > /tmp/loamspine-storage.log 2>&1 &

LOAMSPINE_PID=$!
sleep 3

if ! ps -p $LOAMSPINE_PID > /dev/null; then
    echo -e "${RED}✗ Failed to start LoamSpine${NC}"
    cat /tmp/loamspine-storage.log
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

# Step 4: Create and populate session
echo ""
echo "Step 4: Creating session with multiple entries..."

SESSION_RESPONSE=$(curl -s -X POST http://localhost:${LOAMSPINE_JSON_PORT}/rpc \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "create_session",
        "params": {
            "manifest": {
                "expected_entries": 3,
                "metadata": {
                    "demo": "storage-capability",
                    "backup_policy": "immediate",
                    "timestamp": "'$(date -Iseconds)'"
                }
            }
        },
        "id": 1
    }')

SESSION_ID=$(echo "$SESSION_RESPONSE" | jq -r '.result.session_id')

echo -e "${GREEN}✓ Session created${NC}"
echo -e "${BLUE}   Session ID: ${SESSION_ID}${NC}"

# Commit 3 entries
for i in {1..3}; do
    CONTENT="Entry $i: Data for backup demonstration"
    HASH=$(echo -n "$CONTENT" | sha256sum | awk '{print $1}')
    
    curl -s -X POST http://localhost:${LOAMSPINE_JSON_PORT}/rpc \
        -H "Content-Type: application/json" \
        -d '{
            "jsonrpc": "2.0",
            "method": "commit_entry",
            "params": {
                "session_id": "'$SESSION_ID'",
                "entry": {
                    "hash": "'$HASH'",
                    "content_type": "text/plain",
                    "data": "'$(echo -n "$CONTENT" | base64)'",
                    "metadata": {"sequence": '$i'}
                }
            },
            "id": '$((i+1))'
        }' > /dev/null
    
    echo -e "${GREEN}   ✓ Entry $i committed${NC}"
done

# Finalize session
curl -s -X POST http://localhost:${LOAMSPINE_JSON_PORT}/rpc \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "finalize_session",
        "params": {"session_id": "'$SESSION_ID'"},
        "id": 5
    }' > /dev/null

echo -e "${GREEN}✓ Session finalized${NC}"

# Step 5: Backup session (simulating NestGate)
echo ""
echo "Step 5: Backing up session to external storage..."
echo -e "${BLUE}   (Simulating NestGate with local filesystem)${NC}"

# In production, this would be:
# curl -X POST http://nestgate:9000/api/backup \
#   -d '{"session_id": "'$SESSION_ID'", "data": "..."}'

# Simulate by copying session files
if [ -d "$STORAGE_PATH" ]; then
    cp -r "$STORAGE_PATH" "$NESTGATE_BACKUP_PATH/session_$SESSION_ID"
    echo -e "${GREEN}✓ Session backed up${NC}"
    echo -e "${BLUE}   Backup location: $NESTGATE_BACKUP_PATH/session_$SESSION_ID${NC}"
else
    echo -e "${RED}✗ Storage path not found${NC}"
    exit 1
fi

# Step 6: Simulate data loss
echo ""
echo "Step 6: Simulating local data loss..."

rm -rf "$STORAGE_PATH"
echo -e "${YELLOW}⚠️  Local storage deleted${NC}"

# Step 7: Restore from backup
echo ""
echo "Step 7: Restoring session from external storage..."
echo -e "${BLUE}   (Simulating restoration from NestGate)${NC}"

# In production, this would be:
# curl -X POST http://nestgate:9000/api/restore \
#   -d '{"session_id": "'$SESSION_ID'"}'

# Simulate by copying back
cp -r "$NESTGATE_BACKUP_PATH/session_$SESSION_ID" "$STORAGE_PATH"
echo -e "${GREEN}✓ Session restored from backup${NC}"

# Step 8: Verify restored data
echo ""
echo "Step 8: Verifying restored session..."

# Restart LoamSpine to load restored data
kill $LOAMSPINE_PID 2>/dev/null || true
sleep 1

cargo run --bin loam-spine-cli -- \
    --storage "$STORAGE_PATH" \
    --json-rpc-port $LOAMSPINE_JSON_PORT \
    --tarp-port $LOAMSPINE_TARP_PORT \
    > /tmp/loamspine-storage.log 2>&1 &

LOAMSPINE_PID=$!
sleep 3

# Try to retrieve session (this would work if session loading is implemented)
SESSION_CHECK=$(curl -s -X POST http://localhost:${LOAMSPINE_JSON_PORT}/rpc \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "get_session",
        "params": {"session_id": "'$SESSION_ID'"},
        "id": 6
    }' || echo '{"error":"not_implemented"}')

if echo "$SESSION_CHECK" | grep -q "error"; then
    echo -e "${YELLOW}⚠️  Session retrieval not yet implemented${NC}"
    echo -e "${YELLOW}   Gap: Need session loading on startup${NC}"
else
    echo -e "${GREEN}✓ Session verified after restoration${NC}"
fi

# Visual flow
echo ""
echo "   ┌──────────────────────────────────────────────┐"
echo "   │   LOAMSPINE + NESTGATE INTEGRATION           │"
echo "   └──────────────────────────────────────────────┘"
echo ""
echo "         LoamSpine Creates Session"
echo "         (local fast storage)"
echo "                  │"
echo "                  ↓"
echo "         Commit Multiple Entries"
echo "         (entries 1, 2, 3)"
echo "                  │"
echo "                  ↓"
echo "          Finalize Session"
echo "         (compute Merkle root)"
echo "                  │"
echo "                  ↓"
echo "      Backup to NestGate (async)"
echo "       (distributed storage)"
echo "                  │"
echo "         ┌────────┴────────┐"
echo "         │                 │"
echo "    Local Storage    NestGate Backup"
echo "    (fast access)    (durability)"
echo "         │                 │"
echo "         ↓                 │"
echo "    Local Failure          │"
echo "    (data loss)            │"
echo "                           ↓"
echo "              Restore from NestGate"
echo "              (disaster recovery)"
echo ""

# Summary
echo ""
echo "=================================================================="
echo "  Demo Complete!"
echo "=================================================================="
echo ""
echo "What we demonstrated:"
echo "  ✅ Created and populated session in LoamSpine"
echo "  ✅ Simulated backup to external storage (NestGate concept)"
echo "  ✅ Simulated local data loss"
echo "  ✅ Restored session from backup"
echo "  ✅ Conceptual inter-primal storage integration"
echo ""
echo "Storage Strategy:"
echo "  Local (LoamSpine):   Fast access, volatile"
echo "  Remote (NestGate):   Durable, distributed"
echo "  Best of both:        Performance + reliability"
echo ""
echo "Gaps discovered:"
echo "  ⚠️  NestGate binary not yet available"
echo "      - Need: ../../bins/nestgate"
echo "      - Capability: 'persistent-storage'"
echo ""
echo "  ⚠️  Session loading not implemented"
echo "      - LoamSpine should load sessions on startup"
echo "      - Check storage directory for existing sessions"
echo "      - Rebuild in-memory state"
echo ""
echo "  ⚠️  Backup mechanism not automatic"
echo "      - Need: Background backup task"
echo "      - Trigger: On session finalization"
echo "      - Policy: Immediate, scheduled, or on-demand"
echo ""
echo "Real NestGate Integration (Future):"
echo ""
echo "  // Discover NestGate via Songbird"
echo "  let storage = songbird"
echo "      .discover_capability('persistent-storage')"
echo "      .await?;"
echo ""
echo "  // Backup session"
echo "  let backup_id = storage"
echo "      .backup_session(session_id)"
echo "      .await?;"
echo ""
echo "  // Restore session"
echo "  let session = storage"
echo "      .restore_session(backup_id)"
echo "      .await?;"
echo ""
echo "Next steps:"
echo "  - Implement: Session loading on startup"
echo "  - Implement: Automatic backup mechanism"
echo "  - Wait for: NestGate binary availability"
echo "  - Integrate: Real NestGate API when ready"
echo "  - Learn: specs/STORAGE_BACKENDS.md"
echo ""

