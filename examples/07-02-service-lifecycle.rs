//! Showcase 07-02: Service Lifecycle with Songbird
//!
//! This example demonstrates:
//! - Creating LoamSpine service with Songbird integration
//! - Auto-advertising on startup
//! - Background heartbeat task
//! - Graceful shutdown
//!
//! Prerequisites:
//! - Songbird running at http://localhost:8082
//!
//! Run: cargo run --example 07-02-service-lifecycle

use loam_spine_core::config::{DiscoveryConfig, DiscoveryMethod, LoamSpineConfig};
use loam_spine_core::service::{LifecycleManager, LoamSpineService};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  🦴 Showcase 07-02: Service Lifecycle               ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    // Create configuration with Songbird enabled
    println!("🔧 Creating configuration with Songbird integration...");
    let config = LoamSpineConfig::default()
        .with_songbird("http://localhost:8082")
        .with_discovery(DiscoveryConfig {
            songbird_enabled: true,
            songbird_endpoint: Some("http://localhost:8082".to_string()),
            auto_advertise: true,
            heartbeat_interval_seconds: 10, // Short interval for demo
            methods: vec![
                DiscoveryMethod::Environment,
                DiscoveryMethod::Songbird,
                DiscoveryMethod::LocalBinaries,
                DiscoveryMethod::Fallback,
            ],
        });
    println!("✅ Configuration created\n");

    // Create service
    println!("🦴 Creating LoamSpine service...");
    let service = LoamSpineService::new();
    println!("✅ Service created\n");

    // Create lifecycle manager
    println!("♻️  Creating lifecycle manager...");
    let mut manager = LifecycleManager::new(service, config);
    println!("✅ Lifecycle manager created\n");

    // Start lifecycle (this will advertise to Songbird)
    println!("🚀 Starting service lifecycle...");
    println!("   This will:");
    println!("   1. Connect to Songbird");
    println!("   2. Advertise LoamSpine capabilities");
    println!("   3. Start background heartbeat task\n");

    match manager.start().await {
        Ok(()) => {
            println!("✅ Lifecycle started successfully\n");

            // Keep service running for a bit to demonstrate heartbeat
            println!("❤️  Service running with heartbeat (10s interval)...");
            println!("   Watch the logs for heartbeat messages");
            println!("   Press Ctrl+C to stop (or wait 30s for auto-stop)\n");

            // Sleep for 30 seconds to show heartbeats
            for i in 1..=30 {
                sleep(Duration::from_secs(1)).await;
                if i % 10 == 0 {
                    println!("   ⏱️  Running for {i}s...");
                }
            }

            println!("\n🛑 Stopping service lifecycle...");
            manager.stop().await?;
            println!("✅ Lifecycle stopped gracefully\n");
        }
        Err(e) => {
            eprintln!("❌ Failed to start lifecycle: {e}");
            eprintln!("\n💡 Make sure Songbird is running:");
            eprintln!("   cd ../../bins && ./songbird-cli tower start\n");
            eprintln!("   If Songbird is unavailable, LoamSpine will continue");
            eprintln!("   without discovery (graceful degradation).");
            return Err(e.into());
        }
    }

    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  ✅ Service Lifecycle Complete                       ║");
    println!("╚══════════════════════════════════════════════════════╝");

    Ok(())
}

