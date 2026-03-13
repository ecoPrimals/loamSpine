// SPDX-License-Identifier: AGPL-3.0-only

//! # 🦴 Demo: Hello `LoamSpine`
//!
//! Your first contact with sovereign spines.
//!
//! This demo shows:
//! - Creating a spine with `SpineBuilder`
//! - Using DIDs for ownership
//! - Basic spine metadata
//!
//! ## Run
//! ```bash
//! cargo run -p loam-spine-core --example demo_hello_loamspine
//! ```

// Examples allow patterns for demonstration purposes
use loam_spine_core::{Did, LoamSpineResult, SpineBuilder};

fn main() -> LoamSpineResult<()> {
    println!("🦴 Demo: Hello LoamSpine");
    println!("========================\n");

    // 1. Create an owner DID
    println!("Step 1: Create Owner Identity");
    println!("------------------------------");
    let owner = Did::new("did:key:z6MkDemoOwner123456789");
    println!("✓ Owner DID: {owner}");
    println!();

    // 2. Create a spine using the builder pattern
    println!("Step 2: Create Sovereign Spine");
    println!("-------------------------------");
    let spine = SpineBuilder::new(owner.clone())
        .with_name("My First Spine")
        .build()?;

    println!("✓ Spine created!");
    println!("  ID: {}", spine.id);
    println!("  Owner: {}", spine.owner);
    println!("  Name: {:?}", spine.name);
    println!("  Created: {}", spine.created_at);
    println!("  Entry count: {}", spine.height);
    println!();

    // 3. Create another spine with different metadata
    println!("Step 3: Create Another Spine");
    println!("-----------------------------");
    let spine2 = SpineBuilder::new(owner)
        .with_name("Game Save History")
        .build()?;

    println!("✓ Second spine created!");
    println!("  ID: {}", spine2.id);
    println!("  Name: {:?}", spine2.name);
    println!();

    // 4. Verify spine independence
    println!("Step 4: Verify Spine Independence");
    println!("----------------------------------");
    println!("Spines are independent: {}", spine.id != spine2.id);
    println!("Same owner: {}", spine.owner == spine2.owner);
    println!();

    // 5. Summary
    println!("🎉 Success!");
    println!("===========");
    println!("You've created your first sovereign spines.");
    println!();
    println!("Key concepts:");
    println!("  • DID: Decentralized identifier for ownership");
    println!("  • Spine: Your sovereign, append-only ledger");
    println!("  • SpineBuilder: Fluent API for spine creation");
    println!();
    println!("Next: cargo run -p loam-spine-core --example demo_entry_types");

    Ok(())
}
