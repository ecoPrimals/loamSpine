#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# benchScale full roundtrip validation for loamSpine.
#
# Starts a TCP JSON-RPC server on an ephemeral port, exercises every
# canonical method via HTTP POST, validates responses, and reports results.
# Aligned with the wateringHole DEPLOYMENT_VALIDATION_STANDARD v1.1.
#
# Usage:
#   ./infra/benchScale/validate_roundtrip.sh                       # build + validate
#   SKIP_BUILD=1 ./infra/benchScale/validate_roundtrip.sh          # skip cargo build
#   LOAMSPINE_PORT=8080 ./infra/benchScale/validate_roundtrip.sh   # fixed port
#
# Dependencies: curl, jq, ss (or lsof)

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# ============================================================================
# Configuration
# ============================================================================

JSONRPC_PORT="${LOAMSPINE_PORT:-19710}"
TARPC_PORT="${LOAMSPINE_TARPC_PORT:-19711}"
BIND="${LOAMSPINE_BIND:-127.0.0.1}"
TIMEOUT=5
PASS=0
FAIL=0
SKIP=0
TOTAL=0
SERVER_PID=""
ACTUAL_PORT=""
RPC_ID=0
SERVER_LOG=""

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# ============================================================================
# Helpers
# ============================================================================

cleanup() {
    if [[ -n "$SERVER_PID" ]]; then
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi
    if [[ -n "$SERVER_LOG" && -f "$SERVER_LOG" ]]; then
        rm -f "$SERVER_LOG"
    fi
}
trap cleanup EXIT

log_header() { echo -e "\n${BOLD}${CYAN}═══ $1 ═══${NC}"; }
log_pass()   { echo -e "  ${GREEN}✓${NC} $1"; PASS=$((PASS + 1)); TOTAL=$((TOTAL + 1)); }
log_fail()   { echo -e "  ${RED}✗${NC} $1"; FAIL=$((FAIL + 1)); TOTAL=$((TOTAL + 1)); }

rpc_call() {
    local method="$1"
    local params
    params="${2:-"{}"}"
    RPC_ID=$((RPC_ID + 1))

    local payload
    payload=$(printf '{"jsonrpc":"2.0","method":"%s","params":%s,"id":%s}' "$method" "$params" "$RPC_ID")

    curl -s --max-time "$TIMEOUT" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$payload" \
        "http://${BIND}:${ACTUAL_PORT}/" 2>/dev/null || echo ""
}

assert_result() {
    local label="$1"
    local response="$2"
    local jq_check="${3:-.result}"

    if [[ -z "$response" ]]; then
        log_fail "$label — no response"
        return 1
    fi

    local error
    error=$(echo "$response" | jq -r '.error // empty' 2>/dev/null)
    if [[ -n "$error" && "$error" != "null" ]]; then
        local msg
        msg=$(echo "$response" | jq -r '.error.message // "unknown"' 2>/dev/null)
        log_fail "$label — error: $msg"
        return 1
    fi

    if echo "$response" | jq -e "$jq_check" >/dev/null 2>&1; then
        log_pass "$label"
        return 0
    else
        log_fail "$label — assertion failed: $jq_check"
        return 1
    fi
}

assert_error() {
    local label="$1"
    local response="$2"
    local expected_code="${3:-}"

    if [[ -z "$response" ]]; then
        log_fail "$label — no response"
        return 1
    fi

    local code
    code=$(echo "$response" | jq -r '.error.code // empty' 2>/dev/null)
    if [[ -n "$code" ]]; then
        if [[ -z "$expected_code" || "$code" == "$expected_code" ]]; then
            log_pass "$label (error code: $code)"
            return 0
        fi
    fi
    log_fail "$label — expected error"
    return 1
}

jq_extract() {
    echo "$1" | jq -c -r "$2" 2>/dev/null
}

# ============================================================================
# Build
# ============================================================================

if [[ "${SKIP_BUILD:-}" != "1" ]]; then
    log_header "Building loamSpine"
    cd "$PROJECT_ROOT"
    if ! cargo build --release 2>&1 | tail -3; then
        echo -e "  ${RED}✗${NC} Build failed"
        exit 1
    fi
    echo -e "  ${GREEN}✓${NC} Build complete"
fi

# ============================================================================
# Start server
# ============================================================================

log_header "Starting loamSpine TCP server"

