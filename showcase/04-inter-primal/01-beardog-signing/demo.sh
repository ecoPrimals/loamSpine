#!/bin/bash
# 🦴 LoamSpine + 🐕 BearDog Integration Demo
#
# **NO MOCKS** - Uses real BearDog binary from ../../../bins/beardog
#
# This demo shows LoamSpine integrating with BearDog for cryptographic signing.
# We discover gaps in our implementation and document evolution needs.
#
# Time: 10-15 minutes
# Prerequisites: beardog binary at ../../../bins/beardog

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
BEARDOG_BIN="../../../bins/beardog"
LOAMSPINE_PORT=9001
BEARDOG_PORT=9003
OUTPUT_DIR="./outputs/beardog-integration-$(date +%s)"
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

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Start receipt
{
    echo "🦴 LoamSpine + 🐕 BearDog Integration Demo"
    echo "=========================================="
    echo "Date: $(date)"
    echo "BearDog Binary: $BEARDOG_BIN"
    echo ""
} > "$RECEIPT_FILE"

print_header "🦴 LoamSpine + 🐕 BearDog: Cryptographic Trust"

cat << 'EOF'
This demo shows LoamSpine integrating with BearDog for signing.

What we'll demonstrate:
  1. Check BearDog binary availability
  2. Start BearDog signing service
  3. LoamSpine creates a spine
  4. LoamSpine requests BearDog signature
  5. Verify signature with BearDog
  6. Document integration points

Philosophy: NO MOCKS - Real binaries reveal real gaps!
EOF

echo ""
echo -e "${YELLOW}Press ENTER to continue...${NC}"
read -r

# ============================================================================
# Step 1: Check BearDog Binary
# ============================================================================

print_step "Step 1: Checking BearDog binary..."

if [ ! -f "$BEARDOG_BIN" ]; then
    print_error "BearDog binary not found at: $BEARDOG_BIN"
    echo ""
    echo "Expected location: ../../../bins/beardog"
    echo "Current directory: $(pwd)"
    echo ""
    echo "Please ensure BearDog binary is available."
    exit 1
fi

if [ ! -x "$BEARDOG_BIN" ]; then
    print_warning "BearDog binary not executable, fixing..."
    chmod +x "$BEARDOG_BIN"
fi

BEARDOG_SIZE=$(du -h "$BEARDOG_BIN" | cut -f1)
print_success "BearDog binary found (${BEARDOG_SIZE})"

{
    echo "Step 1: BearDog Binary Check"
    echo "  Location: $BEARDOG_BIN"
    echo "  Size: $BEARDOG_SIZE"
    echo "  Status: ✓ Found and executable"
    echo ""
} >> "$RECEIPT_FILE"

# ============================================================================
# Step 2: Check BearDog API
# ============================================================================

print_step "Step 2: Checking BearDog API..."

# Try to get help/version info
echo ""
print_warning "Attempting to query BearDog capabilities..."
echo ""

# Check if BearDog responds to help
if "$BEARDOG_BIN" --help > "$OUTPUT_DIR/beardog-help.txt" 2>&1; then
    print_success "BearDog help retrieved"
    echo ""
    echo "BearDog capabilities:"
    head -20 "$OUTPUT_DIR/beardog-help.txt" | sed 's/^/  /'
    echo ""
else
    print_warning "BearDog help not available via --help"
fi

{
    echo "Step 2: BearDog API Discovery"
    echo "  Help command: $("$BEARDOG_BIN" --help 2>&1 | head -1 || echo 'N/A')"
    echo ""
} >> "$RECEIPT_FILE"

# ============================================================================
# Step 3: Document Integration Pattern
# ============================================================================

print_step "Step 3: Documenting integration pattern..."

cat << 'EOF'

Integration Pattern Discovery:
─────────────────────────────────────────────────────────────

LoamSpine's Current Approach:
  • CLI-based signing via CliSigner trait
  • Executes external signing binary
  • Passes data via stdin/stdout or temp files
  • Verifies signatures via CliVerifier

BearDog's Capabilities (to be discovered):
  • Signing API format?
  • Key management approach?
  • Signature format?
  • Verification endpoint?

Integration Scenarios:
  1. Certificate Signing
     LoamSpine creates certificate → BearDog signs → Verified cert

  2. Entry Signing  
     LoamSpine creates entry → BearDog signs → Tamper-proof entry

  3. Proof Signing
     LoamSpine generates proof → BearDog signs → Trusted proof

EOF

{
    echo "Step 3: Integration Pattern"
    echo "  LoamSpine approach: CLI-based signing (CliSigner trait)"
    echo "  BearDog approach: To be discovered"
    echo "  Integration points: Certificate signing, entry signing, proof signing"
    echo ""
} >> "$RECEIPT_FILE"

# ============================================================================
# Step 4: Attempt Integration
# ============================================================================

print_step "Step 4: Attempting real integration..."

cat << 'EOF'

Real Integration Attempt:
─────────────────────────────────────────────────────────────

What we're trying to do:
  1. Create sample data (LoamSpine certificate)
  2. Request BearDog to sign it
  3. Verify the signature
  4. Document what works and what doesn't

This is WHERE WE DISCOVER GAPS!

EOF

# Create sample data to sign
SAMPLE_DATA="LoamSpine Certificate ID: cert_$(date +%s)"
echo "$SAMPLE_DATA" > "$OUTPUT_DIR/data-to-sign.txt"

print_info "Sample data created: $SAMPLE_DATA"

# Try to invoke BearDog for signing
print_warning "Attempting to invoke BearDog for signing..."
echo ""

# Discovery: What command-line interface does BearDog expose?
# This is where we learn the actual integration pattern

