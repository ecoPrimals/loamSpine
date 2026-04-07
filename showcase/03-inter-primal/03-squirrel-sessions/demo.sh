#!/bin/bash
# 🦴 LoamSpine + 🐿️ Squirrel Integration Demo
#
# **NO MOCKS** - Uses real Squirrel binary from ../../../bins/squirrel
#
# This demo shows LoamSpine anchoring AI/ML session commits from Squirrel.
# We discover integration patterns and document evolution needs.
#
# Time: 15-20 minutes
# Prerequisites: squirrel binary at ../../../bins/squirrel

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# Configuration
SQUIRREL_BIN="../../../bins/squirrel"
SQUIRREL_CLI="../../../bins/squirrel-cli"
LOAMSPINE_PORT=9001
SQUIRREL_PORT=9002
OUTPUT_DIR="./outputs/squirrel-integration-$(date +%s)"
RECEIPT_FILE="$OUTPUT_DIR/receipt.txt"

print_header() {
    echo ""
    echo -e "${CYAN}================================================================${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}================================================================${NC}"
    echo ""
}

print_step() {
    echo -e "${BLUE}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${MAGENTA}ℹ $1${NC}"
}

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Start receipt
{
    echo "🦴 LoamSpine + 🐿️ Squirrel Integration Demo"
    echo "============================================="
    echo "Date: $(date)"
    echo "Squirrel Binary: $SQUIRREL_BIN"
    echo ""
} > "$RECEIPT_FILE"

print_header "🦴 LoamSpine + 🐿️ Squirrel: AI Session Permanence"

cat << 'EOF'
This demo shows LoamSpine providing permanence for Squirrel AI sessions.

What we'll demonstrate:
  1. Check Squirrel binary availability
  2. Understand Squirrel's session model
  3. Simulate AI session completion
  4. Attempt session commit to LoamSpine
  5. Verify permanent anchoring
  6. Document integration patterns

Why this matters:
  • Squirrel = Ephemeral AI sessions (RAG, inference)
  • LoamSpine = Permanent record (audit, replay, provenance)
  • Together = Trustworthy AI with full history

Use Cases:
  • Research: Track all AI experiments
  • Compliance: Audit AI decision-making
  • Reproducibility: Replay exact sessions
  • Provenance: Prove AI model and data used

Philosophy: Real AI workloads, real permanence needs!
EOF

echo ""
echo -e "${YELLOW}Press ENTER to continue...${NC}"
read -r

# ============================================================================
# Step 1: Check Squirrel Binary
# ============================================================================

print_step "Step 1: Checking Squirrel binaries..."

if [ ! -f "$SQUIRREL_BIN" ]; then
    print_error "Squirrel binary not found at: $SQUIRREL_BIN"
    exit 1
fi

if [ ! -x "$SQUIRREL_BIN" ]; then
    chmod +x "$SQUIRREL_BIN"
fi

SQUIRREL_SIZE=$(du -h "$SQUIRREL_BIN" | cut -f1)
print_success "Squirrel binary found (${SQUIRREL_SIZE})"

if [ -f "$SQUIRREL_CLI" ]; then
    if [ ! -x "$SQUIRREL_CLI" ]; then
        chmod +x "$SQUIRREL_CLI"
    fi
    SQUIRREL_CLI_SIZE=$(du -h "$SQUIRREL_CLI" | cut -f1)
    print_success "Squirrel CLI found (${SQUIRREL_CLI_SIZE})"
fi

{
    echo "Step 1: Squirrel Binary Check"
    echo "  Server: $SQUIRREL_BIN (${SQUIRREL_SIZE})"
    echo "  CLI: $SQUIRREL_CLI (${SQUIRREL_CLI_SIZE:-N/A})"
    echo "  Status: ✓ Found and executable"
    echo ""
} >> "$RECEIPT_FILE"

# ============================================================================
# Step 2: Discover Squirrel Capabilities
# ============================================================================

print_step "Step 2: Discovering Squirrel capabilities..."

echo ""
print_info "Querying Squirrel for session capabilities..."
echo ""

if "$SQUIRREL_BIN" --help > "$OUTPUT_DIR/squirrel-help.txt" 2>&1; then
    print_success "Squirrel help retrieved"
    echo ""
    echo "Squirrel capabilities:"
    head -30 "$OUTPUT_DIR/squirrel-help.txt" | sed 's/^/  /'
    echo ""
else
    print_warning "Squirrel help not available via --help"
fi