cd "$PROJECT_ROOT"
BINARY="${LOAMSPINE_BINARY:-}"
if [[ -z "$BINARY" ]] || [[ ! -x "$BINARY" ]]; then
    BINARY="$PROJECT_ROOT/target/release/loamspine"
fi
if [[ ! -x "$BINARY" ]]; then
    BINARY="$PROJECT_ROOT/target/debug/loamspine"
fi
if [[ ! -x "$BINARY" ]]; then
    BINARY="$(command -v loamspine 2>/dev/null || true)"
fi

if [[ -z "$BINARY" ]] || [[ ! -x "$BINARY" ]]; then
    echo -e "  ${RED}✗${NC} Binary not found. Run: cargo build --release (or set LOAMSPINE_BINARY)"
    exit 1
fi
echo -e "  Using binary: ${BINARY}"

SERVER_LOG=$(mktemp /tmp/loamspine-benchscale.XXXXXX.log)
ACTUAL_PORT="$JSONRPC_PORT"

# Clean stale sockets from previous runs
rm -f /run/user/"$(id -u)"/biomeos/loamspine.sock 2>/dev/null
rm -f /run/user/"$(id -u)"/biomeos/ledger.sock 2>/dev/null
rm -f /run/user/"$(id -u)"/biomeos/permanence.sock 2>/dev/null

RUST_LOG=warn "$BINARY" server \
    --port "$JSONRPC_PORT" \
    --tarpc-port "$TARPC_PORT" \
    --bind-address "$BIND" \
    >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!

# Wait for TCP readiness
READY=0
for _ in $(seq 1 30); do
    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
        echo -e "  ${RED}✗${NC} Server exited unexpectedly"
        cat "$SERVER_LOG" 2>/dev/null
        exit 1
    fi
    if curl -s --max-time 1 -X POST -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"health.liveness","id":0}' \
        "http://${BIND}:${ACTUAL_PORT}/" 2>/dev/null | jq -e '.result' >/dev/null 2>&1; then
        READY=1
        break
    fi
    sleep 0.3
done

if [[ "$READY" -ne 1 ]]; then
    echo -e "  ${RED}✗${NC} Server not ready after 9s"
    cat "$SERVER_LOG" 2>/dev/null
    exit 1
fi

echo -e "  ${GREEN}✓${NC} Server running (PID=$SERVER_PID, JSON-RPC port=$ACTUAL_PORT)"

TIMESTAMP_NS=$(date +%s)000000000

# ============================================================================
# PHASE 1: Health Triad (DEPLOYMENT_VALIDATION_STANDARD §1)
# ============================================================================

log_header "Phase 1: Health Triad"

resp=$(rpc_call "health.liveness")
assert_result "health.liveness" "$resp" '.result.status == "alive"'

resp=$(rpc_call "health.readiness")
assert_result "health.readiness" "$resp" '.result.ready == true'

resp=$(rpc_call "health.check" '{"include_details":true}')
assert_result "health.check (with details)" "$resp" '.result.status'

resp=$(rpc_call "health.check" '{}')
assert_result "health.check (default)" "$resp" '.result.status'

# ============================================================================
# PHASE 2: Meta / Discovery
# ============================================================================

log_header "Phase 2: Meta & Discovery"

resp=$(rpc_call "capabilities.list")
assert_result "capabilities.list" "$resp" '.result.capabilities'

CAP_COUNT=$(jq_extract "$resp" '.result.capabilities | length')
echo -e "       capabilities: $CAP_COUNT"

resp=$(rpc_call "identity.get")
assert_result "identity.get" "$resp" '.result.primal'

PRIMAL_NAME=$(jq_extract "$resp" '.result.primal')
echo -e "       primal: $PRIMAL_NAME"

resp=$(rpc_call "lifecycle.status")
assert_result "lifecycle.status" "$resp" '.result.uptime_s'

# ============================================================================
# PHASE 3: Auth Gate
# ============================================================================

log_header "Phase 3: Auth Gate"

resp=$(rpc_call "auth.mode")
assert_result "auth.mode" "$resp" '.result.mode'

resp=$(rpc_call "auth.check" '{"method":"health.check"}')
assert_result "auth.check (public method)" "$resp" '.result.allowed'

resp=$(rpc_call "auth.check" '{"method":"spine.create"}')
assert_result "auth.check (protected method)" "$resp" '.result'

