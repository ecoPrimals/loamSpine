// SPDX-License-Identifier: AGPL-3.0-or-later

//! `LoamSpine` — the permanence layer for `ecoPrimals`.
//!
//! UniBin-compliant single binary with subcommand structure.
//! Supports dual-protocol RPC (tarpc + JSON-RPC 2.0) with all
//! configuration resolved from environment or CLI at runtime.
//!
//! # Usage
//!
//! ```bash
//! # Run with defaults (discovered from environment)
//! loamspine server
//!
//! # Custom ports via CLI
//! loamspine server --tarpc-port 9001 --jsonrpc-port 8080
//!
//! # Custom ports via environment
//! LOAMSPINE_TARPC_PORT=9001 LOAMSPINE_JSONRPC_PORT=8080 loamspine server
//!
//! # Discovery registration
//! DISCOVERY_ENDPOINT=http://registry:8082 loamspine server
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::borrow::Cow;
use std::io::{Write as _, stdout};
use std::net::{IpAddr, SocketAddr};

use clap::{Parser, Subcommand};
use loam_spine_api::{LoamSpineRpcService, run_jsonrpc_server, run_tarpc_server};
use loam_spine_core::LoamSpineService;
use loam_spine_core::config::LoamSpineConfig;
use loam_spine_core::constants::network;
use loam_spine_core::error::OrExit;
use loam_spine_core::service::LifecycleManager;
use tracing::{error, info, warn};

/// `LoamSpine` — permanent ledger for the `ecoPrimals` ecosystem.
#[derive(Parser)]
#[command(
    name = "loamspine",
    version,
    about = "Permanence layer — selective memory & certificates for ecoPrimals"
)]
struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    command: Command,
}

/// Available subcommands (`UniBin` standard).
#[derive(Subcommand)]
enum Command {
    /// Start the `LoamSpine` service (`tarpc` + JSON-RPC dual protocol).
    Server {
        /// `tarpc` structured RPC port (env: `LOAMSPINE_TARPC_PORT`, `TARPC_PORT`).
        #[arg(long)]
        tarpc_port: Option<u16>,

        /// JSON-RPC 2.0 TCP port (`UniBin` standard flag).
        ///
        /// Alias for `--jsonrpc-port`. Follows `UNIBIN_ARCHITECTURE_STANDARD.md` v1.1.
        #[arg(long, conflicts_with = "jsonrpc_port")]
        port: Option<u16>,

        /// JSON-RPC 2.0 port (env: `LOAMSPINE_JSONRPC_PORT`, `JSONRPC_PORT`).
        #[arg(long, conflicts_with = "port")]
        jsonrpc_port: Option<u16>,

        /// Bind address (env: `LOAMSPINE_BIND_ADDRESS`, `BIND_ADDRESS`).
        #[arg(long)]
        bind_address: Option<String>,

        /// UDS socket path override (env: `LOAMSPINE_SOCKET`).
        ///
        /// Explicit socket path for launcher/orchestrator wiring.
        /// When omitted, resolved from `LOAMSPINE_SOCKET` env, then
        /// `$XDG_RUNTIME_DIR/biomeos/permanence.sock`, then platform default.
        #[arg(long)]
        socket: Option<String>,

        /// Use abstract UDS namespace (Linux only, `UniBin` standard).
        ///
        /// When set, the UDS socket is created in the abstract namespace
        /// instead of the filesystem, avoiding cleanup issues on crash.
        #[arg(long)]
        r#abstract: bool,
    },

    /// List capabilities provided by this primal (`UniBin` standard).
    Capabilities,

    /// Show socket path for `NeuralAPI` IPC (`UniBin` standard).
    Socket,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Server {
            tarpc_port,
            port,
            jsonrpc_port,
            bind_address,
            socket,
            r#abstract,
        } => {
            run_server(
                tarpc_port,
                port.or(jsonrpc_port),
                bind_address,
                socket,
                r#abstract,
            )
            .await?;
        }
        Command::Capabilities => {
            writeln!(
                stdout(),
                "{}",
                loam_spine_core::neural_api::capability_list_pretty()
            )?;
        }
        Command::Socket => {
            writeln!(
                stdout(),
                "{}",
                loam_spine_core::neural_api::resolve_socket_path().display()
            )?;
        }
    }

    Ok(())
}

