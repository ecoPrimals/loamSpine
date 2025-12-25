#!/usr/bin/env bash
# Demo: Inter-Primal Integration - Full Ecosystem Coordination
# Uses demo_inter_primal.rs to show capability-based integration

set -euo pipefail

# Load common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

# ============================================================================
# DEMO CONFIGURATION
# ============================================================================

readonly DEMO_NAME="inter-primal-integration"

# ============================================================================
# CLEANUP
# ============================================================================

cleanup() {
    log_info "Cleaning up..."
}

register_cleanup cleanup

# ============================================================================
# MAIN DEMO
# ============================================================================

main() {
    log_header "🦴 Inter-Primal Integration - Full Ecosystem"
    
    log_info "This demo shows:"
    log_info "  • CommitAcceptor (ephemeral → LoamSpine)"
    log_info "  • BraidAcceptor (attribution → LoamSpine)"
    log_info "  • Capability-based discovery (no hardcoding!)"
    log_info "  • Runtime primal coordination"
    log_info "  • **REAL PATTERN - NO MOCKS!**"
    echo ""
    
    demo_pause
    
    # ========================================================================
    # Step 1: Architecture Overview
    # ========================================================================
    
    log_step "Step 1: Architecture Overview..."
    
    log_section "Inter-Primal Coordination Pattern"
    cat <<'EOF'

┌─────────────────┐         ┌──────────────┐         ┌─────────────────┐
│  RhizoCrypt     │         │   Songbird   │         │   SweetGrass    │
│  (Ephemeral)    │────────▶│  (Discovery) │◀────────│  (Attribution)  │
└─────────────────┘         └──────────────┘         └─────────────────┘
        │                           │                          │
        │ SessionCommit             │ Discovery                │ BraidCommit
        ▼                           ▼                          ▼
┌───────────────────────────────────────────────────────────────────────┐
│                          LoamSpine                                     │
│                     (Permanent Ledger)                                 │
│                                                                         │
│  • CommitAcceptor trait   ← receives ephemeral commits                 │
│  • BraidAcceptor trait    ← receives attribution commits               │
│  • Capability Registry    ← discovers primals at runtime               │
│  • Zero hardcoding        ← no primal names in code!                   │
└───────────────────────────────────────────────────────────────────────┘

Key Principles:
  1. Traits define integration contracts
  2. Runtime discovery (not compile-time)
  3. Each primal sovereign and independent
  4. Coordination through capabilities

EOF
    
    demo_pause
    
    # ========================================================================
    # Step 2: Run Inter-Primal Demo
    # ========================================================================
    
    log_step "Step 2: Running inter-primal integration..."
    
    cd "${PROJECT_ROOT}"
    
    log_info "This demonstrates:"
    log_info "  1. Capability registry setup"
    log_info "  2. Session commit (ephemeral → permanent)"
    log_info "  3. Braid commit (attribution → permanent)"
    log_info "  4. Capability discovery patterns"
    echo ""
    
    demo_pause "Press Enter to run integration demo..."
    
    if cargo run --example demo_inter_primal 2>&1 | tee "${LOGS_DIR}/${DEMO_NAME}.log"; then
        log_success "Inter-primal integration demo completed!"
    else
        log_error "Demo failed"
        exit 1
    fi
    
    echo ""
    demo_pause
    
    # ========================================================================
    # Step 3: Integration Patterns Explained
    # ========================================================================
    
    log_header "🔗 Integration Patterns"
    
    log_section "1. CommitAcceptor (Ephemeral → Permanent)"
    cat <<'EOF'

Purpose: Accept commits from ephemeral storage primals

Trait:
```rust
#[async_trait]
pub trait CommitAcceptor {
    async fn commit_session(
        &self,
        spine_id: SpineId,
        committer: Did,
        summary: DehydrationSummary,
    ) -> LoamSpineResult<CommitRef>;
}
```

Flow:
1. RhizoCrypt (ephemeral) finishes session
2. Dehydrates DAG to Merkle root
3. Calls LoamSpine.commit_session()
4. LoamSpine creates SessionCommit entry
5. Returns commit reference

No hardcoding - RhizoCrypt discovers LoamSpine via Songbird!

EOF
    
    demo_pause
    
    log_section "2. BraidAcceptor (Attribution → Permanent)"
    cat <<'EOF'

Purpose: Accept semantic attribution commits

Trait:
```rust
#[async_trait]
pub trait BraidAcceptor {
    async fn commit_braid(
        &self,
        spine_id: SpineId,
        committer: Did,
        summary: BraidSummary,
    ) -> LoamSpineResult<EntryHash>;
}
```

Flow:
1. SweetGrass creates attribution braid
2. Computes braid hash
3. Calls LoamSpine.commit_braid()
4. LoamSpine creates BraidCommit entry
5. Returns entry hash

No hardcoding - SweetGrass discovers LoamSpine via Songbird!

EOF
    
    demo_pause
    
    log_section "3. Capability Registry (Runtime Discovery)"
    cat <<'EOF'

Purpose: Discover primals at runtime (zero hardcoding)

Pattern:
```rust
// LoamSpine creates registry
let registry = CapabilityRegistry::new();
let service = LoamSpineService::with_capabilities(registry.clone());

// Signing primal registers (if available)
registry.register_signer(signer).await;

// LoamSpine uses signing when needed
if let Some(signer) = registry.get_signer().await {
    let signature = signer.sign(data).await?;
}
```

Benefits:
  • No compile-time coupling
  • Primals can come and go
  • Graceful degradation
  • True sovereignty

EOF
    
    demo_pause
    
    # ========================================================================
    # Step 4: Real-World Integration
    # ========================================================================
    
    log_header "🌍 Real-World Integration"
    
    log_section "With BearDog (Signing)"
    log_info "LoamSpine can integrate with ../bins/beardog for:"
    echo ""
    log_info "• Entry signing (cryptographic integrity)"
    log_info "• DID verification (identity validation)"
    log_info "• HSM integration (hardware security)"
    echo ""
    log_info "Pattern:"
    log_info "  1. BearDog registers as 'signing' capability"
    log_info "  2. LoamSpine discovers via Songbird"
    log_info "  3. LoamSpine calls BearDog for signatures"
    log_info "  4. Zero hardcoding between them!"
    echo ""
    
    demo_pause
    
    log_section "With NestGate (Storage)"
    log_info "LoamSpine can integrate with ../bins/nestgate for:"
    echo ""
    log_info "• Large payload storage (content-addressed)"
    log_info "• ZFS backends (enterprise storage)"
    log_info "• Backup/restore (data durability)"
    echo ""
    log_info "Pattern:"
    log_info "  1. NestGate registers as 'storage' capability"
    log_info "  2. LoamSpine discovers via Songbird"
    log_info "  3. LoamSpine stores payloads in NestGate"
    log_info "  4. Spine entries reference NestGate URLs"
    echo ""
    
    demo_pause
    
    log_section "Complete Ecosystem"
    cat <<'EOF'

Full integration stack:

Songbird (Discovery)
    ↓
LoamSpine (Permanence) ←→ BearDog (Signing)
    ↓                      ↑
    ├──────────────────────┤
    ↓                      ↓
NestGate (Storage)    ToadStool (Compute)
    ↓                      ↓
    └──────────────────────┘
              ↓
         Squirrel (AI)

All connected via:
  • Capability-based discovery
  • Runtime coordination
  • Zero hardcoding
  • Trait-based contracts

EOF
    
    demo_pause
    
    # ========================================================================
    # Step 5: Gaps Discovered
    # ========================================================================
    
    log_header "🔍 Gaps Discovered"
    
    log_section "Gap #4: Service Lifecycle Coordination"
    cat <<'EOF'

What we learned running this demo:
  • Need standardized service startup order
  • Need health check coordination
  • Need graceful failure handling
  • Need service dependency management

Questions that emerged:
  1. How does LoamSpine know Songbird is ready?
  2. What if BearDog starts after LoamSpine?
  3. How to handle service restarts?
  4. What's the retry strategy?

These are GOOD questions - real integration reveals real needs!

Actions needed:
  1. Define service startup protocol
  2. Implement health check polling
  3. Add reconnection logic
  4. Document dependency management

EOF
    
    echo ""
    demo_pause
    
    # ========================================================================
    # Summary
    # ========================================================================
    
    log_header "🎉 Demo Complete!"
    
    log_success "You've learned:"
    log_info "  ✅ Inter-primal integration patterns"
    log_info "  ✅ CommitAcceptor (ephemeral → permanent)"
    log_info "  ✅ BraidAcceptor (attribution → permanent)"
    log_info "  ✅ Capability-based discovery"
    log_info "  ✅ Zero hardcoding architecture"
    log_info "  ✅ Real ecosystem coordination"
    echo ""
    
    log_warning "Gaps discovered:"
    log_info "  • Gap #4: Service lifecycle coordination"
    log_info "  • Need startup protocol definition"
    log_info "  • Need health check patterns"
    log_info "  • Need dependency management"
    echo ""
    
    log_info "These gaps are VALUABLE - they show what to build next!"
    echo ""
    
    log_info "Next steps:"
    log_info "  • Test with real BearDog binary (../bins/beardog)"
    log_info "  • Test with real NestGate binary (../bins/nestgate)"
    log_info "  • Document service coordination patterns"
    log_info "  • Implement lifecycle manager enhancements"
    echo ""
    
    # Generate receipt
    generate_receipt "${DEMO_NAME}" "success" \
        "Demonstrated inter-primal integration" \
        "CommitAcceptor (ephemeral → permanent)" \
        "BraidAcceptor (attribution → permanent)" \
        "Capability-based discovery patterns" \
        "**DISCOVERED Gap #4: Service lifecycle coordination**"
    
    log_success "Demo completed - real integration patterns shown! 🦴"
}

# Run main
main "$@"