{
    echo "Step 2: Squirrel Capabilities"
    echo "  Purpose: AI/ML session management"
    echo "  Sessions: Ephemeral RAG, inference, training"
    echo "  Integration: Session commits to permanent ledger"
    echo ""
} >> "$RECEIPT_FILE"

# ============================================================================
# Step 3: Integration Pattern Analysis
# ============================================================================

print_step "Step 3: Analyzing session commit pattern..."

cat << 'EOF'

LoamSpine Session Commit Pattern:
─────────────────────────────────────────────────────────────

The Integration Flow:
  1. Squirrel: Run AI session (RAG query, inference, etc.)
  2. Squirrel: Generate session summary (metadata, results, provenance)
  3. Squirrel → LoamSpine: Commit session via RPC
  4. LoamSpine: Create permanent entry with session data
  5. LoamSpine → Squirrel: Return commitment proof
  6. Squirrel: Store proof for future reference

What Gets Committed:
  • Session ID (unique identifier)
  • Timestamp (when session completed)
  • Model used (which AI model)
  • Prompt/query (what was asked)
  • Result summary (what was found)
  • Data sources (which documents/embeddings)
  • Merkle root (for full DAG integrity)
  • Committer DID (who ran this)

Why This Matters:
  • Audit trail: Every AI interaction recorded
  • Reproducibility: Can replay exact session
  • Trust: Cryptographic proof of what happened
  • Compliance: Meets regulatory requirements
  • Research: Track all experiments permanently

LoamSpine's commit_session API:
  RPC Method: commit_session
  Request:
    - spine_id: Target spine
    - session_id: Unique session identifier
    - committer: DID of session owner
    - summary: Session metadata
      - merkle_root: DAG integrity hash
      - vertex_count: Number of operations
      - metadata: Custom session data

  Response:
    - entry_hash: Permanent commitment hash
    - timestamp: When committed
    - proof: Inclusion proof

EOF

{
    echo "Step 3: Session Commit Pattern"
    echo "  Flow: Squirrel session → Summary → LoamSpine commit → Proof"
    echo "  API: commit_session (RPC method)"
    echo "  Data: Session metadata, merkle root, provenance"
    echo ""
} >> "$RECEIPT_FILE"

# ============================================================================
# Step 4: Simulate AI Session
# ============================================================================

print_step "Step 4: Simulating AI session completion..."

# Create simulated session data
SESSION_ID="session_$(date +%s)_demo"
SESSION_DATA_FILE="$OUTPUT_DIR/session-summary.json"

cat > "$SESSION_DATA_FILE" << SESSION_EOF
{
  "session_id": "$SESSION_ID",
  "session_type": "RAG_Query",
  "timestamp": "$(date -Iseconds)",
  "model": "mistral-7b-instruct",
  "prompt": "What are the key principles of sovereign data?",
  "result_summary": "Found 5 relevant documents emphasizing user control, no surveillance, and open standards",
  "data_sources": [
    "doc_ecoPrimals_philosophy.md",
    "doc_human_dignity_principles.md",
    "doc_sovereign_architecture.md"
  ],
  "vertex_count": 127,
  "merkle_root": "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a",
  "committer": "did:key:z6MkSquirrelDemo",
  "metadata": {
    "embeddings_used": "all-MiniLM-L6-v2",
    "chunk_count": 42,
    "relevance_threshold": 0.75,
    "execution_time_ms": 234
  }
}
SESSION_EOF

print_success "AI session simulated"
echo ""
echo "Session summary:"
cat "$SESSION_DATA_FILE" | jq '.' 2>/dev/null || cat "$SESSION_DATA_FILE" | sed 's/^/  /'
echo ""

{
    echo "Step 4: AI Session Simulation"
    echo "  Session ID: $SESSION_ID"
    echo "  Type: RAG Query"
    echo "  Model: mistral-7b-instruct"
    echo "  Operations: 127 vertices"
    echo ""
} >> "$RECEIPT_FILE"

# ============================================================================
# Step 5: Attempt Session Commit
# ============================================================================

print_step "Step 5: Attempting session commit to LoamSpine..."

cat << 'EOF'

Commit Attempt:
─────────────────────────────────────────────────────────────

We'll attempt to commit this session using:
  1. LoamSpine's commit_session RPC method
  2. Squirrel's CLI (if available)
  3. Direct API call
  4. Document what works and gaps discovered

This reveals the actual integration pattern!

EOF

# Try via LoamSpine RPC (if service is running)
print_info "Attempting commit via LoamSpine RPC..."