resp=$(rpc_call "auth.peer_info")
assert_result "auth.peer_info" "$resp" '.result'

# ============================================================================
# PHASE 4: BTSP Infrastructure
# ============================================================================

log_header "Phase 4: BTSP Infrastructure"

resp=$(rpc_call "btsp.capabilities")
assert_result "btsp.capabilities" "$resp" '.result.ciphers'

resp=$(rpc_call "btsp.negotiate" '{"session_id":"benchscale-test","preferred_cipher":"chacha20-poly1305","ciphers":["chacha20-poly1305"],"client_nonce":"dGVzdA=="}')
assert_result "btsp.negotiate (no session → null)" "$resp" '.result.cipher == "null"'

# ============================================================================
# PHASE 5: Spine CRUD Lifecycle
# ============================================================================

log_header "Phase 5: Spine CRUD Lifecycle"

resp=$(rpc_call "spine.create" '{"owner":"did:key:z6MkBenchScale","name":"benchScale roundtrip"}')
assert_result "spine.create" "$resp" '.result.spine_id'
SPINE_ID=$(jq_extract "$resp" '.result.spine_id')

resp=$(rpc_call "spine.get" "{\"spine_id\":\"$SPINE_ID\"}")
assert_result "spine.get" "$resp" '.result.found == true'

resp=$(rpc_call "spine.list" '{}')
assert_result "spine.list" "$resp" '.result.spine_ids | length > 0'

# ============================================================================
# PHASE 6: Entry Operations
# ============================================================================

log_header "Phase 6: Entry Operations"

DATA_HASH="[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]"
resp=$(rpc_call "entry.append" "{\"spine_id\":\"$SPINE_ID\",\"entry_type\":{\"DataAnchor\":{\"data_hash\":$DATA_HASH,\"mime_type\":\"application/json\",\"size\":1024}},\"committer\":\"did:key:z6MkBenchScale\"}")
assert_result "entry.append" "$resp" '.result.entry_hash'
ENTRY_HASH=$(echo "$resp" | jq -c '.result.entry_hash' 2>/dev/null)

resp=$(rpc_call "entry.get" "{\"spine_id\":\"$SPINE_ID\",\"entry_hash\":$ENTRY_HASH}")
assert_result "entry.get" "$resp" '.result.found == true'

resp=$(rpc_call "entry.get_tip" "{\"spine_id\":\"$SPINE_ID\"}")
assert_result "entry.get_tip" "$resp" '.result.tip_hash'

resp=$(rpc_call "entry.list" "{\"spine_id\":\"$SPINE_ID\"}")
assert_result "entry.list" "$resp" '.result.entries | length > 0'

ENTRY_COUNT=$(jq_extract "$resp" '.result.entries | length')
echo -e "       entries: $ENTRY_COUNT"

# ============================================================================
# PHASE 7: Provenance Integration (session.commit / braid.commit)
# ============================================================================

log_header "Phase 7: Provenance Integration"

SESSION_UUID=$(cat /proc/sys/kernel/random/uuid 2>/dev/null || python3 -c 'import uuid; print(uuid.uuid4())' 2>/dev/null || echo "a0a0a0a0-b1b1-c2c2-d3d3-e4e4e4e4e4e4")
BRAID_UUID=$(cat /proc/sys/kernel/random/uuid 2>/dev/null || python3 -c 'import uuid; print(uuid.uuid4())' 2>/dev/null || echo "b0b0b0b0-c1c1-d2d2-e3e3-f4f4f4f4f4f4")

SESSION_HASH="[42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42]"
BRAID_HASH="[2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2]"

resp=$(rpc_call "session.commit" "{\"spine_id\":\"$SPINE_ID\",\"session_id\":\"$SESSION_UUID\",\"session_hash\":$SESSION_HASH,\"vertex_count\":10,\"committer\":\"did:key:z6MkBenchScale\"}")
assert_result "session.commit" "$resp" '.result.commit_hash'

resp=$(rpc_call "braid.commit" "{\"spine_id\":\"$SPINE_ID\",\"braid_id\":\"$BRAID_UUID\",\"braid_hash\":$BRAID_HASH,\"subjects\":[\"did:key:z6MkAgent1\",\"did:key:z6MkAgent2\"],\"committer\":\"did:key:z6MkBenchScale\"}")
assert_result "braid.commit" "$resp" '.result.commit_hash'

