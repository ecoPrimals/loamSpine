// SPDX-License-Identifier: AGPL-3.0-only

//! # 🦴 Demo: Backup and Restore
//!
//! Export and import spines with verification.
//!
//! This demo shows:
//! - Creating a spine with entries
//! - Exporting to binary backup
//! - Exporting to JSON (human-readable)
//! - Importing and verifying
//!
//! ## Run
//! ```bash
//! cargo run -p loam-spine-core --example demo_backup_restore
//! ```

// Examples allow patterns for demonstration purposes
use loam_spine_core::{
    Did, LoamSpineResult, SpineBuilder, backup::SpineBackup, entry::EntryType, types::SessionId,
};
use std::io::Cursor;

fn main() -> LoamSpineResult<()> {
    println!("🦴 Demo: Backup and Restore");
    println!("===========================\n");

    // 1. Create a spine with some entries
    println!("1. Create Spine with Entries");
    println!("----------------------------");
    let owner = Did::new("did:key:z6MkBackupDemo");
    let mut spine = SpineBuilder::new(owner.clone())
        .with_name("Backup Demo Spine")
        .build()?;

    // Add some entries
    for i in 0..5 {
        let entry = spine.create_entry(EntryType::SessionCommit {
            session_id: SessionId::now_v7(),
            merkle_root: [i; 32],
            vertex_count: u64::from((i + 1) * 10),
            committer: owner.clone(),
        });
        spine.append(entry)?;
    }

    println!("✓ Created spine with {} entries", spine.height);
    println!("  Spine ID: {}", spine.id);
    println!();

    // 2. Create backup
    println!("2. Create Backup");
    println!("----------------");
    let entries: Vec<_> = spine.entries().to_vec();

    let backup = SpineBackup::new(spine.clone(), entries, vec![])
        .with_description("Demo backup - full spine export");

    println!("✓ Backup created");
    println!("  Entries: {}", backup.entries.len());
    println!("  Description: {:?}", backup.description);
    println!();

    // 3. Export to binary
    println!("3. Export to Binary");
    println!("-------------------");
    let mut binary_buffer = Vec::new();
    backup.export(&mut binary_buffer)?;
    println!("✓ Binary export: {} bytes", binary_buffer.len());
    println!();

    // 4. Export to JSON
    println!("4. Export to JSON");
    println!("-----------------");
    let json_string = backup.to_json()?;
    println!("✓ JSON export: {} bytes", json_string.len());
    let preview_len = json_string.len().min(100);
    println!("  Preview: {}...", &json_string[..preview_len]);
    println!();

    // 5. Import from binary
    println!("5. Import from Binary");
    println!("---------------------");
    let mut cursor = Cursor::new(&binary_buffer);
    let restored_backup = SpineBackup::import(&mut cursor)?;
    println!("✓ Imported from binary");
    println!("  Spine ID: {}", restored_backup.spine.id);
    println!("  Entries: {}", restored_backup.entries.len());
    println!();

    // 6. Verify backup
    println!("6. Verify Backup");
    println!("----------------");
    let verification = restored_backup.verify();
    println!("✓ Verification complete");
    println!("  Valid: {}", verification.valid);
    if !verification.errors.is_empty() {
        println!("  Errors: {:?}", verification.errors);
    }
    println!();

    // 7. Compare original and restored
    println!("7. Compare Original vs Restored");
    println!("-------------------------------");
    let original_id = spine.id;
    let restored_id = restored_backup.spine.id;
    println!("  Spine IDs match: {}", original_id == restored_id);
    println!(
        "  Entry counts match: {}",
        spine.height == u64::try_from(restored_backup.entries.len()).unwrap_or(0)
    );
    println!(
        "  Owner matches: {}",
        spine.owner == restored_backup.spine.owner
    );
    println!();

    println!("🎉 Success!");
    println!("===========");
    println!("You've completed a full backup/restore cycle:");
    println!();
    println!("  Create → Populate → Export → Import → Verify");
    println!();
    println!("Key concepts:");
    println!("  • SpineBackup: Container for spine + entries + certs");
    println!("  • Binary export: Compact, fast (bincode)");
    println!("  • JSON export: Human-readable, portable");
    println!("  • Verification: Chain integrity + certificate validity");

    Ok(())
}
