// SPDX-License-Identifier: AGPL-3.0-or-later

//! # 🦴 Demo: RPC Service
//!
//! Demonstrates the Pure Rust RPC API.
//!
//! This demo shows:
//! - Creating an RPC service
//! - Making RPC calls
//! - Health checks
//!
//! ## Run
//! ```bash
//! cargo run -p loam-spine-api --example demo_rpc_service
//! ```

use loam_spine_api::{
    LoamSpineRpcService,
    types::{CreateSpineRequest, GetSpineRequest, HealthCheckRequest},
};
use loam_spine_core::Did;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦴 Demo: RPC Service");
    println!("====================\n");

    // 1. Create RPC service
    println!("1. Create RPC Service");
    println!("---------------------");
    let service = LoamSpineRpcService::default_service();
    println!("✓ RPC service created");
    println!("  Pure Rust: tarpc + pure JSON-RPC");
    println!("  No gRPC, no protobuf!");
    println!();

    // 2. Health check
    println!("2. Health Check");
    println!("---------------");
    let health_req = HealthCheckRequest {
        include_details: true,
    };
    let health_resp = service.health_check(health_req).await?;
    println!("✓ Health check response:");
    println!("  Status: {:?}", health_resp.status);
    if let Some(report) = &health_resp.report {
        println!("  Name: {}", report.name);
        println!("  Version: {}", report.version);
    }
    println!();

    // 3. Create spine via RPC
    println!("3. Create Spine via RPC");
    println!("-----------------------");
    let owner = Did::new("did:key:z6MkRpcDemo");
    let create_req = CreateSpineRequest {
        owner: owner.clone(),
        name: "RPC Demo Spine".to_string(),
        config: None,
    };
    let create_resp = service.create_spine(create_req).await?;
    println!("✓ Spine created via RPC:");
    println!("  ID: {}", create_resp.spine_id);
    println!("  Genesis: {:?}", create_resp.genesis_hash);
    println!();

    // 4. Get spine via RPC
    println!("4. Get Spine via RPC");
    println!("--------------------");
    let get_req = GetSpineRequest {
        spine_id: create_resp.spine_id,
    };
    let get_resp = service.get_spine(get_req).await?;
    println!("✓ Spine retrieved via RPC:");
    println!("  Found: {}", get_resp.found);
    if let Some(spine) = &get_resp.spine {
        println!("  ID: {}", spine.id);
        println!("  Name: {:?}", spine.name);
        println!("  Height: {}", spine.height);
        println!("  State: {:?}", spine.state);
    }
    println!();

    // 5. Summary
    println!("═══════════════════════════════════════════════════════════════");
    println!("  RPC API SUMMARY");
    println!("═══════════════════════════════════════════════════════════════");
    println!();
    println!("18 RPC Methods Available:");
    println!("  Spine:       create_spine, get_spine, seal_spine");
    println!("  Entry:       append_entry, get_entry, get_tip");
    println!("  Certificate: mint, get, transfer, loan, return");
    println!("  Waypoint:    anchor_slice, checkout_slice");
    println!("  Proof:       generate_inclusion_proof, verify_inclusion_proof");
    println!("  Integration: commit_session, commit_braid");
    println!("  Health:      health_check");
    println!();
    println!("Dual Protocol:");
    println!("  • tarpc - Structured JSON/TCP, fast, primal-to-primal");
    println!("  • JSON-RPC 2.0 - Language-agnostic external clients");
    println!("  (Ports configured at runtime via environment)");
    println!();

    println!("🎉 Success!");
    println!("===========");
    println!("You've used the Pure Rust RPC API.");
    println!();
    println!("Key concepts:");
    println!("  • No gRPC, no protobuf, no C++ tooling");
    println!("  • Native serde serialization");
    println!("  • Standard cargo build");

    Ok(())
}
