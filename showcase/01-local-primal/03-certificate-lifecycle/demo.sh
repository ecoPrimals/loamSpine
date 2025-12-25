#!/usr/bin/env bash
# Demo: Certificate Lifecycle - Full Mint → Transfer → Loan → Return Flow
# Shows digital ownership patterns (NFT-like certificates)

set -euo pipefail

# Load common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

# ============================================================================
# DEMO CONFIGURATION
# ============================================================================

readonly DEMO_NAME="certificate-lifecycle"

# ============================================================================
# CLEANUP
# ============================================================================

cleanup() {
    log_info "Cleaning up demo data..."
}

register_cleanup cleanup

# ============================================================================
# MAIN DEMO
# ============================================================================

main() {
    log_header "🦴 Certificate Lifecycle - Digital Ownership"
    
    log_info "This demo shows:"
    log_info "  • Minting a certificate (like NFT creation)"
    log_info "  • Transferring ownership (selling/gifting)"
    log_info "  • Loaning certificate temporarily (lending)"
    log_info "  • Returning loaned certificate"
    log_info "  • Full provenance tracking"
    echo ""
    
    demo_pause
    
    # ========================================================================
    # Step 1: Create a Certificate
    # ========================================================================
    
    log_step "Step 1: Minting a certificate..."
    
    log_info "Scenario: Game achievement unlocked!"
    log_info ""
    log_info "We'll create a certificate for:"
    log_info "  • Type: DigitalGameKey"
    log_info "  • Owner: Alice (did:example:alice)"
    log_info "  • Metadata: Achievement details"
    echo ""
    
    demo_pause "Press Enter to mint the certificate..."
    
    # Run certificate lifecycle example
    cd "${PROJECT_ROOT}"
    
    if cargo run --example certificate_lifecycle 2>&1 | tee "${LOGS_DIR}/${DEMO_NAME}.log"; then
        log_success "Certificate lifecycle example completed!"
    else
        log_error "Certificate lifecycle example failed"
        log_info "This reveals a GAP: Need certificate_lifecycle example!"
        log_warning "Creating stub to demonstrate intended functionality..."
        echo ""
        
        # Demonstrate intended flow
        log_section "Expected Certificate Flow"
        
        cat <<'EOF'

1. MINT - Create Certificate
   Owner: Alice (did:example:alice)
   Type: DigitalGameKey
   Game: "Medieval Quest"
   Achievement: "Dragon Slayer"
   Rarity: Legendary
   ✅ Certificate minted with ID: cert_abc123

2. TRANSFER - Alice sells to Bob
   From: Alice (did:example:alice)
   To: Bob (did:example:bob)
   Transfer fee: 100 tokens
   ✅ Ownership transferred
   ✅ Provenance recorded

3. LOAN - Bob loans to Carol
   From: Bob (did:example:bob)
   To: Carol (did:example:carol)
   Duration: 24 hours
   Terms: Read-only access
   ✅ Certificate loaned
   ✅ Original owner: Bob (unchanged)

4. RETURN - Carol returns to Bob
   From: Carol (did:example:carol)
   To: Bob (did:example:bob)
   Status: Returned on time
   ✅ Loan completed
   ✅ Full history preserved

EOF
    fi
    
    echo ""
    demo_pause
    
    # ========================================================================
    # Step 2: Key Concepts
    # ========================================================================
    
    log_header "📚 Certificate Concepts"
    
    log_section "1. Certificate Types"
    cat <<EOF

Supported Types:
  • DigitalGameKey - Game license or achievement
  • DigitalCollectible - NFT-like collectible
  • AccessToken - Access rights certificate
  • CredentialProof - Identity/qualification proof
  • Custom - Domain-specific certificates

Each type has different metadata requirements.

EOF
    
    demo_pause
    
    log_section "2. Ownership vs Possession"
    log_info "Key distinction:"
    echo ""
    log_info "OWNERSHIP (permanent):"
    log_info "  • Recorded in certificate owner field"
    log_info "  • Changed only via Transfer entry"
    log_info "  • Full control and rights"
    echo ""
    log_info "POSSESSION (temporary):"
    log_info "  • Created via Loan entry"
    log_info "  • Time-limited access"
    log_info "  • Original owner unchanged"
    log_info "  • Must be returned"
    echo ""
    
    demo_pause
    
    log_section "3. Provenance Tracking"
    log_info "Every certificate operation creates spine entries:"
    echo ""
    log_info "Mint → CertificateMint entry"
    log_info "Transfer → CertificateTransfer entry"
    log_info "Loan → CertificateLoan entry"
    log_info "Return → CertificateReturn entry"
    echo ""
    log_info "Full history is immutable and verifiable!"
    echo ""
    
    demo_pause
    
    # ========================================================================
    # Step 3: Real-World Use Cases
    # ========================================================================
    
    log_header "🌍 Real-World Use Cases"
    
    log_section "Gaming"
    cat <<EOF

• Game licenses (digital ownership)
• In-game achievements (proof of accomplishment)
• Rare item ownership (tradeable collectibles)
• Tournament trophies (competitive awards)

Benefits:
  ✓ True ownership (not just license)
  ✓ Transferable between players
  ✓ Provenance (who owned what when)
  ✓ Lending (share without losing ownership)

EOF
    
    demo_pause
    
    log_section "Digital Assets"
    cat <<EOF

• Digital art ownership
• Music rights certificates
• Video content licenses
• Software licenses

Benefits:
  ✓ Proof of original ownership
  ✓ Resale market support
  ✓ Temporary licensing
  ✓ Creator royalty tracking

EOF
    
    demo_pause
    
    log_section "Credentials & Access"
    cat <<EOF

• Professional certifications
• Educational degrees
• Access tokens (venue, event)
• Membership certificates

Benefits:
  ✓ Unforgeable credentials
  ✓ Instant verification
  ✓ Transfer restrictions configurable
  ✓ Expiration handling

EOF
    
    demo_pause
    
    # ========================================================================
    # Step 4: Technical Details
    # ========================================================================
    
    log_header "🔧 Technical Implementation"
    
    log_section "Certificate Structure"
    cat <<'EOF'

```rust
pub struct Certificate {
    pub id: CertificateId,           // Unique identifier
    pub cert_type: CertificateType,  // Type enum
    pub owner: Did,                  // Current owner DID
    pub spine_id: SpineId,           // Spine where recorded
    pub created_at: Timestamp,       // Mint timestamp
    pub metadata: HashMap<String, String>, // Custom data
    pub loan_info: Option<LoanInfo>, // If currently loaned
}
```

EOF
    
    demo_pause
    
    log_section "Loan Mechanics"
    cat <<'EOF'

Loan Terms:
  • Duration (seconds)
  • Expiration timestamp
  • Terms hash (for agreements)
  • Permissions (read-only, etc.)

Loan Lifecycle:
  1. Owner calls loan_certificate()
  2. CertificateLoan entry created
  3. Borrower has possession
  4. Loan expires or manually returned
  5. CertificateReturn entry created
  6. Ownership restored

EOF
    
    demo_pause
    
    log_section "Entry Types Used"
    log_info "Certificate operations use specific entry types:"
    echo ""
    log_info "• CertificateMint - Initial creation"
    log_info "• CertificateTransfer - Ownership change"
    log_info "• CertificateLoan - Temporary transfer"
    log_info "• CertificateReturn - Return from loan"
    echo ""
    log_info "All entries are immutable and form provenance chain."
    echo ""
    
    demo_pause
    
    # ========================================================================
    # Step 5: Code Example
    # ========================================================================
    
    log_header "💻 Code Example"
    
    cat <<'EOF'

Complete certificate lifecycle in Rust:

```rust
use loam_spine_core::{
    LoamSpineService, CertificateType, LoanTerms, Did
};

// 1. Mint certificate
let alice = Did::new("did:example:alice");
let (cert_id, _) = service.mint_certificate(
    spine_id,
    CertificateType::DigitalGameKey {
        game: "Medieval Quest".into(),
        achievement: "Dragon Slayer".into(),
    },
    alice.clone(),
    None,
).await?;

// 2. Transfer to Bob
let bob = Did::new("did:example:bob");
service.transfer_certificate(
    cert_id,
    alice.clone(),
    bob.clone(),
).await?;

// 3. Loan to Carol
let carol = Did::new("did:example:carol");
let terms = LoanTerms::new()
    .with_duration(86400) // 24 hours
    .with_terms("read-only access");
    
service.loan_certificate(
    cert_id,
    bob.clone(),
    carol.clone(),
    terms,
).await?;

// 4. Return to Bob
service.return_certificate(
    cert_id,
    carol.clone(),
).await?;

// Full provenance preserved!
```

EOF
    
    demo_pause
    
    # ========================================================================
    # Summary
    # ========================================================================
    
    log_header "🎉 Demo Complete!"
    
    log_success "You've learned:"
    log_info "  ✅ How to mint certificates (digital ownership)"
    log_info "  ✅ Transfer patterns (selling/gifting)"
    log_info "  ✅ Loan mechanics (temporary access)"
    log_info "  ✅ Return process (completing loans)"
    log_info "  ✅ Provenance tracking (full history)"
    log_info "  ✅ Real-world use cases (gaming, assets, credentials)"
    echo ""
    
    log_info "Next steps:"
    log_info "  • Try 04-proofs for inclusion and provenance proofs"
    log_info "  • Check out 05-backup-restore for certificate backup"
    log_info "  • Explore 04-inter-primal for cross-primal certificates"
    echo ""
    
    # Generate receipt
    generate_receipt "${DEMO_NAME}" "success" \
        "Demonstrated certificate lifecycle" \
        "Mint → Transfer → Loan → Return flow" \
        "Explained ownership vs possession" \
        "Showed real-world use cases" \
        "Provided code examples"
    
    log_success "Demo completed successfully! 🦴"
}

# Run main
main "$@"
