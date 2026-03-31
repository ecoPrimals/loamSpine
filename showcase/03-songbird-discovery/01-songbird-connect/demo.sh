#!/usr/bin/env bash
# Demo: Songbird Connect - Register LoamSpine with Discovery Service
# This is where we discover REAL integration gaps!

set -euo pipefail

# Load common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

# ============================================================================
# DEMO CONFIGURATION
# ============================================================================

readonly DEMO_NAME="songbird-connect"
readonly SONGBIRD_PORT="${SONGBIRD_PORT:-8082}"
readonly SONGBIRD_URL="http://127.0.0.1:${SONGBIRD_PORT}"

# ============================================================================
# CLEANUP
# ============================================================================

cleanup() {
    log_info "Cleaning up..."
    # Stop Songbird if we started it
    if [ -f "${LOGS_DIR}/songbird.pid" ]; then
        "${SCRIPT_DIR}/../../scripts/stop_songbird.sh" || true
    fi
}

register_cleanup cleanup

# ============================================================================
# MAIN DEMO
# ============================================================================

main() {
    log_header "🦴 Songbird Connect - Discovery Integration"
    
    log_info "This demo shows:"
    log_info "  • Starting Songbird orchestrator from ../bins/"
    log_info "  • Connecting LoamSpine to Songbird"
    log_info "  • Advertising LoamSpine capabilities"
    log_info "  • O(n) discovery architecture (vs O(n²))"
    log_info "  • **REAL BINARY - NO MOCKS!**"
    echo ""
    
    demo_pause
    
    # ========================================================================
    # Step 1: Start Songbird
    # ========================================================================
    
    log_step "Step 1: Starting Songbird orchestrator..."
    
    log_info "Using real binary from: ../bins/songbird-orchestrator"
    log_info "Port: ${SONGBIRD_PORT}"
    echo ""
    
    demo_pause "Press Enter to start Songbird..."
    
    if "${SCRIPT_DIR}/../../scripts/start_songbird.sh"; then
        log_success "Songbird started!"
    else
        log_error "Failed to start Songbird"
        log_warning "**GAP DISCOVERED**: Need to document Songbird startup requirements"
        log_info "Showing expected behavior in demo mode..."
        DEMO_MODE=1
    fi
    
    echo ""
    demo_pause
    
    # ========================================================================
    # Step 2: Check Songbird Health
    # ========================================================================
    
    log_step "Step 2: Checking Songbird health..."
    
    if [ "${DEMO_MODE:-0}" != "1" ]; then
        if curl -s -f "${SONGBIRD_URL}/health" > /dev/null 2>&1; then
            log_success "Songbird is healthy!"
            
            # Get version info if available
            VERSION=$(curl -s "${SONGBIRD_URL}/version" 2>/dev/null || echo "unknown")
            log_info "Version: ${VERSION}"
        else
            log_warning "Songbird health check failed"
            log_warning "**GAP DISCOVERED**: May need different health endpoint"
            DEMO_MODE=1
        fi
    fi
    
    if [ "${DEMO_MODE:-0}" = "1" ]; then
        log_warning "[DEMO MODE] Simulating Songbird health check..."
        log_info "Expected response: {\"status\": \"healthy\", \"version\": \"0.1.0\"}"
    fi
    
    echo ""
    demo_pause
    
    # ========================================================================
    # Step 3: Register LoamSpine
    # ========================================================================
    
    log_step "Step 3: Registering LoamSpine capabilities..."
    
    log_info "LoamSpine capabilities to advertise:"
    log_info "  • loamspine.permanence"
    log_info "  • loamspine.certificates"
    log_info "  • loamspine.proofs"
    log_info "  • loamspine.spine_management"
    echo ""
    
    if [ "${DEMO_MODE:-0}" != "1" ]; then
        log_info "Attempting registration..."
        
        # Try to register (this will reveal the actual API)
        REGISTER_DATA='{
            "service_name": "loamspine",
            "endpoint": "http://127.0.0.1:9001",
            "capabilities": [
                "loamspine.permanence",
                "loamspine.certificates",
                "loamspine.proofs"
            ],
            "metadata": {
                "version": "0.9.15",
                "protocol": "tarpc"
            }
        }'
        
        if RESPONSE=$(curl -s -f -X POST "${SONGBIRD_URL}/register" \
            -H "Content-Type: application/json" \
            -d "${REGISTER_DATA}" 2>&1); then
            
            log_success "Registration successful!"
            log_info "Response: ${RESPONSE}"
        else
            log_warning "Registration failed"
            log_warning "**GAP DISCOVERED**: Need to understand Songbird registration API"
            log_info "Error: ${RESPONSE}"
            DEMO_MODE=1
        fi
    fi
    
    if [ "${DEMO_MODE:-0}" = "1" ]; then
        log_warning "[DEMO MODE] Simulating registration..."
        cat <<'EOF'

Expected registration flow:

1. POST /register
   {
     "service_name": "loamspine",
     "endpoint": "http://127.0.0.1:9001",
     "capabilities": [...],
     "metadata": {...}
   }