COMMIT_PAYLOAD=$(cat << COMMIT_EOF
{
  "spine_id": "spine_demo_squirrel_sessions",
  "session_id": "$SESSION_ID",
  "committer": "did:key:z6MkSquirrelDemo",
  "summary": {
    "merkle_root": "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a",
    "vertex_count": 127,
    "metadata": $(cat "$SESSION_DATA_FILE")
  }
}
COMMIT_EOF
)

if curl -X POST "http://localhost:${LOAMSPINE_PORT}/api/v1/commit_session" \
     -H "Content-Type: application/json" \
     -d "$COMMIT_PAYLOAD" \
     > "$OUTPUT_DIR/commit-response.txt" 2>&1; then
    print_success "Session commit succeeded!"
    echo ""
    echo "Commitment response:"
    cat "$OUTPUT_DIR/commit-response.txt" | jq '.' 2>/dev/null || cat "$OUTPUT_DIR/commit-response.txt" | sed 's/^/  /'
    echo ""
    
    {
        echo "Step 5: Session Commit"
        echo "  Method: LoamSpine RPC"
        echo "  Result: ✓ SUCCESS"
        echo "  Response: $(cat "$OUTPUT_DIR/commit-response.txt")"
        echo ""
    } >> "$RECEIPT_FILE"
else
    print_warning "LoamSpine RPC not available (service may not be running)"
    print_info "This is expected - we're discovering the integration pattern"
    echo ""
    
    {
        echo "Step 5: Session Commit"
        echo "  Method: LoamSpine RPC"
        echo "  Result: ⚠ GAP DISCOVERED"
        echo "  Issue: LoamSpine service not running"
        echo "  Next: Test with actual LoamSpine service"
        echo ""
    } >> "$RECEIPT_FILE"
fi

# Try via Squirrel CLI (if it has commit capability)
if [ -f "$SQUIRREL_CLI" ]; then
    print_info "Attempting commit via Squirrel CLI..."
    
    if "$SQUIRREL_CLI" commit --session "$SESSION_ID" --loamspine "http://localhost:${LOAMSPINE_PORT}" \
         > "$OUTPUT_DIR/cli-commit.txt" 2>&1; then
        print_success "CLI commit succeeded!"
        
        {
            echo "  Alternative Method: Squirrel CLI"
            echo "  Result: ✓ SUCCESS"
            echo ""
        } >> "$RECEIPT_FILE"
    else
        print_warning "Squirrel CLI commit not available"
        
        {
            echo "  Alternative Method: Squirrel CLI"
            echo "  Result: ⚠ GAP DISCOVERED"
            echo "  Issue: CLI interface needs discovery"
            echo ""
        } >> "$RECEIPT_FILE"
    fi
fi

# ============================================================================
# Step 6: Document Integration Gaps
# ============================================================================

print_step "Step 6: Documenting integration gaps..."

cat << 'EOF'

╔══════════════════════════════════════════════════════════════╗
║                    GAPS DISCOVERED                           ║
╚══════════════════════════════════════════════════════════════╝

Integration Gaps Identified:
─────────────────────────────────────────────────────────────

1. Service Discovery
   • Need: How does Squirrel find LoamSpine?
   • Current: Hardcoded endpoint
   • Evolution: Use Songbird for runtime discovery

2. Commit API Format
   • Need: Align commit_session payload format
   • Questions: JSON schema? Required fields?
   • Evolution: Document and test actual API

3. Session Metadata Schema
   • Need: Standard session summary format
   • Questions: What fields are required? Optional?
   • Evolution: Define SessionSummary schema

4. Proof Handling
   • Need: What does Squirrel do with the proof?
   • Questions: Store? Verify later? Share?
   • Evolution: Document proof lifecycle

5. Error Handling
   • Need: What if LoamSpine is unavailable?
   • Current: Unknown fallback behavior
   • Evolution: Queue commits? Retry? Fail gracefully?

6. Batch Commits
   • Need: Can Squirrel commit multiple sessions at once?
   • Questions: Batch API? Transaction support?
   • Evolution: Design efficient batch commit pattern

7. Query/Retrieval
   • Need: How does Squirrel query past sessions?
   • Questions: By session_id? By date? By model?
   • Evolution: Design query interface

8. Authentication
   • Need: How does LoamSpine verify committer?
   • Questions: DID-based? Signature? Token?
   • Evolution: Implement auth mechanism

Positive Discoveries:
─────────────────────────────────────────────────────────────
✓ Clear use case (AI provenance)
✓ Well-defined data model (session summary)
✓ Existing commit_session API
✓ Strong value proposition

