// SPDX-License-Identifier: AGPL-3.0-or-later

//! Well-known constants for LoamSpine.
//!
//! These provide sensible defaults but can always be overridden via
//! environment variables or configuration files.
//!
//! ## Port Assignment Philosophy
//!
//! In production, consider:
//! - Setting ports to `0` for OS assignment (maximizes flexibility)
//! - Using environment variables for explicit configuration
//! - Leveraging service discovery instead of hardcoded ports

pub mod env_resolution;
pub mod network;

/// Default tarpc port for primal-to-primal communication.
///
/// **Development default only** - Never hardcode this in production logic!
///
/// Production deployments should:
/// - Set port to `0` for OS assignment (recommended), or
/// - Configure via `TARPC_PORT` or `LOAMSPINE_TARPC_PORT` environment variable, or
/// - Use service discovery to find available endpoints
///
/// This constant exists solely for development convenience, similar to how
/// HTTP uses port 80 or SSH uses port 22 as conventional defaults.
///
/// # Example
///
/// ```rust
/// use loam_spine_core::constants::DEFAULT_TARPC_PORT;
///
/// let port = std::env::var("TARPC_PORT")
///     .ok()
///     .and_then(|s| s.parse().ok())
///     .unwrap_or(DEFAULT_TARPC_PORT);
/// ```
pub const DEFAULT_TARPC_PORT: u16 = 9001;

/// Default JSON-RPC port for external clients.
///
/// **Development default only** - Never hardcode this in production logic!
///
/// Production deployments should:
/// - Set port to `0` for OS assignment (recommended), or
/// - Configure via `JSONRPC_PORT` or `LOAMSPINE_JSONRPC_PORT` environment variable, or
/// - Use service discovery to find available endpoints
///
/// This constant exists solely for development convenience, similar to how
/// HTTP uses port 80 or HTTPS uses port 443 as conventional defaults.
///
/// # Example
///
/// ```rust
/// use loam_spine_core::constants::DEFAULT_JSONRPC_PORT;
///
/// let port = std::env::var("JSONRPC_PORT")
///     .ok()
///     .and_then(|s| s.parse().ok())
///     .unwrap_or(DEFAULT_JSONRPC_PORT);
/// ```
pub const DEFAULT_JSONRPC_PORT: u16 = 8080;

/// Default discovery service port (fallback only).
///
/// **WARNING**: This is only used as a last-resort development fallback when:
/// - No `DISCOVERY_ENDPOINT` environment variable is set
/// - DNS SRV lookup fails
/// - mDNS discovery fails
///
/// **Production deployments should NEVER rely on this fallback.**
/// Always configure discovery explicitly via:
/// - Environment variables (`DISCOVERY_ENDPOINT`)
/// - DNS SRV records (`_discovery._tcp.local`)
/// - Service registry configuration
///
/// This constant only exists for local development convenience. Using it in
/// production will generate warning logs.
///
/// # Example
///
/// ```rust
/// use loam_spine_core::constants::DEFAULT_DISCOVERY_PORT;
///
/// // This logs a warning if used!
/// tracing::warn!(
///     "Using development fallback port: {}. Configure DISCOVERY_ENDPOINT in production!",
///     DEFAULT_DISCOVERY_PORT
/// );
/// ```
pub const DEFAULT_DISCOVERY_PORT: u16 = 8082;

/// OS-assigned port (let kernel choose available port).
///
/// **Recommended** for production to maximize portability and avoid conflicts.
///
/// # Example
///
/// ```rust
/// use loam_spine_core::constants::OS_ASSIGNED_PORT;
///
/// // Production configuration
/// let tarpc_endpoint = format!("0.0.0.0:{}", OS_ASSIGNED_PORT);
/// // OS will assign an available port automatically
/// ```
pub const OS_ASSIGNED_PORT: u16 = 0;

/// Default bind address for all interfaces (IPv4).
///
/// Used for server endpoints that should accept connections from any network interface.
/// In containerized environments, this allows external access to services.
///
/// # Example
///
/// ```rust
/// use loam_spine_core::constants::{BIND_ALL_IPV4, DEFAULT_TARPC_PORT};
///
/// let endpoint = format!("http://{}:{}", BIND_ALL_IPV4, DEFAULT_TARPC_PORT);
/// // Results in "http://0.0.0.0:9001"
/// ```
pub const BIND_ALL_IPV4: &str = "0.0.0.0";

/// Localhost address (IPv4).
///
/// Used for local development and testing. Connections are restricted to the local machine.
///
/// # Example
///
/// ```rust
/// use loam_spine_core::constants::{LOCALHOST, DEFAULT_DISCOVERY_PORT};
///
/// let endpoint = format!("http://{}:{}", LOCALHOST, DEFAULT_DISCOVERY_PORT);
/// // Results in "http://localhost:8082" or "http://127.0.0.1:8082"
/// ```
pub const LOCALHOST: &str = "localhost";

