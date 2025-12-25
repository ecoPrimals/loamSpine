#!/usr/bin/env bash
# Demo: Entry Types - All 15+ Entry Type Variants
# Shows the full range of entry types LoamSpine supports

set -euo pipefail

# Load common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

# ============================================================================
# DEMO CONFIGURATION
# ============================================================================

readonly DEMO_NAME="entry-types"
readonly DEMO_DATA_DIR="${SCRIPT_DIR}/data"

# ============================================================================
# CLEANUP
# ============================================================================

cleanup() {
    log_info "Cleaning up demo data..."
    rm -rf "${DEMO_DATA_DIR}"
}

register_cleanup cleanup

# ============================================================================
# MAIN DEMO
# ============================================================================

main() {
    log_header "🦴 Entry Types - All 15+ Variants"
    
    log_info "This demo shows:"
    log_info "  • All entry type variants supported by LoamSpine"
    log_info "  • SessionCommit (from ephemeral primals)"
    log_info "  • BraidCommit (from attribution primals)"
    log_info "  • SliceAnchor and SliceReturn (waypoints)"
    log_info "  • Certificate operations"
    log_info "  • And more..."
    echo ""
    
    demo_pause
    
    # ========================================================================
    # Step 1: Run the entry_types example
    # ========================================================================
    
    log_step "Step 1: Running entry types demonstration..."
    
    log_info "We'll run the entry_types example which demonstrates:"
    log_info "  • Genesis entries (chain start)"
    log_info "  • SessionCommit (ephemeral → permanent)"
    log_info "  • BraidCommit (semantic attribution)"
    log_info "  • SliceAnchor (borrowed state waypoints)"
    log_info "  • SliceReturn (return borrowed state)"
    log_info "  • RollupSummary (compression)"
    log_info "  • Certificate operations"
    log_info "  • Metadata and text entries"
    echo ""
    
    demo_pause "Press Enter to see all entry types in action..."
    
    # Run the example
    cd "${PROJECT_ROOT}"
    
    if cargo run --example entry_types 2>&1 | tee "${LOGS_DIR}/${DEMO_NAME}.log"; then
        log_success "Entry types example completed successfully!"
    else
        log_error "Entry types example failed"
        log_info "Check logs at: ${LOGS_DIR}/${DEMO_NAME}.log"
        exit 1
    fi
    
    echo ""
    demo_pause
    
    # ========================================================================
    # Step 2: Explain Key Entry Types
    # ========================================================================
    
    log_header "📚 Key Entry Types Explained"
    
    log_section "1. SessionCommit (Ephemeral → Permanent)"
    log_info "Purpose: Commit ephemeral DAG sessions to permanent spine"
    log_info "Used by: RhizoCrypt and other ephemeral primals"
    log_info "Contains: Session ID, Merkle root, vertex count"
    log_info ""
    log_info "Example use case:"
    log_info "  • Game session ends"
    log_info "  • RhizoCrypt dehydrates to Merkle root"
    log_info "  • LoamSpine commits SessionCommit entry"
    log_info "  • Permanent game save history created"
    echo ""
    
    demo_pause
    
    log_section "2. BraidCommit (Semantic Attribution)"
    log_info "Purpose: Record semantic attribution braids"
    log_info "Used by: SweetGrass and attribution primals"
    log_info "Contains: Braid ID, subject hash, braid hash"
    log_info ""
    log_info "Example use case:"
    log_info "  • AI model generates content"
    log_info "  • SweetGrass creates attribution braid"
    log_info "  • LoamSpine commits BraidCommit entry"
    log_info "  • Permanent provenance trail established"
    echo ""
    
    demo_pause
    
    log_section "3. SliceAnchor (Waypoint for Borrowed State)"
    log_info "Purpose: Anchor borrowed state at a waypoint"
    log_info "Used by: State lending between primals"
    log_info "Contains: Slice hash, origin spine, loan terms"
    log_info ""
    log_info "Example use case:"
    log_info "  • Primal A lends state to Primal B"
    log_info "  • LoamSpine creates waypoint spine"
    log_info "  • SliceAnchor records the loan"
    log_info "  • State tracking for borrowed data"
    echo ""
    
    demo_pause
    
    log_section "4. SliceReturn (Return Borrowed State)"
    log_info "Purpose: Record return of borrowed state"
    log_info "Used by: Completing state loans"
    log_info "Contains: Original anchor entry, resolution"
    log_info ""
    log_info "Example use case:"
    log_info "  • Primal B completes work on borrowed state"
    log_info "  • Returns state to Primal A"
    log_info "  • SliceReturn closes the loop"
    log_info "  • Complete provenance of state journey"
    echo ""
    
    demo_pause
    
    log_section "5. CertificateMint (Digital Ownership)"
    log_info "Purpose: Mint a new certificate (like NFT)"
    log_info "Used by: Creating digital ownership"
    log_info "Contains: Certificate ID, type, owner, metadata"
    log_info ""
    log_info "Example use case:"
    log_info "  • Game achievement unlocked"
    log_info "  • Certificate minted for achievement"
    log_info "  • Player owns the certificate"
    log_info "  • Can transfer or loan to others"
    echo ""
    
    demo_pause
    
    log_section "6. RollupSummary (Compression)"
    log_info "Purpose: Compress multiple entries into summary"
    log_info "Used by: Spine compression/archival"
    log_info "Contains: Range of entries, summary hash"
    log_info ""
    log_info "Example use case:"
    log_info "  • Spine grows to 10,000 entries"
    log_info "  • First 1,000 rolled up to summary"
    log_info "  • Reduces storage, maintains integrity"
    log_info "  • Full history still verifiable"
    echo ""
    
    demo_pause
    
    # ========================================================================
    # Step 3: Complete Entry Type List
    # ========================================================================
    
    log_header "📋 Complete Entry Type Reference"
    
    cat <<EOF

Entry Types in LoamSpine:

1.  Genesis              - Chain initialization
2.  SessionCommit        - Ephemeral session → permanent
3.  BraidCommit          - Semantic attribution commit
4.  SliceAnchor          - Anchor borrowed state at waypoint
5.  SliceReturn          - Return borrowed state
6.  CertificateMint      - Create new certificate
7.  CertificateTransfer  - Transfer certificate ownership
8.  CertificateLoan      - Loan certificate temporarily
9.  CertificateReturn    - Return loaned certificate
10. RollupSummary        - Compress entry range
11. Metadata             - Key-value metadata
12. Text                 - Plain text entry
13. Binary               - Raw binary data
14. Reference            - Reference to payload
15. Seal                 - Seal spine (immutable)

Plus more specialized types for specific use cases!

EOF
    
    demo_pause
    
    # ========================================================================
    # Step 4: Integration Points
    # ========================================================================
    
    log_header "🔗 Integration Points"
    
    log_info "Entry types enable integration with:"
    echo ""
    
    log_info "Ephemeral Primals (RhizoCrypt):"
    log_info "  └─ SessionCommit entries"
    echo ""
    
    log_info "Attribution Primals (SweetGrass):"
    log_info "  └─ BraidCommit entries"
    echo ""
    
    log_info "State Management:"
    log_info "  └─ SliceAnchor + SliceReturn entries"
    echo ""
    
    log_info "Digital Ownership:"
    log_info "  └─ Certificate* entries"
    echo ""
    
    log_info "Compression/Archival:"
    log_info "  └─ RollupSummary entries"
    echo ""
    
    demo_pause
    
    # ========================================================================
    # Summary
    # ========================================================================
    
    log_header "🎉 Demo Complete!"
    
    log_success "You've learned:"
    log_info "  ✅ All 15+ entry types LoamSpine supports"
    log_info "  ✅ SessionCommit for ephemeral → permanent"
    log_info "  ✅ BraidCommit for semantic attribution"
    log_info "  ✅ SliceAnchor/Return for state lending"
    log_info "  ✅ Certificate operations for digital ownership"
    log_info "  ✅ Integration patterns with other primals"
    echo ""
    
    log_info "Next steps:"
    log_info "  • Try 03-certificate-lifecycle for full certificate flow"
    log_info "  • Check out 04-proofs for inclusion and provenance proofs"
    log_info "  • Explore 04-inter-primal for live primal integration"
    echo ""
    
    # Generate receipt
    generate_receipt "${DEMO_NAME}" "success" \
        "Demonstrated all 15+ entry types" \
        "Explained SessionCommit (ephemeral integration)" \
        "Explained BraidCommit (attribution integration)" \
        "Explained SliceAnchor/Return (state lending)" \
        "Explained Certificate operations" \
        "Showed integration patterns"
    
    log_success "Demo completed successfully! 🦴"
}

# Run main
main "$@"
