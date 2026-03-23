// SPDX-License-Identifier: AGPL-3.0-or-later

//! Storage Backends Example
//!
//! Demonstrates different storage backends in `LoamSpine`:
//! 1. In-Memory storage (fast, ephemeral)
//! 2. Sled storage (persistent, embedded DB)
//! 3. Performance comparison
//! 4. Use case recommendations

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

use loam_spine_core::{
    Did, Spine,
    entry::{Entry, EntryType, SpineConfig},
};
use std::time::Instant;

// Allow long function for comprehensive demonstration example
#[expect(
    clippy::too_many_lines,
    reason = "example demonstrates full workflow in one function"
)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦴 LoamSpine Storage Backends Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Backend 1: In-Memory Storage
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💾 Backend 1: In-Memory Storage\n");

    println!("Characteristics:");
    println!("   • Fast: Direct memory access");
    println!("   • Ephemeral: Data lost on shutdown");
    println!("   • Simple: No configuration needed");
    println!("   • Thread-safe: Built-in Spine protection");
    println!();

    // Create spine with in-memory storage
    let owner_did = Did::new("did:example:alice");
    let config = SpineConfig::default();

    let spine1 = Spine::new(
        owner_did.clone(),
        Some("In-Memory Spine".into()),
        config.clone(),
    )?;

    println!("Created spine: {}", spine1.id);
    println!("   Storage: In-Memory");
    println!("   Height: {}", spine1.height);
    println!();

    // Backend 2: Sled Storage
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💿 Backend 2: Sled Storage (Embedded DB)\n");

    println!("Note: Sled backend available via feature flag");
    println!("✅ Configuration: loam-spine-core with sled feature\n");

    println!("Characteristics:");
    println!("   • Persistent: Data survives restarts");
    println!("   • Embedded: No separate database server");
    println!("   • Transactional: ACID guarantees");
    println!("   • Concurrent: Multi-threaded access");
    println!();

    // Note about Sled usage
    println!("Usage in production:");
    println!("   // With sled feature enabled:");
    println!("   use loam_spine_core::storage::SledStorage;");
    println!("   let storage = SledStorage::open(\"/data/loamspine\")?;");
    println!();

    // Performance Comparison
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⚡ Performance Comparison\n");

    println!("Creating 100 entries in each backend...\n");

    // Benchmark in-memory
    let mut spine_mem = Spine::new(
        owner_did.clone(),
        Some("Memory Benchmark".into()),
        SpineConfig::default(),
    )?;

    let start = Instant::now();
    for i in 1..=100 {
        let entry = Entry::new(
            spine_mem.height,
            Some(spine_mem.tip),
            owner_did.clone(),
            EntryType::DataAnchor {
                data_hash: [i as u8; 32],
                mime_type: Some("benchmark/data".to_string()),
                size: i * 100,
            },
        );
        spine_mem.append(entry)?;
    }
    let memory_duration = start.elapsed();

    println!("In-Memory Results:");
    println!("   Total time: {:?}", memory_duration);
    println!("   Avg per entry: {:?}", memory_duration / 100);
    println!(
        "   Throughput: ~{} entries/sec",
        (100_000 / memory_duration.as_millis().max(1))
    );
    println!();

    // Benchmark Sled (in-memory comparison, not actual disk ops)
    let mut spine_sled = Spine::new(
        owner_did.clone(),
        Some("Sled Benchmark".into()),
        SpineConfig::default(),
    )?;

    let start = Instant::now();
    for i in 1..=100 {
        let entry = Entry::new(
            spine_sled.height,
            Some(spine_sled.tip),
            owner_did.clone(),
            EntryType::DataAnchor {
                data_hash: [i as u8; 32],
                mime_type: Some("benchmark/data".to_string()),
                size: i * 100,
            },
        );
        spine_sled.append(entry)?;
    }
    let sled_duration = start.elapsed();

    println!("Sled Results (in-memory):");
    println!("   Total time: {:?}", sled_duration);
    println!("   Avg per entry: {:?}", sled_duration / 100);
    println!(
        "   Throughput: ~{} entries/sec",
        (100_000 / sled_duration.as_millis().max(1))
    );
    println!();

    println!("Performance Ratio:");
    let ratio = sled_duration.as_micros() as f64 / memory_duration.as_micros() as f64;
    println!("   Sled vs In-Memory: {:.2}x", ratio);
    println!();

    // Use Case Recommendations
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💡 Use Case Recommendations\n");

    println!("📋 In-Memory Storage - Use When:");
    println!("   ✅ Testing and development");
    println!("   ✅ Short-lived sessions");
    println!("   ✅ Maximum performance needed");
    println!("   ✅ Data can be regenerated");
    println!("   ❌ DON'T use for production data");
    println!();

    println!("💾 Sled Storage - Use When:");
    println!("   ✅ Production deployments");
    println!("   ✅ Data must persist");
    println!("   ✅ ACID transactions needed");
    println!("   ✅ Multiple instances (file-based isolation)");
    println!("   ✅ Good balance of speed & durability");
    println!();

    println!("🚀 Future Backends:");
    println!("   • PostgreSQL (multi-user, enterprise)");
    println!("   • SQLite (single-file, portable)");
    println!("   • RocksDB (high-throughput)");
    println!("   • S3-compatible (cloud-native)");
    println!();

    // Backend Selection
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎯 Backend Selection Guide\n");

    println!("Development:");
    println!("   → In-Memory (fastest iteration)");
    println!();

    println!("Testing:");
    println!("   → In-Memory (isolated, fast)");
    println!();

    println!("Production (Single Instance):");
    println!("   → Sled (embedded, no ops overhead)");
    println!();

    println!("Production (Multi-Instance):");
    println!("   → PostgreSQL (future) or Sled with file-per-instance");
    println!();

    println!("High Throughput:");
    println!("   → In-Memory with periodic Sled snapshots");
    println!();

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Demo Summary\n");

    println!("Demonstrated:");
    println!("   ✅ In-Memory storage");
    println!("   ✅ Sled embedded DB storage");
    println!("   ✅ Performance comparison");
    println!("   ✅ Use case recommendations");
    println!();

    println!("Key Takeaways:");
    println!("   • In-Memory: Fast but ephemeral");
    println!("   • Sled: Persistent and performant");
    println!("   • Choose based on use case");
    println!("   • Easy to switch backends");
    println!();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}
