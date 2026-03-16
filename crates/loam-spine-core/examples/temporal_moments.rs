// SPDX-License-Identifier: AGPL-3.0-or-later

//! Example: Creating and committing temporal moments to `LoamSpine`.
//!
//! This demonstrates how to use the temporal module to track universal time
//! across any domain: code commits, art creation, life events, etc.

#![allow(clippy::unwrap_used)] // Examples use unwrap for clarity
#![allow(clippy::too_many_lines)] // Examples are educational and comprehensive
#![allow(clippy::uninlined_format_args)] // Older style for clarity

use loam_spine_core::{
    SpineBuilder,
    entry::EntryType,
    temporal::{Anchor, AtomicAnchor, Moment, MomentContext, TimePrecision},
    types::{ContentHash, Did, Signature},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦴 LoamSpine Temporal Moments Example\n");

    // Create a personal spine for temporal tracking
    let owner = Did::new("did:key:z6MkAlice");
    let mut spine = SpineBuilder::new(owner.clone())
        .with_name("Alice's Timeline")
        .personal()
        .build()?;

    println!("✅ Created spine: {}", spine.name.as_ref().unwrap());
    println!("   ID: {}\n", spine.id);

    // === Example 1: Code Commit Moment ===
    println!("📝 Example 1: Code Commit");

    let code_moment = Moment {
        id: ContentHash::default(), // Would be computed from content
        timestamp: std::time::SystemTime::now(),
        agent: owner.to_string(),
        state_hash: ContentHash::default(),
        signature: Signature::empty(),
        context: MomentContext::CodeChange {
            message: "feat: add temporal moment support to LoamSpine".to_string(),
            tree_hash: ContentHash::default(),
        },
        parents: vec![],
        anchor: Some(Anchor::Atomic(AtomicAnchor {
            timestamp: std::time::SystemTime::now(),
            precision: TimePrecision::Millisecond,
            source: "system-clock".to_string(),
        })),
        ephemeral_provenance: None,
    };

    let entry = spine.create_entry(EntryType::TemporalMoment {
        moment_id: code_moment.id,
        moment: Box::new(code_moment.clone()),
    });
    let hash = spine.append(entry)?;

    println!("   Committed code change moment");
    println!("   Category: {}", code_moment.context.category());
    println!("   Entry hash: {:?}\n", hash);

    // === Example 2: Art Creation Moment ===
    println!("🎨 Example 2: Art Creation");

    let art_moment = Moment {
        id: ContentHash::default(),
        timestamp: std::time::SystemTime::now(),
        agent: owner.to_string(),
        state_hash: ContentHash::default(),
        signature: Signature::empty(),
        context: MomentContext::ArtCreation {
            title: "Starry Night in Digital Space".to_string(),
            medium: "Digital painting".to_string(),
            content_hash: ContentHash::default(),
        },
        parents: vec![],
        anchor: Some(Anchor::Atomic(AtomicAnchor {
            timestamp: std::time::SystemTime::now(),
            precision: TimePrecision::Second,
            source: "system-clock".to_string(),
        })),
        ephemeral_provenance: None,
    };

    let entry = spine.create_entry(EntryType::TemporalMoment {
        moment_id: art_moment.id,
        moment: Box::new(art_moment.clone()),
    });
    spine.append(entry)?;

    println!("   Created art moment");
    println!("   Category: {}", art_moment.context.category());
    println!(
        "   Title: {:?}\n",
        if let MomentContext::ArtCreation { title, .. } = &art_moment.context {
            title
        } else {
            "Unknown"
        }
    );

    // === Example 3: Life Event Moment ===
    println!("🎓 Example 3: Life Event");

    let life_moment = Moment {
        id: ContentHash::default(),
        timestamp: std::time::SystemTime::now(),
        agent: owner.to_string(),
        state_hash: ContentHash::default(),
        signature: Signature::empty(),
        context: MomentContext::LifeEvent {
            event_type: "graduation".to_string(),
            participants: vec![owner.to_string()],
            description: "Graduated with honors from University of Life".to_string(),
        },
        parents: vec![],
        anchor: Some(Anchor::Atomic(AtomicAnchor {
            timestamp: std::time::SystemTime::now(),
            precision: TimePrecision::Second,
            source: "system-clock".to_string(),
        })),
        ephemeral_provenance: None,
    };

    let entry = spine.create_entry(EntryType::TemporalMoment {
        moment_id: life_moment.id,
        moment: Box::new(life_moment.clone()),
    });
    spine.append(entry)?;

    println!("   Recorded life event");
    println!("   Category: {}", life_moment.context.category());
    println!("   Type: graduation\n");

    // === Example 4: Scientific Experiment Moment ===
    println!("🔬 Example 4: Scientific Experiment");

    let experiment_moment = Moment {
        id: ContentHash::default(),
        timestamp: std::time::SystemTime::now(),
        agent: owner.to_string(),
        state_hash: ContentHash::default(),
        signature: Signature::empty(),
        context: MomentContext::Experiment {
            hypothesis: "Temporal moments can track any domain universally".to_string(),
            result: "Hypothesis confirmed - flexible context system works!".to_string(),
            data_hash: ContentHash::default(),
        },
        parents: vec![],
        anchor: Some(Anchor::Atomic(AtomicAnchor {
            timestamp: std::time::SystemTime::now(),
            precision: TimePrecision::Nanosecond,
            source: "atomic-clock".to_string(),
        })),
        ephemeral_provenance: None,
    };

    let entry = spine.create_entry(EntryType::TemporalMoment {
        moment_id: experiment_moment.id,
        moment: Box::new(experiment_moment.clone()),
    });
    spine.append(entry)?;

    println!("   Documented experiment");
    println!("   Category: {}", experiment_moment.context.category());
    println!("   Result: Success!\n");

    // === Verify the spine ===
    println!("🔍 Verifying spine integrity...");
    let verification = spine.verify();

    if verification.valid {
        println!("✅ Spine verification passed!");
    } else {
        println!(
            "❌ Spine verification failed: {} errors",
            verification.errors.len()
        );
    }

    println!("\n📊 Spine stats:");
    println!("   Height: {} entries", spine.height);
    println!("   State: {:?}", spine.state);

    // Show all moment categories
    println!("\n🎯 Moment categories tracked:");
    for category in spine.entries().iter().filter_map(|e| {
        if let EntryType::TemporalMoment { moment, .. } = &e.entry_type {
            Some(moment.context.category())
        } else {
            None
        }
    }) {
        println!("   - {}", category);
    }

    println!("\n🎉 Temporal moments demonstration complete!");
    println!("    Time is the primitive, not version control.");

    Ok(())
}
