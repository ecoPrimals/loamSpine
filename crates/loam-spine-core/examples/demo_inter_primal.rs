// SPDX-License-Identifier: AGPL-3.0-only

//! # 🦴 Demo: Inter-Primal Integration
//!
//! Shows how `LoamSpine` integrates with other primals via capability discovery.
//!
//! This demo shows:
//! - `CommitAcceptor` trait (ephemeral storage → `LoamSpine`)
//! - `BraidAcceptor` trait (semantic attribution → `LoamSpine`)
//! - Capability-based discovery (no hardcoded primal names)
//!
//! ## Run
//! ```bash
//! cargo run -p loam-spine-core --example demo_inter_primal
//! ```

// Examples allow patterns for demonstration purposes
use loam_spine_core::{
    Did, LoamSpineResult,
    discovery::CapabilityRegistry,
    service::LoamSpineService,
    traits::{BraidAcceptor, BraidSummary, CommitAcceptor, DehydrationSummary, SpineQuery},
    types::SessionId,
};

#[tokio::main]
async fn main() -> LoamSpineResult<()> {
    println!("🦴 Demo: Inter-Primal Integration");
    println!("==================================\n");

    // 1. Create service with capability registry
    println!("1. Create Service with Capability Registry");
    println!("-------------------------------------------");
    let registry = CapabilityRegistry::new();
    let service = LoamSpineService::with_capabilities(registry.clone());
    println!("✓ Service created with capability registry");
    println!("  Primals discover each other at runtime");
    println!("  No hardcoded dependencies!");
    println!();

    // Create a spine for the demo
    let owner = Did::new("did:key:z6MkInterPrimalDemo");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Inter-Primal Demo".into()))
        .await?;
    println!("✓ Spine created: {spine_id}");
    println!();

    // 2. Ephemeral Storage → LoamSpine (CommitAcceptor)
    println!("2. Ephemeral Storage → LoamSpine (CommitAcceptor)");
    println!("--------------------------------------------------");
    println!("Simulating session dehydration from ephemeral primal...");

    let session_id = SessionId::now_v7();
    let summary =
        DehydrationSummary::new(session_id, "game-session", [0xAB; 32]).with_vertex_count(42);

    let commit_ref = service
        .commit_session(spine_id, owner.clone(), summary)
        .await?;

    println!("✓ Session committed to spine!");
    println!("  Session ID: {session_id}");
    println!("  Entry Index: {}", commit_ref.index);
    println!("  Entry Hash: {:?}", commit_ref.entry_hash);
    println!();

    // 3. Semantic Attribution → LoamSpine (BraidAcceptor)
    println!("3. Semantic Attribution → LoamSpine (BraidAcceptor)");
    println!("----------------------------------------------------");
    println!("Simulating braid commit from attribution primal...");

    let braid_id = uuid::Uuid::now_v7();
    let braid = BraidSummary::new(braid_id, "attribution", [0xCD; 32], [0xEF; 32]);

    let entry_hash = service.commit_braid(spine_id, owner.clone(), braid).await?;

    println!("✓ Braid committed to spine!");
    println!("  Braid ID: {braid_id}");
    println!("  Entry Hash: {entry_hash:?}");
    println!();

    // 4. Capability-Based Discovery
    println!("4. Capability-Based Discovery");
    println!("-----------------------------");
    println!("Checking available capabilities...");

    // Check for signer capability
    if let Some(_signer) = registry.get_signer().await {
        println!("  ✓ Signer capability: Available");
    } else {
        println!("  ⊘ Signer capability: Not registered");
        println!("    (A signing primal would register this at runtime)");
    }

    // Check for verifier capability
    if let Some(_verifier) = registry.get_verifier().await {
        println!("  ✓ Verifier capability: Available");
    } else {
        println!("  ⊘ Verifier capability: Not registered");
        println!("    (A verification primal would register this at runtime)");
    }
    println!();

    // 5. Query the spine
    println!("5. Query Committed Entries");
    println!("--------------------------");
    let entries = service.get_entries(spine_id, 0, 10).await?;
    println!("Spine now has {} entries:", entries.len());
    for (i, entry) in entries.iter().enumerate() {
        println!("  {}. {:?}", i, entry.entry_type.domain());
    }
    println!();

    // Summary
    println!("═══════════════════════════════════════════════════════════════");
    println!("  INTER-PRIMAL INTEGRATION SUMMARY");
    println!("═══════════════════════════════════════════════════════════════");
    println!();
    println!("Integration Traits:");
    println!("  • CommitAcceptor: Ephemeral sessions → LoamSpine entries");
    println!("  • BraidAcceptor: Semantic braids → LoamSpine entries");
    println!("  • Signer/Verifier: Crypto via capability registry");
    println!();
    println!("Architecture Principles:");
    println!("  • Runtime discovery (no compile-time coupling)");
    println!("  • Capability-based integration (no hardcoded primal names)");
    println!("  • Graceful degradation when services unavailable");
    println!("  • Zero vendor lock-in");
    println!();

    println!("🎉 Success!");
    println!("===========");
    println!("You've seen inter-primal integration in action.");
    println!();
    println!("When real primals are connected:");
    println!("  • Ephemeral primals dehydrate sessions → LoamSpine commits");
    println!("  • Attribution primals create braids → LoamSpine commits");
    println!("  • Signing primals provide crypto via capability registry");

    Ok(())
}