Next Steps:
─────────────────────────────────────────────────────────────
1. Start LoamSpine service and test actual commit
2. Document commit_session API precisely
3. Create SessionSummary schema
4. Implement in Squirrel
5. Add integration tests
6. Document pattern for other AI primals

EOF

{
    echo "Step 6: Gaps & Evolution"
    echo "──────────────────────────────────"
    echo "Gaps Discovered:"
    echo "  1. Service discovery (hardcoded vs runtime)"
    echo "  2. Commit API format alignment"
    echo "  3. Session metadata schema needed"
    echo "  4. Proof handling lifecycle unclear"
    echo "  5. Error handling/fallback undefined"
    echo "  6. Batch commit pattern needed"
    echo "  7. Query/retrieval interface missing"
    echo "  8. Authentication mechanism unclear"
    echo ""
    echo "Evolution Priorities:"
    echo "  1. Test with actual LoamSpine service"
    echo "  2. Document commit_session API"
    echo "  3. Define SessionSummary schema"
    echo "  4. Implement in Squirrel"
    echo "  5. Add discovery via Songbird"
    echo ""
} >> "$RECEIPT_FILE"

# ============================================================================
# Step 7: Value Proposition
# ============================================================================

print_step "Step 7: Understanding the value..."

cat << 'EOF'

╔══════════════════════════════════════════════════════════════╗
║                    VALUE PROPOSITION                         ║
╚══════════════════════════════════════════════════════════════╝

Why LoamSpine + Squirrel is Transformative:
─────────────────────────────────────────────────────────────

Squirrel Provides:
  • RAG (Retrieval-Augmented Generation)
  • Fast inference and querying
  • Embedding-based search
  • Ephemeral session management

LoamSpine Provides:
  • Permanent session records
  • Cryptographic proofs
  • Audit trails
  • Provenance tracking

Together They Enable:
  • Trustworthy AI: Every decision is recorded
  • Reproducible Research: Replay exact sessions
  • Compliance: Audit trails for regulations
  • Experiment Tracking: Never lose research progress
  • Model Provenance: Prove which model was used
  • Data Provenance: Prove which data was accessed

Real-World Scenarios:
  
  1. Medical AI:
     • Record all diagnostic AI sessions
     • Prove which model and data used
     • Compliance with HIPAA
     • Liability protection

  2. Financial AI:
     • Audit all AI trading decisions
     • Regulatory compliance
     • Risk management
     • Fraud detection provenance

  3. Research AI:
     • Track all experiments permanently
     • Reproducibility guarantee
     • Paper citations with proofs
     • Grant reporting

  4. Legal AI:
     • Record all legal research sessions
     • Discovery requirements
     • Attorney-client privilege audit
     • Case provenance

  5. Personal AI:
     • Your AI assistant's full history
     • Sovereignty over your data
     • Prove AI didn't leak your data
     • Digital legacy

EOF

{
    echo "Value Proposition:"
    echo "  Squirrel: Ephemeral AI sessions (fast, efficient)"
    echo "  LoamSpine: Permanent records (trustworthy, auditable)"
    echo "  Together: Trustworthy AI with full provenance"
    echo ""
    echo "Use Cases: Medical, Financial, Research, Legal, Personal AI"
    echo ""
} >> "$RECEIPT_FILE"

# ============================================================================
# Summary
# ============================================================================

print_header "Demo Complete!"

echo -e "${GREEN}What we accomplished:${NC}"
echo "  ✓ Verified Squirrel binary availability"
echo "  ✓ Analyzed AI session commit pattern"
echo "  ✓ Simulated complete AI session"
echo "  ✓ Attempted real session commit"
echo "  ✓ Discovered integration gaps"
echo "  ✓ Documented evolution path"
echo "  ✓ Understood transformative value"
echo ""

echo -e "${CYAN}Key Learning:${NC}"
echo "  AI provenance is CRITICAL for trustworthy AI."
echo "  LoamSpine provides the missing permanent layer."
echo ""

echo -e "${YELLOW}Outputs:${NC}"
echo "  Receipt: $RECEIPT_FILE"
echo "  Session summary: $SESSION_DATA_FILE"
echo "  Directory: $OUTPUT_DIR"
echo ""

echo -e "${BLUE}Next Steps:${NC}"
echo "  1. Test with running LoamSpine service"
echo "  2. Implement commit_session integration"
echo "  3. Add to Squirrel workflow"
echo "  4. Try next demo: 04-toadstool-compute/"
echo ""

echo -e "${CYAN}Philosophy: Trustworthy AI needs permanent provenance!${NC}"
echo ""

print_success "Receipt saved to: $RECEIPT_FILE"

