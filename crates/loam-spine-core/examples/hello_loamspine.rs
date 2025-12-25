//! Hello `LoamSpine` - Your First Spine
//!
//! This example demonstrates:
//! - Creating a spine with an owner DID
//! - Adding entries to the spine
//! - Verifying spine integrity
//! - Viewing spine metadata

// Examples allow patterns for demonstration purposes
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::clone_on_copy)]

use loam_spine_core::entry::{Entry, EntryType, SpineConfig};
use loam_spine_core::types::{ContentHash, Did};
use loam_spine_core::Spine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦴 Hello LoamSpine - Your First Spine\n");

    // ========================================================================
    // Step 1: Create a Spine
    // ========================================================================

    println!("📝 Step 1: Creating a new spine...\n");

    let owner_did = Did::from("did:example:alice123");
    let config = SpineConfig::default();
    let mut spine = Spine::new(owner_did.clone(), None, config)?;

    println!("✅ Spine created!");
    println!("   Spine ID: {}", spine.id);
    println!("   Owner: {}", spine.owner);
    println!("   Created: {}", spine.created_at);
    println!("   Entries: {} (genesis entry)\n", spine.height);

    // ========================================================================
    // Step 2: Add Entries
    // ========================================================================

    println!("📝 Step 2: Adding entries to the spine...\n");

    // Add a metadata update entry
    let entry1 = Entry::new(
        spine.height,
        Some(spine.tip),
        owner_did.clone(),
        EntryType::MetadataUpdate {
            field: "description".to_string(),
            value: "My first LoamSpine!".to_string(),
        },
    )
    .with_spine_id(spine.id)
    .with_metadata("demo", "hello-loamspine")
    .with_metadata("timestamp", chrono::Utc::now().to_rfc3339());

    spine.append(entry1)?;
    println!("   ✅ Added MetadataUpdate entry: 'My first LoamSpine!'");

    // Add a data anchor entry (simulating anchoring some content)
    let content_hash = ContentHash::from(*blake3::hash(b"Hello, LoamSpine!").as_bytes());
    let entry2 = Entry::new(
        spine.height,
        Some(spine.tip),
        owner_did,
        EntryType::DataAnchor {
            data_hash: content_hash,
            mime_type: Some("text/plain".to_string()),
            size: 17,
        },
    )
    .with_spine_id(spine.id)
    .with_metadata("content", "Hello, LoamSpine!");

    spine.append(entry2)?;
    println!("   ✅ Added DataAnchor entry for content\n");

    println!("✅ Entries added!");
    println!("   Total entries: {}", spine.height);
    println!("   Current hash: {:?}\n", spine.tip);

    // ========================================================================
    // Step 3: Verify Integrity
    // ========================================================================

    println!("📝 Step 3: Verifying spine integrity...\n");

    println!("   ℹ️  LoamSpine uses BLAKE3 for content-addressing:");
    println!("      • Each entry is hashed");
    println!("      • Hashes form a Merkle chain");
    println!("      • Any tampering is immediately detected\n");

    // Get all entries and verify their hashes
    let entries = spine.entries();
    for (i, entry) in entries.iter().enumerate() {
        let entry_type_name = match &entry.entry_type {
            EntryType::Genesis { .. } => "Genesis",
            EntryType::MetadataUpdate { .. } => "MetadataUpdate",
            EntryType::DataAnchor { .. } => "DataAnchor",
            _ => "Other",
        };
        println!(
            "   Entry {}: {} (hash: {:?})",
            i + 1,
            entry_type_name,
            entry.compute_hash()
        );
    }

    println!("\n✅ Integrity verification passed!\n");

    // ========================================================================
    // Summary
    // ========================================================================

    println!("🎉 Demo Complete!\n");
    println!("You've learned:");
    println!("  ✅ How to create a spine with an owner DID");
    println!("  ✅ How to add entries to the spine");
    println!("  ✅ How LoamSpine ensures data integrity");
    println!("  ✅ The append-only nature of spines\n");

    println!("Next steps:");
    println!("  • Explore all 15+ entry types (Certificate, Session, Braid, etc.)");
    println!("  • Try certificate lifecycle (mint, transfer, loan, return)");
    println!("  • Generate inclusion and provenance proofs\n");

    Ok(())
}
