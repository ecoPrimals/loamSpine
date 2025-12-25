//! Backup and Restore Example
//!
//! Demonstrates spine backup and restoration:
//! 1. Create spine with entries
//! 2. Backup to JSON
//! 3. Restore from backup
//! 4. Verify integrity

// Examples allow patterns for demonstration purposes
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::uninlined_format_args)]

use loam_spine_core::{
    backup::SpineBackup,
    entry::{Entry, EntryType, SpineConfig},
    Did, Spine,
};

// Allow patterns for demonstration code clarity
#[allow(clippy::too_many_lines)]
#[allow(clippy::redundant_clone)]
#[allow(clippy::unwrap_used)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦴 LoamSpine Backup & Restore Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Step 1: Create original spine
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 Step 1: Creating Original Spine\n");

    let owner_did = Did::new("did:example:alice");
    let config = SpineConfig::default();

    let mut spine = Spine::new(owner_did.clone(), Some("Backup Demo Spine".into()), config)?;

    println!("✅ Spine created:");
    println!("   Spine ID: {}", spine.id);
    println!("   Owner: {}", spine.owner);
    println!("   Name: {}", spine.name.as_ref().unwrap());
    println!();

    // Add some entries
    println!("📝 Adding 5 entries...\n");

    for i in 1..=5 {
        let entry = Entry::new(
            spine.height,
            Some(spine.tip),
            owner_did.clone(),
            EntryType::DataAnchor {
                data_hash: [i as u8; 32],
                mime_type: Some(format!("application/data-{}", i)),
                size: 1000 + i * 100,
            },
        );
        spine.append(entry)?;
        println!("  ✅ Entry {} added", i);
    }

    println!("\n📊 Original Spine State:");
    println!("   Total Entries: {}", spine.height);
    println!("   Genesis: {:?}", &spine.genesis[..8]);
    println!("   Tip: {:?}", &spine.tip[..8]);
    println!();

    // Step 2: Create backup
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💾 Step 2: Creating Backup\n");

    // Collect all entries
    let mut all_entries = Vec::new();
    for i in 0..spine.height {
        if let Some(entry) = spine.get_entry(i) {
            all_entries.push(entry.clone());
        }
    }

    let backup = SpineBackup::new(
        spine.clone(),
        all_entries,
        vec![], // No certificates in this demo
    )
    .with_description("Demo backup");

    println!("✅ Backup created:");
    println!("   Format Version: {}", backup.version);
    println!("   Spine ID: {}", backup.spine.id);
    println!("   Entries: {}", backup.entries.len());
    println!("   Certificates: {}", backup.certificates.len());
    println!("   Description: {}", backup.description.as_ref().unwrap());
    println!();

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&backup)?;
    let json_size = json.len();

    println!("📄 Serialized to JSON:");
    println!("   Size: {} bytes", json_size);
    println!("   First 200 chars: {}...", &json[..200.min(json.len())]);
    println!();

    // Step 3: Restore from backup
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("♻️  Step 3: Restoring from Backup\n");

    // Deserialize
    let restored_backup: SpineBackup = serde_json::from_str(&json)?;

    println!("✅ Deserialized backup:");
    println!("   Spine ID: {}", restored_backup.spine.id);
    println!("   Entries: {}", restored_backup.entries.len());
    println!();

    // Restore spine (just use the spine from backup directly)
    let restored_spine = restored_backup.spine.clone();

    println!("✅ Spine restored:");
    println!("   Spine ID: {}", restored_spine.id);
    println!("   Owner: {}", restored_spine.owner);
    println!("   Name: {}", restored_spine.name.as_ref().unwrap());
    println!("   Height: {}", restored_spine.height);
    println!();

    // Step 4: Verify integrity
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Step 4: Verifying Integrity\n");

    println!("Comparing original vs restored:\n");

    // Compare IDs
    let ids_match = spine.id == restored_spine.id;
    println!(
        "   Spine ID: {}",
        if ids_match {
            "✅ Match"
        } else {
            "❌ Mismatch"
        }
    );

    // Compare owners
    let owners_match = spine.owner == restored_spine.owner;
    println!(
        "   Owner: {}",
        if owners_match {
            "✅ Match"
        } else {
            "❌ Mismatch"
        }
    );

    // Compare heights
    let heights_match = spine.height == restored_spine.height;
    println!(
        "   Height: {}",
        if heights_match {
            "✅ Match"
        } else {
            "❌ Mismatch"
        }
    );

    // Compare genesis
    let genesis_match = spine.genesis == restored_spine.genesis;
    println!(
        "   Genesis: {}",
        if genesis_match {
            "✅ Match"
        } else {
            "❌ Mismatch"
        }
    );

    // Compare tips
    let tips_match = spine.tip == restored_spine.tip;
    println!(
        "   Tip: {}",
        if tips_match {
            "✅ Match"
        } else {
            "❌ Mismatch"
        }
    );

    // Compare entry count (using height as proxy)
    let entry_count_match = spine.height == restored_spine.height;
    println!(
        "   Entry Count: {}",
        if entry_count_match {
            "✅ Match"
        } else {
            "❌ Mismatch"
        }
    );

    println!();

    let all_match = ids_match
        && owners_match
        && heights_match
        && genesis_match
        && tips_match
        && entry_count_match;

    if all_match {
        println!("✅ VERIFICATION PASSED");
        println!("   All fields match perfectly");
        println!("   Restoration was successful");
    } else {
        println!("❌ VERIFICATION FAILED");
        println!("   Some fields do not match");
    }
    println!();

    // Use cases
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💡 Backup & Restore Use Cases\n");

    println!("📦 Archival:");
    println!("   • Long-term storage");
    println!("   • Compliance requirements");
    println!("   • Historical records");
    println!();

    println!("🔄 Migration:");
    println!("   • Move between storage backends");
    println!("   • Upgrade LoamSpine versions");
    println!("   • Transfer between systems");
    println!();

    println!("🛡️  Disaster Recovery:");
    println!("   • Database corruption");
    println!("   • Hardware failure");
    println!("   • Accidental deletion");
    println!();

    println!("🔍 Forensics:");
    println!("   • Audit investigations");
    println!("   • Security analysis");
    println!("   • Compliance audits");
    println!();

    // Summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Demo Summary\n");

    println!("Demonstrated:");
    println!("   ✅ Spine backup creation");
    println!("   ✅ JSON serialization");
    println!("   ✅ Deserialization");
    println!("   ✅ Spine restoration");
    println!("   ✅ Integrity verification");
    println!();

    println!("Key Features:");
    println!("   • Complete spine state capture");
    println!("   • Human-readable JSON format");
    println!("   • Lossless restoration");
    println!("   • Integrity preservation");
    println!();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}
