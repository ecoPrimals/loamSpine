// SPDX-License-Identifier: AGPL-3.0-only

//! # 🦴 Demo: Entry Types
//!
//! Explore entry type variants.
//!
//! This demo shows:
//! - Creating entries with different types
//! - Entry chaining and validation
//! - Entry hashing with Blake3
//!
//! ## Run
//! ```bash
//! cargo run -p loam-spine-core --example demo_entry_types
//! ```

// Examples allow patterns for demonstration purposes
use loam_spine_core::{Did, LoamSpineResult, SpineBuilder, entry::EntryType, types::SessionId};

fn main() -> LoamSpineResult<()> {
    println!("🦴 Demo: Entry Types");
    println!("====================\n");

    let owner = Did::new("did:key:z6MkDemoOwner");
    let mut spine = SpineBuilder::new(owner.clone())
        .with_name("Entry Types Demo")
        .build()?;

    println!("Created spine: {}\n", spine.id);

    // 1. SessionCommit - from ephemeral storage primal
    println!("1. SessionCommit (from ephemeral primal)");
    println!("-----------------------------------------");
    let session_id = SessionId::now_v7();
    let entry1 = spine.create_entry(EntryType::SessionCommit {
        session_id,
        merkle_root: [0xAB; 32],
        vertex_count: 42,
        committer: owner.clone(),
    });
    let hash1 = spine.append(entry1)?;
    println!("✓ SessionCommit added: {hash1:?}");
    println!("  Session: {session_id}");
    println!("  Vertices: 42");
    println!();

    // 2. BraidCommit - from attribution primal
    println!("2. BraidCommit (from attribution primal)");
    println!("-----------------------------------------");
    let entry2 = spine.create_entry(EntryType::BraidCommit {
        braid_id: uuid::Uuid::now_v7(),
        subject_hash: [0xCD; 32],
        braid_hash: [0xEF; 32],
    });
    let hash2 = spine.append(entry2)?;
    println!("✓ BraidCommit added: {hash2:?}");
    println!();

    // 3. DataAnchor - external data reference
    println!("3. DataAnchor (external data)");
    println!("-----------------------------");
    let entry3 = spine.create_entry(EntryType::DataAnchor {
        data_hash: [0x11; 32],
        mime_type: Some("application/octet-stream".to_string()),
        size: 1024 * 1024, // 1 MB
    });
    let hash3 = spine.append(entry3)?;
    println!("✓ DataAnchor added: {hash3:?}");
    println!("  Size: 1 MB");
    println!();

    // 4. CertificateMint - ownership creation
    println!("4. CertificateMint (ownership)");
    println!("------------------------------");
    let entry4 = spine.create_entry(EntryType::CertificateMint {
        cert_id: uuid::Uuid::now_v7(),
        cert_type: "digital_game".to_string(),
        initial_owner: owner,
    });
    let hash4 = spine.append(entry4)?;
    println!("✓ CertificateMint added: {hash4:?}");
    println!();

    // 5. MetadataUpdate - spine metadata
    println!("5. MetadataUpdate (spine metadata)");
    println!("----------------------------------");
    let entry5 = spine.create_entry(EntryType::MetadataUpdate {
        field: "description".to_string(),
        value: "Updated spine description".to_string(),
    });
    let hash5 = spine.append(entry5)?;
    println!("✓ MetadataUpdate added: {hash5:?}");
    println!();

    // 6. Custom - extensible entry type
    println!("6. Custom (extensible type)");
    println!("---------------------------");
    let entry6 = spine.create_entry(EntryType::Custom {
        type_uri: "loamspine://annotation/v1".to_string(),
        payload: loam_spine_core::types::ByteBuffer::from_static(b"This was an important session"),
    });
    let hash6 = spine.append(entry6)?;
    println!("✓ Custom added: {hash6:?}");
    println!();

    // Summary
    println!("📊 Spine Summary");
    println!("================");
    println!("Total entries: {}", spine.height);
    println!("Tip: {:?}", spine.tip);
    println!();

    // Verify chain integrity
    println!("🔗 Chain Verification");
    println!("=====================");
    let verification = spine.verify();
    println!("Valid: {}", verification.valid);
    println!("Entries verified: {}", verification.entries_verified);
    if !verification.errors.is_empty() {
        println!("Errors: {:?}", verification.errors);
    }
    println!();

    println!("🎉 Success!");
    println!("===========");
    println!("You've explored 6 entry types.");
    println!();
    println!("Other entry types include:");
    println!("  • Genesis (auto-created)");
    println!("  • SpineSealed");
    println!("  • SliceCheckout/SliceReturn");
    println!("  • CertificateTransfer/Loan/Return");
    println!();
    println!("Next: cargo run -p loam-spine-core --example demo_certificate_lifecycle");

    Ok(())
}
