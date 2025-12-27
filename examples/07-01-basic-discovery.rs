//! Showcase 07-01: Basic Songbird Discovery
//!
//! This example demonstrates:
//! - Connecting to Songbird
//! - Discovering all available services
//! - Filtering by capability
//!
//! Prerequisites:
//! - Songbird running at http://localhost:8082
//!
//! Run: cargo run --example 07-01-basic-discovery

use loam_spine_core::constants::DEFAULT_DISCOVERY_PORT;
use loam_spine_core::discovery_client::DiscoveryClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  🔍 Showcase 07-01: Basic Songbird Discovery        ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    // Connect to Songbird
    let discovery_endpoint = format!("http://localhost:{DEFAULT_DISCOVERY_PORT}");
    println!("🔍 Connecting to Songbird at {discovery_endpoint}...");
    let client = match DiscoveryClient::connect(&discovery_endpoint).await {
        Ok(c) => {
            println!("✅ Connected to Songbird\n");
            c
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to Songbird: {e}");
            eprintln!("\n💡 Make sure Songbird is running:");
            eprintln!("   cd ../../bins && ./songbird-cli tower start");
            return Err(e.into());
        }
    };

    // Discover all services
    println!("📡 Discovering all services...");
    match client.discover_all().await {
        Ok(services) => {
            println!("✅ Found {} service(s):\n", services.len());

            for service in &services {
                println!("  • {} ({}) at {}", 
                    service.name,
                    service.capabilities.first().unwrap_or(&"unknown".to_string()),
                    service.endpoint
                );
                println!("    Capabilities: {}", service.capabilities.join(", "));
                println!("    Healthy: {}\n", if service.healthy { "✅" } else { "❌" });
            }
        }
        Err(e) => {
            println!("⚠️  No services discovered: {e}\n");
        }
    }

    // Discover signing capability
    println!("🔎 Discovering 'signing' capability...");
    match client.discover_capability("signing").await {
        Ok(services) => {
            if services.is_empty() {
                println!("⚠️  No services found with 'signing' capability\n");
                println!("💡 Start BearDog to provide signing:");
                println!("   cd ../../phase1/bearDog && cargo run --release -- service start\n");
            } else {
                println!("✅ Found {} service(s) with 'signing':\n", services.len());
                for service in services {
                    println!("  • {} at {}", service.name, service.endpoint);
                }
                println!();
            }
        }
        Err(e) => {
            println!("❌ Discovery failed: {e}\n");
        }
    }

    // Discover storage capability
    println!("🔎 Discovering 'storage' capability...");
    match client.discover_capability("storage").await {
        Ok(services) => {
            if services.is_empty() {
                println!("⚠️  No services found with 'storage' capability\n");
                println!("💡 Start NestGate to provide storage:");
                println!("   cd ../../phase1/nestGate && cargo run --release -- service start\n");
            } else {
                println!("✅ Found {} service(s) with 'storage':\n", services.len());
                for service in services {
                    println!("  • {} at {}", service.name, service.endpoint);
                }
                println!();
            }
        }
        Err(e) => {
            println!("❌ Discovery failed: {e}\n");
        }
    }

    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  ✅ Basic Discovery Complete                         ║");
    println!("╚══════════════════════════════════════════════════════╝");

    Ok(())
}