# Alias: provenance.commit → session.commit (exp084 replay attack scenario)
ALIAS_HASH="[99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99]"
resp=$(rpc_call "provenance.commit" "{\"spine_id\":\"$SPINE_ID\",\"session_id\":\"$SESSION_UUID\",\"session_hash\":$ALIAS_HASH,\"vertex_count\":5,\"committer\":\"did:key:z6MkBenchScale\"}")
assert_result "provenance.commit (alias → session.commit)" "$resp" '.result.commit_hash'

# ============================================================================
# PHASE 8: Certificate Operations
# ============================================================================

log_header "Phase 8: Certificate Operations"

CERT_UUID=$(cat /proc/sys/kernel/random/uuid 2>/dev/null || echo "c0c0c0c0-d1d1-e2e2-f3f3-a4a4a4a4a4a4")

resp=$(rpc_call "certificate.mint" "{\"spine_id\":\"$SPINE_ID\",\"cert_type\":{\"AcademicCredential\":{\"institution\":\"benchScale University\",\"credential_type\":\"degree\",\"field_of_study\":\"Validation\",\"date_awarded\":$TIMESTAMP_NS}},\"owner\":\"did:key:z6MkBenchScale\"}")
assert_result "certificate.mint" "$resp" '.result.mint_hash'

resp=$(rpc_call "certificate.get" "{\"certificate_id\":\"$CERT_UUID\"}")
assert_result "certificate.get" "$resp" '.result'

# ============================================================================
# PHASE 9: Slice/Waypoint Operations
# ============================================================================

log_header "Phase 9: Slice/Waypoint Operations"

resp=$(rpc_call "spine.create" '{"owner":"did:key:z6MkBenchScale","name":"waypoint spine"}')
WAYPOINT_ID=$(jq_extract "$resp" '.result.spine_id')

SLICE_UUID=$(cat /proc/sys/kernel/random/uuid 2>/dev/null || echo "d0d0d0d0-e1e1-f2f2-a3a3-b4b4b4b4b4b4")

resp=$(rpc_call "slice.anchor" "{\"waypoint_spine_id\":\"$WAYPOINT_ID\",\"slice_id\":\"$SLICE_UUID\",\"origin_spine_id\":\"$SPINE_ID\",\"committer\":\"did:key:z6MkBenchScale\"}")
assert_result "slice.anchor" "$resp" '.result.anchor_hash'

resp=$(rpc_call "slice.checkout" "{\"waypoint_spine_id\":\"$WAYPOINT_ID\",\"slice_id\":\"$SLICE_UUID\",\"requester\":\"did:key:z6MkBenchScale\"}")
if echo "$resp" | jq -e '.result' >/dev/null 2>&1 || echo "$resp" | jq -e '.error' >/dev/null 2>&1; then
    log_pass "slice.checkout (responded)"
else
    log_fail "slice.checkout — no response"
fi

# ============================================================================
# PHASE 10: Inclusion Proofs
# ============================================================================

log_header "Phase 10: Inclusion Proofs"

resp=$(rpc_call "proof.generate_inclusion" "{\"spine_id\":\"$SPINE_ID\",\"entry_hash\":$ENTRY_HASH}")
assert_result "proof.generate_inclusion" "$resp" '.result.proof'

PROOF=$(echo "$resp" | jq -c '.result.proof' 2>/dev/null)
resp=$(rpc_call "proof.verify_inclusion" "{\"proof\":$PROOF}")
assert_result "proof.verify_inclusion" "$resp" '.result | has("valid")'

# ============================================================================
# PHASE 11: Public Chain Anchoring
# ============================================================================

log_header "Phase 11: Public Chain Anchoring"

resp=$(rpc_call "anchor.publish" "{\"spine_id\":\"$SPINE_ID\",\"anchor_target\":\"Bitcoin\",\"tx_ref\":\"0xbenchscale_btc_txhash\",\"block_height\":850000,\"anchor_timestamp\":$TIMESTAMP_NS}")
assert_result "anchor.publish (Bitcoin)" "$resp" '.result.entry_hash'
ANCHOR_HASH=$(echo "$resp" | jq -c '.result.entry_hash' 2>/dev/null)

resp=$(rpc_call "anchor.verify" "{\"spine_id\":\"$SPINE_ID\",\"anchor_entry_hash\":$ANCHOR_HASH}")
assert_result "anchor.verify (Bitcoin)" "$resp" '.result.verified == true'