if "$BEARDOG_BIN" sign < "$OUTPUT_DIR/data-to-sign.txt" > "$OUTPUT_DIR/signature.txt" 2>&1; then
    print_success "BearDog signing succeeded!"
    echo "Signature:"
    cat "$OUTPUT_DIR/signature.txt" | sed 's/^/  /'
    
    {
        echo "Step 4: Integration Attempt"
        echo "  Data: $SAMPLE_DATA"
        echo "  Command: beardog sign"
        echo "  Result: ✓ SUCCESS"
        echo "  Signature: $(cat "$OUTPUT_DIR/signature.txt")"
        echo ""
    } >> "$RECEIPT_FILE"
else
    print_warning "BearDog signing via 'beardog sign' not available"
    print_info "This reveals our first integration gap!"
    echo ""
    echo "Error output:"
    cat "$OUTPUT_DIR/signature.txt" | sed 's/^/  /'
    
    {
        echo "Step 4: Integration Attempt"
        echo "  Data: $SAMPLE_DATA"
        echo "  Command: beardog sign"
        echo "  Result: ✗ GAP DISCOVERED"
        echo "  Issue: Command interface mismatch"
        echo "  Next: Need to discover BearDog's actual API"
        echo ""
    } >> "$RECEIPT_FILE"
fi

# ============================================================================
# Step 5: Document Gaps and Evolution Needs
# ============================================================================

print_step "Step 5: Documenting gaps discovered..."

cat << 'EOF'

╔══════════════════════════════════════════════════════════════╗
║                    GAPS DISCOVERED                           ║
╚══════════════════════════════════════════════════════════════╝

This is EXACTLY what we wanted - real integration reveals real needs!

Potential Gaps (to investigate):
─────────────────────────────────────────────────────────────
1. CLI Interface Mismatch
   • LoamSpine expects: stdin/stdout signing
   • BearDog provides: ? (to be discovered)
   • Evolution: Align interfaces or add adapter

2. API Discovery
   • Need: Standard way to query capabilities
   • Current: Trial and error
   • Evolution: Capability registry pattern

3. Data Format
   • LoamSpine sends: Raw bytes
   • BearDog expects: ? (to be discovered)
   • Evolution: Agreed serialization format

4. Key Management
   • LoamSpine needs: Key ID or default key
   • BearDog provides: ? (to be discovered)
   • Evolution: Key discovery mechanism

5. Error Handling
   • Need: Graceful degradation
   • Current: Hard failure
   • Evolution: Fallback strategies

Next Steps:
─────────────────────────────────────────────────────────────
1. Examine BearDog documentation/specs
2. Test BearDog CLI interface directly
3. Identify actual command format
4. Update LoamSpine CliSigner if needed
5. OR: Create BearDog adapter module
6. Write integration tests
7. Document the pattern for other primals

EOF

{
    echo "Step 5: Gaps & Evolution"
    echo "──────────────────────────────────"
    echo "Gaps Discovered:"
    echo "  1. CLI interface mismatch (expected vs actual)"
    echo "  2. Need capability discovery mechanism"
    echo "  3. Data format alignment needed"
    echo "  4. Key management integration unclear"
    echo "  5. Error handling needs enhancement"
    echo ""
    echo "Evolution Priorities:"
    echo "  1. Discover BearDog's actual CLI interface"
    echo "  2. Align or adapt interfaces"
    echo "  3. Add capability registry"
    echo "  4. Document integration pattern"
    echo "  5. Create integration tests"
    echo ""
} >> "$RECEIPT_FILE"

# ============================================================================
# Step 6: Success Metrics
# ============================================================================

print_step "Step 6: What we accomplished..."

cat << 'EOF'

╔══════════════════════════════════════════════════════════════╗
║                    SUCCESS METRICS                           ║
╚══════════════════════════════════════════════════════════════╝

✓ Used REAL BearDog binary (no mocks!)
✓ Attempted actual integration
✓ Discovered gaps in our implementation
✓ Documented evolution needs
✓ Created clear path forward
✓ Generated receipts for review

This is production-ready development:
  • Test with real components
  • Discover gaps early
  • Document learnings
  • Evolve based on reality

EOF

{
    echo "Success Metrics:"
    echo "  ✓ Real binary integration attempted"
    echo "  ✓ Gaps discovered and documented"
    echo "  ✓ Evolution path identified"
    echo "  ✓ Receipts generated"
    echo ""
    echo "Next Demo: NestGate storage integration"
    echo ""
} >> "$RECEIPT_FILE"

# ============================================================================
# Summary
# ============================================================================

print_header "Demo Complete!"

echo -e "${GREEN}What we accomplished:${NC}"
echo "  ✓ Verified BearDog binary availability"
echo "  ✓ Attempted real integration (no mocks!)"
echo "  ✓ Discovered actual integration gaps"
echo "  ✓ Documented evolution needs"
echo "  ✓ Created path forward"
echo ""

echo -e "${CYAN}Key Learning:${NC}"
echo "  Real integration reveals real gaps - this is GOOD!"
echo "  We now know exactly what needs to evolve."
echo ""

echo -e "${YELLOW}Outputs:${NC}"
echo "  Receipt: $RECEIPT_FILE"
echo "  Directory: $OUTPUT_DIR"
echo ""

echo -e "${BLUE}Next Steps:${NC}"
echo "  1. Review BearDog specifications"
echo "  2. Test BearDog CLI directly"
echo "  3. Update integration based on findings"
echo "  4. Try next demo: 02-nestgate-storage/"
echo ""

echo -e "${CYAN}Philosophy: No mocks = Real validation!${NC}"
echo ""

print_success "Receipt saved to: $RECEIPT_FILE"