/// Localhost IP address (IPv4).
///
/// Numeric form of localhost, useful when DNS resolution should be avoided.
pub const LOCALHOST_IP: &str = "127.0.0.1";

/// Standard HTTPS port.
///
/// Used in discovery endpoint construction: when a service reports port 443,
/// the port is omitted from the URL since HTTPS uses it by default.
pub const HTTPS_DEFAULT_PORT: u16 = 443;

/// Protocol identifiers for service advertisement and discovery.
pub mod protocol {
    /// tarpc (Rust-to-Rust) transport protocol identifier.
    pub const TARPC: &str = "tarpc";
    /// JSON-RPC transport protocol identifier.
    pub const JSONRPC: &str = "jsonrpc";
    /// Standard health-check endpoint path.
    pub const HEALTH_PATH: &str = "/health";
}

/// Service registry HTTP endpoint paths.
///
/// Used by `DiscoveryClient` to interact with HTTP-based registries
/// (Songbird, Consul adapter, etcd adapter, etc.).
pub mod registry {
    /// Capability discovery endpoint.
    pub const DISCOVER_PATH: &str = "/discover";
    /// Service registration endpoint.
    pub const REGISTER_PATH: &str = "/register";
    /// Heartbeat / keep-alive endpoint.
    pub const HEARTBEAT_PATH: &str = "/heartbeat";
    /// Service deregistration endpoint.
    pub const DEREGISTER_PATH: &str = "/deregister";
}

/// Metadata keys and values for service advertisement.
pub mod metadata {
    /// Implementation language.
    pub const LANGUAGE: &str = "rust";
    /// RPC style used by this primal.
    pub const RPC_STYLE: &str = "pure-rust";
    /// Storage backend.
    pub const STORAGE_BACKEND: &str = "redb";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[expect(
        clippy::assertions_on_constants,
        reason = "validating const values is the test's purpose"
    )]
    fn constants_are_reasonable() {
        // Tarpc port should be in user range
        assert!(DEFAULT_TARPC_PORT >= 1024);
        assert!(DEFAULT_TARPC_PORT < 65535);

        // JSON-RPC port should be in user range
        assert!(DEFAULT_JSONRPC_PORT >= 1024);
        assert!(DEFAULT_JSONRPC_PORT < 65535);

        // Discovery port should be in user range
        assert!(DEFAULT_DISCOVERY_PORT >= 1024);
        assert!(DEFAULT_DISCOVERY_PORT < 65535);

        // OS assigned is always 0
        assert_eq!(OS_ASSIGNED_PORT, 0);

        // Addresses have expected lengths
        assert!(BIND_ALL_IPV4.len() > 6); // "0.0.0.0"
        assert!(LOCALHOST.len() > 5); // "localhost"
        assert!(LOCALHOST_IP.len() > 6); // "127.0.0.1"
    }

    #[test]
    fn ports_dont_conflict() {
        // All default ports should be different
        assert_ne!(DEFAULT_TARPC_PORT, DEFAULT_JSONRPC_PORT);
        assert_ne!(DEFAULT_TARPC_PORT, DEFAULT_DISCOVERY_PORT);
        assert_ne!(DEFAULT_JSONRPC_PORT, DEFAULT_DISCOVERY_PORT);
    }

    #[test]
    fn localhost_forms_are_valid() {
        // Both forms should be valid for use in URLs
        assert!(LOCALHOST.chars().all(|c| c.is_alphanumeric() || c == '.'));
        assert!(LOCALHOST_IP.chars().all(|c| c.is_numeric() || c == '.'));

        // Bind all should be valid
        assert!(BIND_ALL_IPV4.chars().all(|c| c.is_numeric() || c == '.'));
    }

    #[test]
    fn registry_paths_start_with_slash() {
        assert!(registry::DISCOVER_PATH.starts_with('/'));
        assert!(registry::REGISTER_PATH.starts_with('/'));
        assert!(registry::HEARTBEAT_PATH.starts_with('/'));
        assert!(registry::DEREGISTER_PATH.starts_with('/'));
        assert!(protocol::HEALTH_PATH.starts_with('/'));
    }

    #[test]
    fn registry_paths_are_distinct() {
        let paths = [
            registry::DISCOVER_PATH,
            registry::REGISTER_PATH,
            registry::HEARTBEAT_PATH,
            registry::DEREGISTER_PATH,
            protocol::HEALTH_PATH,
        ];
        for (i, a) in paths.iter().enumerate() {
            for b in &paths[i + 1..] {
                assert_ne!(a, b, "registry paths must be distinct");
            }
        }
    }
}
