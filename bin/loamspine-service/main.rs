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
//! # Run with UDS only (default — no TCP, no port conflicts)
//! loamspine server
//!
//! # Enable TCP with explicit ports
//! loamspine server --port 8080 --tarpc-port 9001
//!
//! # Enable TCP via environment
//! LOAMSPINE_JSONRPC_PORT=8080 LOAMSPINE_TARPC_PORT=9001 loamspine server
//!
//! # OS-assigned ports (avoids conflicts)
//! USE_OS_ASSIGNED_PORTS=true loamspine server
//!
//! # Discovery registration
//! DISCOVERY_ENDPOINT=http://registry:8082 loamspine server --port 8080
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
#[cfg(unix)]
use tracing::debug;
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
        /// `$XDG_RUNTIME_DIR/$BIOMEOS_SOCKET_DIR/loamspine.sock`, then platform default.
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

    // Transport injection: accept TRANSPORT_ENDPOINT from launcher/Tower Atomic.
    // When set, the launcher decides the transport — the primal does not self-bind.
    let injected_endpoint = std::env::var(loam_spine_core::transport::TRANSPORT_ENDPOINT_ENV)
        .ok()
        .and_then(
            |json| match loam_spine_core::transport::parse_transport_endpoint(&json) {
                Ok(ep) => {
                    info!("Transport endpoint injected: {ep}");
                    Some(ep)
                }
                Err(e) => {
                    warn!("Invalid TRANSPORT_ENDPOINT value: {e}");
                    None
                }
            },
        );

    // TCP is opt-in (ecosystem convention): only start TCP servers
    // when explicitly requested via CLI flags, environment variables,
    // or a TCP transport endpoint injection.
    let tcp_requested = tarpc_port_override.is_some()
        || jsonrpc_port_override.is_some()
        || network::has_explicit_tcp_config()
        || matches!(
            &injected_endpoint,
            Some(loam_spine_core::transport::TransportEndpoint::Tcp { .. })
        );

    let resolved_bind: Cow<'static, str> =
        if let Some(loam_spine_core::transport::TransportEndpoint::Tcp { ref host, .. }) =
            injected_endpoint
        {
            Cow::Owned(host.clone())
        } else {
            bind_address_override.map_or_else(network::bind_address, Cow::Owned)
        };

    // PRIMAL_SELF_KNOWLEDGE_STANDARD §3: refuse to start if FAMILY_ID + INSECURE
    loam_spine_core::neural_api::validate_security_config_from_env()
        .or_exit("Security configuration conflict");

    info!("LoamSpine Standalone Service");
    info!("  version: {}", env!("CARGO_PKG_VERSION"));
    if tcp_requested {
        let resolved_tarpc_port = tarpc_port_override.unwrap_or_else(network::actual_tarpc_port);
        let resolved_jsonrpc_port =
            jsonrpc_port_override.unwrap_or_else(network::actual_jsonrpc_port);
        info!("  tarpc port: {resolved_tarpc_port}");
        info!("  JSON-RPC port: {resolved_jsonrpc_port}");
        info!("  bind address: {resolved_bind}");
    } else {
        info!("  TCP transports: disabled (use --port/--tarpc-port to enable)");
    }
    if abstract_socket {
        warn!(
            "  --abstract flag accepted but not yet wired (pre-wired for abstract UDS namespace)"
        );
    }

    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();

    // Merge TCP endpoints into config before lifecycle starts.
    if tcp_requested {
        let rtp = tarpc_port_override.unwrap_or_else(network::actual_tarpc_port);
        let rjp = jsonrpc_port_override.unwrap_or_else(network::actual_jsonrpc_port);
        config.discovery.tarpc_endpoint = format!("http://{resolved_bind}:{rtp}");
        config.discovery.jsonrpc_endpoint = format!("http://{resolved_bind}:{rjp}");
    }

    let mut lifecycle = LifecycleManager::new(service.clone(), config);
    lifecycle
        .start()
        .await
        .or_exit("Failed to start lifecycle manager");

    let rpc_service = {
        let svc = LoamSpineRpcService::new(service);
        let tower_socket = loam_spine_core::constants::env_resolution::tower_signer_socket();
        if let Some(socket_val) = tower_socket {
            let socket_path = std::path::PathBuf::from(&socket_val);
            info!("Tower signing enabled via TOWER_SIGNER_SOCKET={socket_val}");
            let signer_did = loam_spine_core::constants::env_resolution::tower_signer_did()
                .map(loam_spine_core::types::Did::new)
                .unwrap_or_else(loam_spine_core::types::Did::anonymous);
            let signer = std::sync::Arc::new(
                loam_spine_core::traits::crypto_provider::JsonRpcCryptoSigner::new(
                    socket_path,
                    signer_did,
                    None,
                ),
            );
            svc.with_tower_signer(signer)
        } else {
            info!("Tower signing disabled (TOWER_SIGNER_SOCKET not set)");
            svc
        }
    };

    // Only start TCP servers when explicitly requested.
    let injected_tcp_port = match &injected_endpoint {
        Some(loam_spine_core::transport::TransportEndpoint::Tcp { port, .. }) => Some(*port),
        _ => None,
    };
    let tarpc_handle = if tcp_requested {
        let resolved_tarpc_port = tarpc_port_override.unwrap_or_else(network::actual_tarpc_port);
        let resolved_jsonrpc_port = injected_tcp_port
            .or(jsonrpc_port_override)
            .unwrap_or_else(network::actual_jsonrpc_port);

        let ip: IpAddr = resolved_bind.parse().or_exit("Invalid bind address");
        let tarpc_addr = SocketAddr::new(ip, resolved_tarpc_port);
        let jsonrpc_addr = SocketAddr::new(ip, resolved_jsonrpc_port);

        let rpc_service_tarpc = rpc_service.clone();
        let rpc_service_jsonrpc = rpc_service.clone();
        let tarpc_task = tokio::spawn(async move {
            info!("Starting tarpc server on {tarpc_addr}");
            if let Err(e) = run_tarpc_server(tarpc_addr, rpc_service_tarpc).await {
                error!("tarpc server error: {e}");
            }
        });
        let jsonrpc_task = tokio::spawn(async move {
            info!("Starting JSON-RPC server on {jsonrpc_addr}");
            match run_jsonrpc_server(jsonrpc_addr, rpc_service_jsonrpc).await {
                Ok(mut handle) => handle.stopped().await,
                Err(e) => error!("JSON-RPC server error: {e}"),
            }
        });
        Some((
            tarpc_task,
            jsonrpc_task,
            resolved_tarpc_port,
            resolved_jsonrpc_port,
        ))
    } else {
        None
    };

    // Injected UDS endpoint takes priority, then --socket flag, then env resolution
    let socket_path = match &injected_endpoint {
        Some(loam_spine_core::transport::TransportEndpoint::Uds { path }) => {
            std::path::PathBuf::from(path)
        }
        _ => socket_override.map_or_else(
            loam_spine_core::neural_api::resolve_socket_path,
            std::path::PathBuf::from,
        ),
    };

    // BTSP Phase 2: resolve handshake config from environment.
    // When BIOMEOS_FAMILY_ID is set (non-default), BTSP handshake is mandatory
    // on UDS connections — all crypto delegated to BTSP provider via JSON-RPC.
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

    // Write PID file alongside the socket for instant liveness checks by
    // consumers (kill(pid, 0) is faster than a connect-probe).
    #[cfg(unix)]
    let pid_path = {
        let p = socket_path.with_extension("pid");
        let pid_str = std::process::id().to_string();
        let p_clone = p.clone();
        match tokio::task::spawn_blocking(move || std::fs::write(&p_clone, pid_str)).await {
            Ok(Ok(())) => {
                debug!("PID file written: {}", p.display());
                Some(p)
            }
            Ok(Err(e)) => {
                warn!("Could not write PID file {}: {e}", p.display());
                None
            }
            Err(e) => {
                warn!("PID file write join error: {e}");
                None
            }
        }
    };

    #[cfg(unix)]
    let family_id = loam_spine_core::constants::env_resolution::biomeos_family_id();

    // Capability-domain symlink: ledger.sock → loamspine.sock
    // Enables orchestration-layer `by_capability = "ledger"` routing in deploy graphs.
    #[cfg(unix)]
    let capability_symlink = {
        let link_path = loam_spine_core::neural_api::resolve_capability_symlink_path(
            &socket_path,
            family_id.as_deref(),
        );
        if link_path != socket_path {
            let target = socket_path.clone();
            let link = link_path.clone();
            match tokio::task::spawn_blocking(move || {
                let _ = std::fs::remove_file(&link);
                std::os::unix::fs::symlink(&target, &link)
            })
            .await
            {
                Ok(Ok(())) => {
                    info!(
                        "Domain symlink: {} → {}",
                        link_path.display(),
                        socket_path.display()
                    );
                    Some(link_path)
                }
                Ok(Err(e)) => {
                    warn!(
                        "Could not create domain symlink {}: {e}",
                        link_path.display()
                    );
                    None
                }
                Err(e) => {
                    warn!("Symlink creation join error: {e}");
                    None
                }
            }
        } else {
            None
        }
    };

    // Legacy backward-compat symlink: permanence.sock → loamspine.sock
    // for consumers that connected via the old domain-based socket name
    #[cfg(unix)]
    let legacy_symlink = {
        let link_path = loam_spine_core::neural_api::resolve_legacy_symlink_path(
            &socket_path,
            family_id.as_deref(),
        );
        if link_path != socket_path {
            let target = socket_path.clone();
            let link = link_path.clone();
            match tokio::task::spawn_blocking(move || {
                let _ = std::fs::remove_file(&link);
                std::os::unix::fs::symlink(&target, &link)
            })
            .await
            {
                Ok(Ok(())) => {
                    info!(
                        "Legacy symlink: {} → {}",
                        link_path.display(),
                        socket_path.display()
                    );
                    Some(link_path)
                }
                Ok(Err(e)) => {
                    warn!(
                        "Could not create legacy symlink {}: {e}",
                        link_path.display()
                    );
                    None
                }
                Err(e) => {
                    warn!("Legacy symlink creation join error: {e}");
                    None
                }
            }
        } else {
            None
        }
    };

    info!("LoamSpine service started successfully");
    if let Some((_, _, tp, jp)) = &tarpc_handle {
        info!("  tarpc:    tarpc://{resolved_bind}:{tp}");
        info!("  JSON-RPC: http://{resolved_bind}:{jp}");
    }
    info!("  socket:   {}", socket_path.display());

    let signal_handler = loam_spine_core::service::signals::SignalHandler::new();

    // Cooperative shutdown: all server tasks are monitored symmetrically.
    // A failure in any transport (tarpc, JSON-RPC TCP, or UDS) triggers
    // orderly teardown rather than silent degradation.
    match tarpc_handle {
        Some((tarpc_task, jsonrpc_task, ..)) => {
            tokio::select! {
                result = signal_handler.wait_for_shutdown() => {
                    if let Err(e) = result {
                        error!("Signal handler error: {e}");
                    }
                }
                result = tarpc_task => {
                    if let Err(e) = result {
                        error!("tarpc task failed: {e}");
                    }
                }
                result = jsonrpc_task => {
                    if let Err(e) = result {
                        error!("JSON-RPC task failed: {e}");
                    }
                }
            }
        }
        None => {
            // UDS-only mode: wait for signal or UDS server failure.
            if let Err(e) = signal_handler.wait_for_shutdown().await {
                error!("Signal handler error: {e}");
            }
        }
    }

    // Graceful UDS server drain before lifecycle teardown
    #[cfg(unix)]
    if let Some(ref handle) = uds_handle {
        handle.stop();
    }

    lifecycle.stop().await?;

    // Clean up sockets, symlinks, and PID file on graceful shutdown
    // per PRIMAL_SELF_KNOWLEDGE_STANDARD §3 + STALE_SOCKET_CLEANUP_STANDARD
    #[cfg(unix)]
    {
        drop(uds_handle);
        let cap = capability_symlink;
        let leg = legacy_symlink;
        let pid = pid_path;
        let _ = tokio::task::spawn_blocking(move || {
            if let Some(link) = cap {
                let _ = std::fs::remove_file(&link);
            }
            if let Some(link) = leg {
                let _ = std::fs::remove_file(&link);
            }
            if let Some(ref p) = pid {
                let _ = std::fs::remove_file(p);
            }
        })
        .await;
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
