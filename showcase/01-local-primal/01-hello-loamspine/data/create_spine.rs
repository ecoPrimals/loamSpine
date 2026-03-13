// SPDX-License-Identifier: AGPL-3.0-only

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