2. Response:
   {
     "status": "registered",
     "service_id": "loamspine-abc123",
     "ttl": 300
   }

3. Heartbeat required every 60 seconds

EOF
    fi
    
    echo ""
    demo_pause
    
    # ========================================================================
    # Step 4: Discover Services
    # ========================================================================
    
    log_step "Step 4: Testing discovery..."
    
    log_info "Query: Find all services with 'signing' capability"
    echo ""
    
    if [ "${DEMO_MODE:-0}" != "1" ]; then
        if SERVICES=$(curl -s -f "${SONGBIRD_URL}/discover?capability=signing" 2>&1); then
            log_success "Discovery works!"
            log_info "Services found:"
            echo "${SERVICES}" | jq '.' 2>/dev/null || echo "${SERVICES}"
        else
            log_warning "Discovery endpoint not working as expected"
            log_warning "**GAP DISCOVERED**: Need Songbird discovery API docs"
            DEMO_MODE=1
        fi
    fi
    
    if [ "${DEMO_MODE:-0}" = "1" ]; then
        log_warning "[DEMO MODE] Simulating discovery..."
        cat <<'EOF'

Expected discovery response:

GET /discover?capability=signing

Response:
[
  {
    "service_name": "beardog",
    "endpoint": "http://127.0.0.1:9000",
    "capabilities": ["signing", "crypto"],
    "last_heartbeat": "2025-12-24T18:10:00Z"
  }
]

This is O(n) - query one service (Songbird) instead of n services!

EOF
    fi
    
    echo ""
    demo_pause
    
    # ========================================================================
    # Step 5: Architecture Explanation
    # ========================================================================
    
    log_header "🏗️ O(n) vs O(n²) Discovery"
    
    log_section "WITHOUT Songbird (O(n²))"
    cat <<'EOF'

Problem: Each primal must know about every other primal

LoamSpine needs to find:
  • BearDog (signing)
  • NestGate (storage)
  • ToadStool (compute)
  • Squirrel (AI)

LoamSpine → queries → BearDog
LoamSpine → queries → NestGate  
LoamSpine → queries → ToadStool
LoamSpine → queries → Squirrel

As ecosystem grows:
  • n primals = n² connections
  • Each primal hardcodes n-1 others
  • Configuration nightmare!

EOF
    
    demo_pause
    
    log_section "WITH Songbird (O(n))"
    cat <<'EOF'

Solution: Universal adapter pattern

Each primal → registers → Songbird
LoamSpine → queries → Songbird → finds all

LoamSpine asks Songbird: "Who has signing?"
Songbird responds: "BearDog at http://..."

Benefits:
  • n primals = n connections (to Songbird)
  • Zero hardcoding between primals
  • Dynamic discovery
  • Scale effortlessly!

EOF
    
    demo_pause
    
    # ========================================================================
    # Step 6: Gaps Discovered
    # ========================================================================
    
    log_header "🔍 Gaps Discovered in This Demo"
    
    log_section "Gap #3: Songbird Integration Details"
    cat <<'EOF'

What we learned:
  • Songbird binary exists and is executable
  • Need to understand startup flags
  • Need to document registration API
  • Need to document discovery endpoints
  • Need to implement heartbeat logic

Actions needed:
  1. Document Songbird CLI options
  2. Create registration helper in LoamSpine
  3. Implement heartbeat keep-alive
  4. Add graceful reconnection logic

This is GOOD - we're discovering real integration needs!

EOF
    
    echo ""
    demo_pause
    
    # ========================================================================
    # Summary
    # ========================================================================
    
    log_header "🎉 Demo Complete!"
    
    log_success "You've learned:"
    log_info "  ✅ How to start Songbird from ../bins/"
    log_info "  ✅ O(n) vs O(n²) discovery architecture"
    log_info "  ✅ Capability-based service discovery"
    log_info "  ✅ **Real integration gaps discovered!**"
    echo ""
    
    log_warning "Gaps discovered:"
    log_info "  • Songbird API documentation needed"
    log_info "  • Registration helper function needed"
    log_info "  • Heartbeat mechanism needed"
    log_info "  • Reconnection logic needed"
    echo ""
    
    log_info "These gaps are VALUABLE - they show what to build next!"
    echo ""
    
    log_info "Next steps:"
    log_info "  • Document Songbird API from real binary"
    log_info "  • Implement SongbirdClient improvements"
    log_info "  • Try 02-capability-discovery for more details"
    log_info "  • Move to inter-primal integration"
    echo ""
    
    # Generate receipt
    generate_receipt "${DEMO_NAME}" "discovery" \
        "Started Songbird from ../bins/ (or attempted)" \
        "Tested health and registration endpoints" \
        "Demonstrated O(n) discovery architecture" \
        "**DISCOVERED REAL INTEGRATION GAPS**" \
        "Gap #3: Songbird API integration details"
    
    log_success "Demo completed - gaps discovered! 🦴"
}

# Run main
main "$@"