async fn run_server(
    tarpc_port_override: Option<u16>,
    jsonrpc_port_override: Option<u16>,
    bind_address_override: Option<String>,
    socket_override: Option<String>,
    abstract_socket: bool,
) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let resolved_tarpc_port = tarpc_port_override.unwrap_or_else(network::actual_tarpc_port);
    let resolved_jsonrpc_port = jsonrpc_port_override.unwrap_or_else(network::actual_jsonrpc_port);
    let resolved_bind: Cow<'static, str> =
        bind_address_override.map_or_else(network::bind_address, Cow::Owned);

    // PRIMAL_SELF_KNOWLEDGE_STANDARD §3: refuse to start if FAMILY_ID + INSECURE
    loam_spine_core::neural_api::validate_security_config_from_env()
        .or_exit("Security configuration conflict");

    info!("LoamSpine Standalone Service");
    info!("  version: {}", env!("CARGO_PKG_VERSION"));
    info!("  tarpc port: {resolved_tarpc_port}");
    info!("  JSON-RPC port: {resolved_jsonrpc_port}");
    info!("  bind address: {resolved_bind}");
    if abstract_socket {
        info!("  UDS namespace: abstract (Linux)");
    }

    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.tarpc_endpoint = format!("http://{resolved_bind}:{resolved_tarpc_port}");
    config.discovery.jsonrpc_endpoint = format!("http://{resolved_bind}:{resolved_jsonrpc_port}");
    let mut lifecycle = LifecycleManager::new(service.clone(), config);
    lifecycle
        .start()
        .await
        .or_exit("Failed to start lifecycle manager");

    let rpc_service = LoamSpineRpcService::new(service);

    let ip: IpAddr = resolved_bind.parse().or_exit("Invalid bind address");
    let tarpc_addr = SocketAddr::new(ip, resolved_tarpc_port);
    let jsonrpc_addr = SocketAddr::new(ip, resolved_jsonrpc_port);

    let rpc_service_tarpc = rpc_service.clone();
    let rpc_service_jsonrpc = rpc_service.clone();
    let tarpc_handle = tokio::spawn(async move {
        info!("Starting tarpc server on {tarpc_addr}");
        if let Err(e) = run_tarpc_server(tarpc_addr, rpc_service_tarpc).await {
            error!("tarpc server error: {e}");
        }
    });

    let jsonrpc_handle = tokio::spawn(async move {
        info!("Starting JSON-RPC server on {jsonrpc_addr}");
        match run_jsonrpc_server(jsonrpc_addr, rpc_service_jsonrpc).await {
            Ok(mut handle) => handle.stopped().await,
            Err(e) => error!("JSON-RPC server error: {e}"),
        }
    });

    // --socket flag takes priority, then env-based resolution
    let socket_path = socket_override.map_or_else(
        loam_spine_core::neural_api::resolve_socket_path,
        std::path::PathBuf::from,
    );

    // BTSP Phase 2: resolve handshake config from environment.
    // When BIOMEOS_FAMILY_ID is set (non-default), BTSP handshake is mandatory
    // on UDS connections — all crypto delegated to BearDog via JSON-RPC.
    let btsp_config = loam_spine_core::btsp::BtspHandshakeConfig::from_env();
    if let Some(ref cfg) = btsp_config {
        info!(
            "BTSP Phase 2 active: family={}, provider={}",
            cfg.family_id,
            cfg.provider_socket.display()
        );
    }

    // Start UDS JSON-RPC server (IPC_COMPLIANCE_MATRIX requirement)
    #[cfg(unix)]
    let uds_handle = {
        match loam_spine_api::run_jsonrpc_uds_server(&socket_path, rpc_service, btsp_config).await {
            Ok(handle) => {
                info!("UDS JSON-RPC server listening on {}", socket_path.display());
                Some(handle)
            }
            Err(e) => {
                error!(
                    "Failed to start UDS JSON-RPC server at {}: {e}",
                    socket_path.display()
                );
                None
            }
        }
    };

    // Legacy backward-compat symlink: loamspine.sock → permanence.sock
    // per PRIMAL_SELF_KNOWLEDGE_STANDARD §3 "Legacy compatibility"
    #[cfg(unix)]
    let legacy_symlink = {
        let family_id = std::env::var("BIOMEOS_FAMILY_ID").ok();
        let link_path = loam_spine_core::neural_api::resolve_legacy_symlink_path(
            &socket_path,
            family_id.as_deref(),
        );
        if link_path != socket_path {
            let _ = std::fs::remove_file(&link_path);
            match std::os::unix::fs::symlink(&socket_path, &link_path) {
                Ok(()) => {
                    info!(
                        "Legacy symlink: {} → {}",
                        link_path.display(),
                        socket_path.display()
                    );
                    Some(link_path)
                }
                Err(e) => {
                    warn!(
                        "Could not create legacy symlink {}: {e}",
                        link_path.display()
                    );
                    None
                }
            }
        } else {
            None
        }
    };

    info!("LoamSpine service started successfully");
    info!("  tarpc:    tarpc://{resolved_bind}:{resolved_tarpc_port}");
    info!("  JSON-RPC: http://{resolved_bind}:{resolved_jsonrpc_port}");
    info!("  socket:   {}", socket_path.display());

    let signal_handler = loam_spine_core::service::signals::SignalHandler::new();

    // Cooperative shutdown: all server tasks are monitored symmetrically.
    // A failure in any transport (tarpc, JSON-RPC TCP, or UDS) triggers
    // orderly teardown rather than silent degradation.
    tokio::select! {
        result = signal_handler.wait_for_shutdown() => {
            if let Err(e) = result {
                error!("Signal handler error: {e}");
            }
        }
        result = tarpc_handle => {
            if let Err(e) = result {
                error!("tarpc task failed: {e}");
            }
        }
        result = jsonrpc_handle => {
            if let Err(e) = result {
                error!("JSON-RPC task failed: {e}");
            }
        }
    }

    // Graceful UDS server drain before lifecycle teardown
    #[cfg(unix)]
    if let Some(ref handle) = uds_handle {
        handle.stop();
    }

    lifecycle.stop().await?;

    // Clean up sockets and symlinks on graceful shutdown
    // per PRIMAL_SELF_KNOWLEDGE_STANDARD §3 requirement
    #[cfg(unix)]
    {
        // Drop the handle (removes socket file via Drop impl)
        drop(uds_handle);
        if let Some(link) = legacy_symlink {
            let _ = std::fs::remove_file(&link);
        }
    }

    info!("LoamSpine service stopped");

    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "tests use panic for unexpected CLI parse branches"
)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn cli_parse_capabilities() {
        let cli = Cli::parse_from(["loamspine", "capabilities"]);
        assert!(matches!(cli.command, Command::Capabilities));
    }

    #[test]
    fn cli_parse_socket() {
        let cli = Cli::parse_from(["loamspine", "socket"]);
        assert!(matches!(cli.command, Command::Socket));
    }

    #[test]
    fn cli_parse_server_defaults() {
        let cli = Cli::parse_from(["loamspine", "server"]);
        if let Command::Server {
            tarpc_port,
            port,
            jsonrpc_port,
            bind_address,
            socket,
            r#abstract,
        } = cli.command
        {
            assert!(tarpc_port.is_none());
            assert!(port.is_none());
            assert!(jsonrpc_port.is_none());
            assert!(bind_address.is_none());
            assert!(socket.is_none());
            assert!(!r#abstract);
        } else {
            panic!("expected Server variant");
        }
    }

    #[test]
    fn cli_parse_server_with_overrides() {
        let cli = Cli::parse_from([
            "loamspine",
            "server",
            "--tarpc-port",
            "9002",
            "--jsonrpc-port",
            "8081",
            "--bind-address",
            "127.0.0.1",
        ]);
        if let Command::Server {
            tarpc_port,
            port,
            jsonrpc_port,
            bind_address,
            socket,
            ..
        } = cli.command
        {
            assert_eq!(tarpc_port, Some(9002));
            assert!(port.is_none());
            assert_eq!(jsonrpc_port, Some(8081));
            assert_eq!(bind_address.as_deref(), Some("127.0.0.1"));
            assert!(socket.is_none());
        } else {
            panic!("expected Server variant");
        }
    }

    #[test]
    fn cli_parse_server_unibin_port_flag() {
        let cli = Cli::parse_from(["loamspine", "server", "--port", "7070"]);
        if let Command::Server {
            port, jsonrpc_port, ..
        } = cli.command
        {
            assert_eq!(port, Some(7070));
            assert!(jsonrpc_port.is_none());
        } else {
            panic!("expected Server variant");
        }
    }

    #[test]
    fn cli_parse_server_socket_flag() {
        let cli = Cli::parse_from([
            "loamspine",
            "server",
            "--socket",
            "/run/custom/permanence.sock",
        ]);
        if let Command::Server { socket, .. } = cli.command {
            assert_eq!(socket.as_deref(), Some("/run/custom/permanence.sock"));
        } else {
            panic!("expected Server variant");
        }
    }
}
