//! LoamSpine Standalone Service
//!
//! This binary provides a standalone LoamSpine service that can be deployed
//! and coordinated independently by BiomeOS. It supports dual-protocol RPC
//! (tarpc + JSON-RPC) for maximum flexibility.
//!
//! ## Usage
//!
//! ```bash
//! # Run with defaults
//! ./loamspine-service
//!
//! # Custom ports via environment variables
//! export TARPC_PORT=9001
//! export JSONRPC_PORT=8080
//! ./loamspine-service
//!
//! # Or via command line
//! ./loamspine-service --tarpc-port 9001 --jsonrpc-port 8080
//! ```
//!
//! ## Discovery
//!
//! The service automatically registers with the discovery service (e.g., Songbird)
//! if configured via `DISCOVERY_ENDPOINT` environment variable.
//!
//! ```bash
//! export DISCOVERY_ENDPOINT=http://songbird:8082
//! ./loamspine-service
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use loam_spine_api::{run_jsonrpc_server, run_tarpc_server, LoamSpineRpcService};
use loam_spine_core::config::LoamSpineConfig;
use loam_spine_core::service::LifecycleManager;
use loam_spine_core::LoamSpineService;
use std::net::SocketAddr;
use tracing::{error, info};

/// Parse command line arguments
struct Args {
    tarpc_port: u16,
    jsonrpc_port: u16,
}

impl Args {
    fn parse() -> Self {
        let mut tarpc_port = std::env::var("TARPC_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(9001);

        let mut jsonrpc_port = std::env::var("JSONRPC_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8080);

        // Parse command line args
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--tarpc-port" => {
                    if let Some(port_str) = args.next() {
                        tarpc_port = port_str.parse().unwrap_or(tarpc_port);
                    }
                }
                "--jsonrpc-port" => {
                    if let Some(port_str) = args.next() {
                        jsonrpc_port = port_str.parse().unwrap_or(jsonrpc_port);
                    }
                }
                "--help" | "-h" => {
                    println!("LoamSpine Standalone Service");
                    println!();
                    println!("USAGE:");
                    println!("    loamspine-service [OPTIONS]");
                    println!();
                    println!("OPTIONS:");
                    println!("    --tarpc-port PORT     tarpc server port (default: 9001, env: TARPC_PORT)");
                    println!("    --jsonrpc-port PORT   JSON-RPC server port (default: 8080, env: JSONRPC_PORT)");
                    println!("    --help, -h            Print this help message");
                    println!();
                    println!("ENVIRONMENT:");
                    println!("    TARPC_PORT            tarpc server port");
                    println!("    JSONRPC_PORT          JSON-RPC server port");
                    println!("    DISCOVERY_ENDPOINT    Discovery service endpoint (e.g., http://songbird:8082)");
                    println!("    RUST_LOG              Logging level (e.g., info, debug)");
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("Unknown argument: {}", arg);
                    eprintln!("Use --help for usage information");
                    std::process::exit(1);
                }
            }
        }

        Self {
            tarpc_port,
            jsonrpc_port,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Parse arguments
    let args = Args::parse();

    info!("🦴 LoamSpine Standalone Service");
    info!("   Version: {}", env!("CARGO_PKG_VERSION"));
    info!("   tarpc port: {}", args.tarpc_port);
    info!("   JSON-RPC port: {}", args.jsonrpc_port);

    // Create service and lifecycle manager
    let service = LoamSpineService::new();
    let config = LoamSpineConfig::default();
    let mut lifecycle = LifecycleManager::new(service.clone(), config);

    // Start lifecycle (discovery, heartbeat, etc.)
    lifecycle.start().await?;

    // Create RPC service wrapper
    let rpc_service = LoamSpineRpcService::new(service);

    // Start tarpc server
    let tarpc_addr: SocketAddr = format!("0.0.0.0:{}", args.tarpc_port).parse()?;
    let rpc_service_tarpc = rpc_service.clone();
    let tarpc_handle = tokio::spawn(async move {
        info!("🚀 Starting tarpc server on {}", tarpc_addr);
        if let Err(e) = run_tarpc_server(tarpc_addr, rpc_service_tarpc).await {
            error!("tarpc server error: {}", e);
        }
    });

    // Start JSON-RPC server
    let jsonrpc_addr: SocketAddr = format!("0.0.0.0:{}", args.jsonrpc_port).parse()?;
    let jsonrpc_handle = tokio::spawn(async move {
        info!("🌐 Starting JSON-RPC server on {}", jsonrpc_addr);
        if let Err(e) = run_jsonrpc_server(jsonrpc_addr, rpc_service).await {
            error!("JSON-RPC server error: {}", e);
        }
    });

    info!("✅ LoamSpine service started successfully");
    info!("   tarpc:    tarpc://0.0.0.0:{}", args.tarpc_port);
    info!("   JSON-RPC: http://0.0.0.0:{}", args.jsonrpc_port);
    info!("");
    info!("Press Ctrl+C to stop...");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;

    info!("🛑 Shutdown signal received, stopping gracefully...");

    // Stop lifecycle (deregister from discovery, etc.)
    lifecycle.stop().await?;

    // Abort server tasks
    tarpc_handle.abort();
    jsonrpc_handle.abort();

    info!("👋 LoamSpine service stopped");

    Ok(())
}