resp=$(rpc_call "anchor.publish_batch" "{\"spine_ids\":[\"$SPINE_ID\",\"$WAYPOINT_ID\"],\"anchor_target\":\"Ethereum\",\"tx_ref\":\"0xbenchscale_eth_txhash\",\"block_height\":20000000,\"anchor_timestamp\":$TIMESTAMP_NS}")
assert_result "anchor.publish_batch (Ethereum)" "$resp" '.result.aggregate_root'

resp=$(rpc_call "anchor.publish" "{\"spine_id\":\"$SPINE_ID\",\"anchor_target\":{\"Rfc3161Tsa\":{\"tsa_url\":\"https://freetsa.org/tsr\"}},\"tx_ref\":\"base64-tst-token\",\"block_height\":0,\"anchor_timestamp\":$TIMESTAMP_NS}")
assert_result "anchor.publish (Rfc3161Tsa)" "$resp" '.result.entry_hash'

# ============================================================================
# PHASE 12: Bond Ledger
# ============================================================================

log_header "Phase 12: Bond Ledger"

resp=$(rpc_call "bonding.ledger.store" '{"bond_id":"bond-benchscale-001","data":{"type":"Covalent","partners":["did:key:z6MkAlpha","did:key:z6MkBeta"]}}')
assert_result "bonding.ledger.store" "$resp" '.result.status == "stored"'

resp=$(rpc_call "bonding.ledger.retrieve" '{"bond_id":"bond-benchscale-001"}')
assert_result "bonding.ledger.retrieve" "$resp" '.result.data.type == "Covalent"'

resp=$(rpc_call "bonding.ledger.list" '{}')
assert_result "bonding.ledger.list" "$resp" '.result.bonds | length > 0'

# ============================================================================
# PHASE 13: Permanence Compat Layer
# ============================================================================

log_header "Phase 13: Permanence Compat Layer"

resp=$(rpc_call "permanence.health_check")
assert_result "permanence.health_check" "$resp" '.result'

PERM_MERKLE=$( printf '%0.s07' $(seq 1 32) )
resp=$(rpc_call "permanence.commit_session" "{\"session_id\":\"$SESSION_UUID\",\"merkle_root\":\"$PERM_MERKLE\",\"summary\":{\"session_type\":\"benchscale\",\"vertex_count\":3,\"leaf_count\":2,\"started_at\":1000000,\"ended_at\":2000000,\"outcome\":\"committed\"},\"committer_did\":\"did:key:z6MkBenchScale\"}")
assert_result "permanence.commit_session" "$resp" '.result.accepted == true'

PERM_ENTRY_HASH=$(jq_extract "$resp" '.result.spine_entry_hash // "0000"')
PERM_SPINE_ID=$(jq_extract "$resp" '.result.spine_id // ""')

resp=$(rpc_call "permanence.get_commit" "{\"spine_id\":\"$PERM_SPINE_ID\",\"entry_hash\":\"$PERM_ENTRY_HASH\",\"index\":0}")
assert_result "permanence.get_commit" "$resp" '.result'

resp=$(rpc_call "permanence.verify_commit" "{\"spine_id\":\"$PERM_SPINE_ID\",\"entry_hash\":\"$PERM_ENTRY_HASH\",\"index\":0}")
assert_result "permanence.verify_commit" "$resp" '.result'

# ============================================================================
# PHASE 14: MCP Tool Discovery
# ============================================================================

log_header "Phase 14: MCP Tool Discovery"

resp=$(rpc_call "tools.list")
assert_result "tools.list" "$resp" '.result.tools | length > 0'

TOOL_COUNT=$(jq_extract "$resp" '.result.tools | length')
echo -e "       MCP tools: $TOOL_COUNT"

resp=$(rpc_call "tools.call" '{"name":"spine_list","arguments":{}}')
assert_result "tools.call (spine_list)" "$resp" '.result.content'

# ============================================================================
# PHASE 15: Method Alias Roundtrip
# ============================================================================

log_header "Phase 15: Method Alias Roundtrip"

resp=$(rpc_call "capability.list")
assert_result "capability.list (alias → capabilities.list)" "$resp" '.result.capabilities'

resp=$(rpc_call "primal.capabilities")
assert_result "primal.capabilities (alias → capabilities.list)" "$resp" '.result.capabilities'

