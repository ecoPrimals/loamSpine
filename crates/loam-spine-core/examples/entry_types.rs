//! Entry Types Demo - Exploring all 15+ `LoamSpine` entry types
//!
//! This example demonstrates:
//! - All entry type variants
//! - When to use each type
//! - Entry metadata and payloads

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
use uuid::Uuid;

// Allow long function for comprehensive demonstration example
#[allow(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦴 LoamSpine Entry Types - Complete Reference\n");

    let owner_did = Did::from("did:example:alice123");
    let config = SpineConfig::default();
    let mut spine = Spine::new(owner_did.clone(), Some("Demo Spine".to_string()), config)?;

    println!("✅ Spine created: {}\n", spine.id);

    // ========================================================================
    // SPINE LIFECYCLE ENTRIES
    // ========================================================================

    println!("📝 Category 1: Spine Lifecycle\n");

    // 1. Genesis (automatically created)
    println!("   1. Genesis - First entry in every spine");
    println!("      Created automatically on Spine::new()");
    println!("      Current entries: {}\n", spine.height);

    // 2. MetadataUpdate
    let metadata_entry = Entry::new(
        spine.height,
        Some(spine.tip),
        owner_did.clone(),
        EntryType::MetadataUpdate {
            field: "status".to_string(),
            value: "active".to_string(),
        },
    )
    .with_spine_id(spine.id);
    spine.append(metadata_entry)?;
    println!("   2. MetadataUpdate - Update spine metadata");
    println!("      Example: Set status to 'active'\n");

    // 3. SpineSealed
    // (Would seal the spine, so we skip adding it)
    println!("   3. SpineSealed - Mark spine as sealed (no more entries)");
    println!("      Example: Spine closed due to completion\n");

    // ========================================================================
    // EPHEMERAL STORAGE INTEGRATION
    // ========================================================================

    println!("📝 Category 2: Ephemeral Storage Integration\n");

    // 4. SessionCommit
    let session_id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
    let session_commit = Entry::new(
        spine.height,
        Some(spine.tip),
        owner_did.clone(),
        EntryType::SessionCommit {
            session_id,
            merkle_root: ContentHash::from(*blake3::hash(b"session_data").as_bytes()),
            vertex_count: 42,
            committer: owner_did.clone(),
        },
    )
    .with_spine_id(spine.id)
    .with_metadata("session_name", "Game Save #1");
    spine.append(session_commit)?;
    println!("   4. SessionCommit - Dehydrated ephemeral session");
    println!("      Example: Game save with 42 vertices\n");

    // 5. SliceCheckout
    let slice_id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
    let slice_checkout = Entry::new(
        spine.height,
        Some(spine.tip),
        owner_did.clone(),
        EntryType::SliceCheckout {
            slice_id,
            source_entry: spine.tip,
            session_id,
            holder: Did::from("did:example:bob456"),
        },
    )
    .with_spine_id(spine.id);
    spine.append(slice_checkout)?;
    println!("   5. SliceCheckout - Slice checked out to ephemeral");
    println!("      Example: Data borrowed by another session\n");

    // 6. SliceReturn
    let slice_return = Entry::new(
        spine.height,
        Some(spine.tip),
        owner_did.clone(),
        EntryType::SliceReturn {
            slice_id,
            checkout_entry: spine.tip,
            success: true,
            summary: Some(ContentHash::from(*blake3::hash(b"updated").as_bytes())),
        },
    )
    .with_spine_id(spine.id);
    spine.append(slice_return)?;
    println!("   6. SliceReturn - Slice returned from ephemeral");
    println!("      Example: Data returned after processing\n");

    // ========================================================================
    // DATA ANCHORING
    // ========================================================================

    println!("📝 Category 3: Data Anchoring\n");

    // 7. DataAnchor
    let data_anchor = Entry::new(
        spine.height,
        Some(spine.tip),
        owner_did.clone(),
        EntryType::DataAnchor {
            data_hash: ContentHash::from(*blake3::hash(b"Important Document").as_bytes()),
            mime_type: Some("application/pdf".to_string()),
            size: 1_024_768,
        },
    )
    .with_spine_id(spine.id)
    .with_metadata("filename", "contract.pdf");
    spine.append(data_anchor)?;
    println!("   7. DataAnchor - Content-addressed data");
    println!("      Example: PDF document (1MB)\n");

    // 8. BraidCommit
    let braid_id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
    let braid_commit = Entry::new(
        spine.height,
        Some(spine.tip),
        owner_did.clone(),
        EntryType::BraidCommit {
            braid_id,
            braid_hash: ContentHash::from(*blake3::hash(b"braid_content").as_bytes()),
            subject_hash: ContentHash::from(*blake3::hash(b"subject").as_bytes()),
        },
    )
    .with_spine_id(spine.id)
    .with_metadata("attribution", "research_paper");
    spine.append(braid_commit)?;
    println!("   8. BraidCommit - Semantic attribution braid");
    println!("      Example: Research paper attribution\n");

    // ========================================================================
    // CERTIFICATE OPERATIONS
    // ========================================================================

    println!("📝 Category 4: Certificate Operations (NFT-like)\n");

    let cert_id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));

    // 9. CertificateMint
    let cert_mint = Entry::new(
        spine.height,
        Some(spine.tip),
        owner_did.clone(),
        EntryType::CertificateMint {
            cert_id: cert_id.clone(),
            cert_type: "Achievement".to_string(),
            initial_owner: owner_did.clone(),
        },
    )
    .with_spine_id(spine.id)
    .with_metadata("achievement", "First Quest Complete");
    spine.append(cert_mint)?;
    println!("   9. CertificateMint - Create new certificate");
    println!("      Example: Achievement certificate\n");

    // 10. CertificateTransfer
    let bob = Did::from("did:example:bob456");
    let cert_transfer = Entry::new(
        spine.height,
        Some(spine.tip),
        owner_did.clone(),
        EntryType::CertificateTransfer {
            cert_id: cert_id.clone(),
            from: owner_did.clone(),
            to: bob.clone(),
        },
    )
    .with_spine_id(spine.id);
    spine.append(cert_transfer)?;
    println!("   10. CertificateTransfer - Transfer ownership");
    println!("       Example: Alice → Bob\n");

    // 11. CertificateLoan
    let cert_loan = Entry::new(
        spine.height,
        Some(spine.tip),
        bob.clone(),
        EntryType::CertificateLoan {
            cert_id: cert_id.clone(),
            lender: bob.clone(),
            borrower: owner_did.clone(),
            duration_secs: Some(3600), // 1 hour
            auto_return: true,
        },
    )
    .with_spine_id(spine.id);
    spine.append(cert_loan)?;
    println!("   11. CertificateLoan - Temporary transfer");
    println!("       Example: Bob loans to Alice for 1 hour\n");

    // 12. CertificateReturn
    let cert_return = Entry::new(
        spine.height,
        Some(spine.tip),
        owner_did.clone(),
        EntryType::CertificateReturn {
            cert_id,
            loan_entry: spine.tip,
        },
    )
    .with_spine_id(spine.id);
    spine.append(cert_return)?;
    println!("   12. CertificateReturn - Return loaned certificate");
    println!("       Example: Alice returns to Bob\n");

    // ========================================================================
    // SLICE OPERATIONS (Waypoints)
    // ========================================================================

    println!("📝 Category 5: Slice Operations (Waypoints)\n");

    let waypoint_slice_id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));

    // 13. SliceAnchor
    let slice_anchor = Entry::new(
        spine.height,
        Some(spine.tip),
        owner_did.clone(),
        EntryType::SliceAnchor {
            slice_id: waypoint_slice_id,
            origin_spine: spine.id,
            origin_entry: spine.genesis,
        },
    )
    .with_spine_id(spine.id);
    spine.append(slice_anchor)?;
    println!("   13. SliceAnchor - Slice anchored at waypoint");
    println!("       Example: Data snapshot at this spine\n");

    // 14. SliceOperation
    let slice_op = Entry::new(
        spine.height,
        Some(spine.tip),
        owner_did.clone(),
        EntryType::SliceOperation {
            slice_id: waypoint_slice_id,
            operation: "transform".to_string(),
        },
    )
    .with_spine_id(spine.id);
    spine.append(slice_op)?;
    println!("   14. SliceOperation - Operation on slice");
    println!("       Example: Data transformation\n");

    // 15. SliceDeparture
    let slice_departure = Entry::new(
        spine.height,
        Some(spine.tip),
        owner_did.clone(),
        EntryType::SliceDeparture {
            slice_id: waypoint_slice_id,
            reason: "Moving to next waypoint".to_string(),
        },
    )
    .with_spine_id(spine.id);
    spine.append(slice_departure)?;
    println!("   15. SliceDeparture - Slice leaves waypoint");
    println!("       Example: Moving to next location\n");

    // ========================================================================
    // SUMMARY
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════\n");
    println!("🎉 Demo Complete!\n");
    println!("Summary:");
    println!("  Total entries: {}", spine.height);
    println!("  Entry types demonstrated: 15");
    println!("  Categories:");
    println!("    • Spine Lifecycle: 3 types");
    println!("    • Ephemeral Integration: 3 types");
    println!("    • Data Anchoring: 2 types");
    println!("    • Certificates: 4 types");
    println!("    • Slice Operations: 3 types\n");

    println!("Key Insights:");
    println!("  ✅ Each entry type serves a specific purpose");
    println!("  ✅ Entries are immutable once added");
    println!("  ✅ Metadata adds context without changing structure");
    println!("  ✅ All entries are content-addressed (BLAKE3)\n");

    Ok(())
}
