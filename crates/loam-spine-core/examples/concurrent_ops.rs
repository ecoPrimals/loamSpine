// SPDX-License-Identifier: AGPL-3.0-only

//! Concurrent Operations Example
//!
//! Demonstrates thread-safe concurrent operations in LoamSpine:
//! 1. Multiple threads appending to spine
//! 2. Thread-safety guarantees
//! 3. Performance under concurrency
//! 4. Best practices

// Allow patterns for demonstration code clarity
#![allow(clippy::too_many_lines)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::doc_markdown)]
#![allow(unused_variables)]

use loam_spine_core::{
    Did, Spine,
    entry::{Entry, EntryType, SpineConfig},
};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦴 LoamSpine Concurrent Operations Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Create spine
    let owner_did = Did::new("did:example:alice");
    let config = SpineConfig::default();

    let spine = Spine::new(
        owner_did.clone(),
        Some("Concurrent Demo Spine".into()),
        config,
    )?;

    println!("📋 Created spine: {}", spine.id);
    println!("   Initial height: {}", spine.height);
    println!();

    // Wrap spine in Arc<Mutex> for thread-safe sharing
    let spine = Arc::new(Mutex::new(spine));

    // Demo 1: Sequential baseline
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Baseline: Sequential Operations\n");

    let baseline_spine = Arc::new(Mutex::new(Spine::new(
        owner_did.clone(),
        Some("Sequential Baseline".into()),
        SpineConfig::default(),
    )?));

    let start = Instant::now();
    for i in 1..=100 {
        let mut spine_lock = baseline_spine.lock().unwrap();
        let entry = Entry::new(
            spine_lock.height,
            Some(spine_lock.tip),
            owner_did.clone(),
            EntryType::DataAnchor {
                data_hash: [i as u8; 32],
                mime_type: Some("benchmark/data".to_string()),
                size: i * 100,
            },
        );
        spine_lock.append(entry)?;
    }
    let sequential_duration = start.elapsed();

    let final_height = baseline_spine.lock().unwrap().height;
    println!("✅ Sequential Results:");
    println!("   Entries added: 100");
    println!("   Final height: {}", final_height);
    println!("   Total time: {:?}", sequential_duration);
    println!("   Avg per entry: {:?}", sequential_duration / 100);
    println!();

    // Demo 2: Concurrent operations (4 threads)
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⚡ Concurrent: 4 Threads, 25 Entries Each\n");

    let concurrent_spine = Arc::new(Mutex::new(Spine::new(
        owner_did.clone(),
        Some("Concurrent Test".into()),
        SpineConfig::default(),
    )?));

    let start = Instant::now();
    let mut handles = vec![];

    for thread_id in 0..4 {
        let spine_clone = Arc::clone(&concurrent_spine);
        let owner_clone = owner_did.clone();

        let handle = thread::spawn(move || {
            for i in 1..=25 {
                let mut spine_lock = spine_clone.lock().unwrap();
                let entry = Entry::new(
                    spine_lock.height,
                    Some(spine_lock.tip),
                    owner_clone.clone(),
                    EntryType::DataAnchor {
                        data_hash: [(thread_id * 25 + i) as u8; 32],
                        mime_type: Some(format!("thread-{}/data", thread_id)),
                        size: i * 100,
                    },
                );

                if let Err(e) = spine_lock.append(entry) {
                    eprintln!("Thread {} error: {}", thread_id, e);
                }
                // Release lock between operations
                drop(spine_lock);
            }
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    let concurrent_duration = start.elapsed();
    let final_height = concurrent_spine.lock().unwrap().height;

    println!("✅ Concurrent Results:");
    println!("   Threads: 4");
    println!("   Entries per thread: 25");
    println!("   Total entries: 100");
    println!("   Final height: {}", final_height);
    println!("   Total time: {:?}", concurrent_duration);
    println!("   Avg per entry: {:?}", concurrent_duration / 100);
    println!();

    // Comparison
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📈 Performance Comparison\n");

    println!("Sequential:");
    println!("   Time: {:?}", sequential_duration);
    println!(
        "   Throughput: {:.0} ops/sec",
        100.0 / sequential_duration.as_secs_f64()
    );
    println!();

    println!("Concurrent (4 threads):");
    println!("   Time: {:?}", concurrent_duration);
    println!(
        "   Throughput: {:.0} ops/sec",
        100.0 / concurrent_duration.as_secs_f64()
    );
    println!();

    let speedup = sequential_duration.as_secs_f64() / concurrent_duration.as_secs_f64();
    println!("Speedup: {:.2}x", speedup);
    println!();

    if speedup > 1.0 {
        println!("✅ Concurrency improved performance!");
    } else {
        println!("⚠️  Note: Lock contention limits speedup");
        println!("   (Expected for append-heavy workloads)");
    }
    println!();

    // Thread Safety Guarantees
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔒 Thread Safety Guarantees\n");

    println!("LoamSpine ensures:");
    println!("   ✅ No data races");
    println!("   ✅ Sequential consistency");
    println!("   ✅ Chain integrity maintained");
    println!("   ✅ No lost updates");
    println!();

    println!("How it works:");
    println!("   • Spine wrapped in Arc<Mutex<T>>");
    println!("   • Mutex ensures exclusive write access");
    println!("   • Each append is atomic");
    println!("   • Chain links always valid");
    println!();

    // Best Practices
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💡 Concurrency Best Practices\n");

    println!("DO ✅:");
    println!("   • Use Arc<Mutex<Spine>> for shared access");
    println!("   • Keep critical sections small");
    println!("   • Release locks between operations");
    println!("   • Handle errors gracefully");
    println!("   • Consider batching for high throughput");
    println!();

    println!("DON'T ❌:");
    println!("   • Don't hold locks across async boundaries");
    println!("   • Don't share Spine without synchronization");
    println!("   • Don't ignore append errors");
    println!("   • Don't create excessive contention");
    println!();

    // Use Cases
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎯 Concurrent Use Cases\n");

    println!("Multi-User Systems:");
    println!("   • Multiple users appending to shared spine");
    println!("   • Example: Collaborative audit log");
    println!();

    println!("High-Throughput Ingestion:");
    println!("   • Multiple data sources → single spine");
    println!("   • Example: IoT sensor aggregation");
    println!();

    println!("Distributed Workers:");
    println!("   • Worker pool processing tasks");
    println!("   • Example: Job result logging");
    println!();

    println!("Real-Time Analytics:");
    println!("   • Concurrent writes + reads");
    println!("   • Example: Live event tracking");
    println!();

    // Summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Demo Summary\n");

    println!("Demonstrated:");
    println!("   ✅ Sequential baseline");
    println!("   ✅ Concurrent operations (4 threads)");
    println!("   ✅ Thread-safety guarantees");
    println!("   ✅ Performance comparison");
    println!("   ✅ Best practices");
    println!();

    println!("Key Takeaways:");
    println!("   • LoamSpine is thread-safe");
    println!("   • Use Arc<Mutex<T>> for sharing");
    println!("   • Lock contention is expected for writes");
    println!("   • Consider batching for high throughput");
    println!();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}