resp=$(rpc_call "permanent-storage.healthCheck")
assert_result "permanent-storage.healthCheck (alias → permanence.health_check)" "$resp" '.result'

# ============================================================================
# PHASE 16: Seal Spine (terminal operation)
# ============================================================================

log_header "Phase 16: Seal Spine (terminal)"

resp=$(rpc_call "spine.seal" "{\"spine_id\":\"$SPINE_ID\",\"sealer\":\"did:key:z6MkBenchScale\"}")
assert_result "spine.seal" "$resp" '.result.success == true'

DATA_HASH2="[9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9]"
resp=$(rpc_call "entry.append" "{\"spine_id\":\"$SPINE_ID\",\"entry_type\":{\"DataAnchor\":{\"data_hash\":$DATA_HASH2,\"mime_type\":\"text/plain\",\"size\":64}},\"committer\":\"did:key:z6MkBenchScale\"}")
assert_error "sealed spine rejects append" "$resp"

# ============================================================================
# PHASE 17: Error Handling
# ============================================================================

log_header "Phase 17: Error Handling"

resp=$(rpc_call "nonexistent.method" '{}')
assert_error "unknown method → -32601" "$resp" "-32601"

resp=$(rpc_call "spine.get" '{"spine_id":"00000000-0000-0000-0000-000000000000"}')
assert_result "spine.get (nil UUID → found=false)" "$resp" '.result.found == false'

# ============================================================================
# PHASE 18: Lifecycle Uptime Verification
# ============================================================================

log_header "Phase 18: Lifecycle Uptime Verification"

resp=$(rpc_call "lifecycle.status")
UPTIME=$(jq_extract "$resp" '.result.uptime_s')
if [[ -n "$UPTIME" && "$UPTIME" != "null" ]]; then
    log_pass "lifecycle.status uptime_s=$UPTIME"
else
    log_fail "lifecycle.status missing uptime_s"
fi

# ============================================================================
# PHASE 19: primal.announce
# ============================================================================

log_header "Phase 19: primal.announce"

resp=$(rpc_call "primal.announce")
assert_result "primal.announce" "$resp" '.result'

# ============================================================================
# Phase 20: Health probe burst (runtime-in-runtime regression test)
# ============================================================================

log_header "Phase 20: Health probe burst (runtime-in-runtime regression)"

for i in $(seq 1 20); do
    rpc_call "health.liveness" >/dev/null
    rpc_call "health.readiness" >/dev/null
done

if ! kill -0 "$SERVER_PID" 2>/dev/null; then
    echo -e "  ${RED}✗${NC} Server crashed during health burst — possible runtime-in-runtime panic"
    ((FAIL++)); ((TOTAL++))
else
    resp=$(rpc_call "health.liveness")
    assert_result "health.liveness (after 40 rapid probes)" "$resp" '.result.status'

    resp=$(rpc_call "health.readiness")
    assert_result "health.readiness (after 40 rapid probes)" "$resp" '.result.ready == true'

    resp=$(rpc_call "health.check")
    assert_result "health.check (after 40 rapid probes)" "$resp" '.result'
fi

# ============================================================================
# Summary
# ============================================================================

echo ""
log_header "benchScale Roundtrip Validation Complete"
echo ""
echo -e "  ${BOLD}Total:${NC}   $TOTAL"
echo -e "  ${GREEN}Passed:${NC}  $PASS"
echo -e "  ${RED}Failed:${NC}  $FAIL"
echo -e "  ${YELLOW}Skipped:${NC} $SKIP"
echo ""

if [[ $FAIL -eq 0 ]]; then
    echo -e "  ${GREEN}${BOLD}ALL ROUNDTRIP VALIDATIONS PASSED${NC}"
    echo ""
    echo -e "  Port:        $ACTUAL_PORT"
    echo -e "  PID:         $SERVER_PID"
    echo -e "  Methods:     $PASS validations across 43 canonical methods"
    echo -e "  Phases:      20 validation phases"
    echo -e "  Transport:   HTTP POST JSON-RPC 2.0"
    echo -e "  Standard:    DEPLOYMENT_VALIDATION_STANDARD v1.1"
    echo ""
    exit 0
else
    echo -e "  ${RED}${BOLD}$FAIL VALIDATIONS FAILED${NC}"
    echo ""
    exit 1
fi
