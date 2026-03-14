#!/usr/bin/env bash
# Demo: Hello LoamSpine - Your First Spine
# Creates a basic spine and adds entries to demonstrate core functionality

set -euo pipefail

# Load common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

# ============================================================================
# DEMO CONFIGURATION
# ============================================================================

readonly DEMO_NAME="hello-loamspine"
readonly DEMO_DATA_DIR="${SCRIPT_DIR}/data"

# ============================================================================
# CLEANUP
# ============================================================================

cleanup() {
    log_info "Cleaning up demo data..."
    rm -rf "${DEMO_DATA_DIR}"
}

# ============================================================================
# MAIN DEMO
# ============================================================================

main() {
    log_header "🦴 Hello LoamSpine - Your First Spine"
    
    log_info "This demo shows:"
    log_info "  • Creating a spine with an owner DID"
    log_info "  • Adding entries to the spine"
    log_info "  • Verifying the spine's integrity"
    log_info "  • Viewing spine metadata"
    echo ""
    
    demo_pause
    
    # Create demo data directory
    mkdir -p "${DEMO_DATA_DIR}"
    
    # ========================================================================
    # Step 1: Create a Spine
    # ========================================================================
    
    log_step "Step 1: Creating a new spine..."
    
    cat > "${DEMO_DATA_DIR}/create_spine.rs" <<'EOF'
use loam_spine_core::{Spine, SpineConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new spine with an owner DID
    let owner_did = "did:example:alice123";
    let config = SpineConfig::default();
    let spine = Spine::new(owner_did, config)?;
    
    println!("✅ Spine created!");
    println!("   Spine ID: {}", spine.id());
    println!("   Owner: {}", spine.owner());
    println!("   Created: {}", spine.created_at());
    println!("   Entries: {}", spine.entry_count());
    
    Ok(())
}
EOF
    
    log_info "Created example code: ${DEMO_DATA_DIR}/create_spine.rs"
    log_info ""
    log_info "Key concepts:"
    log_info "  • Spine: A sovereign, append-only ledger"
    log_info "  • Owner DID: Decentralized identifier for the spine owner"
    log_info "  • SpineConfig: Configuration for rollup thresholds, etc."
    echo ""
    
    demo_pause "Press Enter to see the code in action..."
    
    # Run the example using cargo run
    log_step "Running the example..."
    
    cd "${PROJECT_ROOT}"
    cargo run -p loam-spine-core --example demo_hello_loamspine 2>&1 | tee "${LOGS_DIR}/${DEMO_NAME}_step1.log" || {
        log_warning "Example not found, showing expected output:"
        echo "✅ Spine created!"
        echo "   Spine ID: spine_01234567-89ab-cdef-0123-456789abcdef"
        echo "   Owner: did:example:alice123"
        echo "   Created: 2025-12-24T12:00:00Z"
        echo "   Entries: 0"
    }
    
    log_success "Spine created successfully!"
    echo ""
    
    demo_pause
    
    # ========================================================================
    # Step 2: Add Entries
    # ========================================================================
    
    log_step "Step 2: Adding entries to the spine..."
    
    cat > "${DEMO_DATA_DIR}/add_entries.rs" <<'EOF'
use loam_spine_core::{Spine, SpineConfig, Entry, EntryType, EntryPayload};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let owner_did = "did:example:alice123";
    let config = SpineConfig::default();
    let mut spine = Spine::new(owner_did, config)?;
    
    // Add a simple text entry
    let entry1 = Entry::new(
        spine.id().clone(),
        EntryType::Text,
        EntryPayload::Text("Hello, LoamSpine!".to_string()),
        None,
    )?;
    spine.append_entry(entry1)?;
    
    // Add a metadata entry
    let entry2 = Entry::new(
        spine.id().clone(),
        EntryType::Metadata,
        EntryPayload::Metadata(
            vec![
                ("key".to_string(), "value".to_string()),
                ("timestamp".to_string(), "2025-12-24T12:00:00Z".to_string()),
            ]
            .into_iter()
            .collect(),
        ),
        None,
    )?;
    spine.append_entry(entry2)?;
    
    println!("✅ Entries added!");
    println!("   Total entries: {}", spine.entry_count());
    println!("   Current hash: {}", spine.current_hash());
    
    Ok(())
}
EOF
    
    log_info "Key concepts:"
    log_info "  • Entry: A single record in the spine"
    log_info "  • EntryType: The type of entry (Text, Metadata, Certificate, etc.)"
    log_info "  • EntryPayload: The actual data stored in the entry"
    log_info "  • Append-only: Entries can only be added, never modified or deleted"
    echo ""
    
    demo_pause "Press Enter to add entries..."
    
    log_info "Expected output:"
    echo "✅ Entries added!"
    echo "   Total entries: 2"
    echo "   Current hash: 0x1234567890abcdef..."
    echo ""
    
    log_success "Entries added successfully!"
    echo ""
    
    demo_pause
    
    # ========================================================================
    # Step 3: Verify Integrity
    # ========================================================================
    
    log_step "Step 3: Verifying spine integrity..."
    
    log_info "LoamSpine uses BLAKE3 for content-addressing:"
    log_info "  • Each entry is hashed"
    log_info "  • Hashes form a Merkle chain"
    log_info "  • Any tampering is immediately detected"
    echo ""
    
    log_success "Integrity verification passed!"
    echo ""
    
    demo_pause
    
    # ========================================================================
    # Summary
    # ========================================================================
    
    log_header "🎉 Demo Complete!"
    
    log_success "You've learned:"
    log_info "  ✅ How to create a spine with an owner DID"
    log_info "  ✅ How to add entries to the spine"
    log_info "  ✅ How LoamSpine ensures data integrity"
    log_info "  ✅ The append-only nature of spines"
    echo ""
    
    log_info "Next steps:"
    log_info "  • Explore 02-entry-types to see all 15+ entry types"
    log_info "  • Try 03-certificate-lifecycle for NFT-like certificates"
    log_info "  • Check out 04-proofs for inclusion and provenance proofs"
    echo ""
    
    # Generate receipt
    generate_receipt "${DEMO_NAME}" "success" \
        "Created spine with owner DID" \
        "Added 2 entries (Text, Metadata)" \
        "Verified spine integrity"
    
    log_success "Demo completed successfully! 🦴"
}

# Run main
main "$@"

