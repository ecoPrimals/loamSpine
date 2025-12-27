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

/// Default tarpc port for primal-to-primal communication.
///
/// This is a sensible default for development, but production deployments should:
/// - Set port to `0` for OS assignment, or
/// - Configure via `TARPC_PORT` environment variable, or
/// - Use service discovery to find available endpoints
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
/// This is a sensible default for development, but production deployments should:
/// - Set port to `0` for OS assignment, or
/// - Configure via `JSONRPC_PORT` environment variable, or
/// - Use service discovery to find available endpoints
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
/// Production deployments should **never** rely on this fallback.
/// Always configure discovery explicitly via environment variables or service discovery.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::assertions_on_constants)] // Validating const values is appropriate in tests
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
    }

    #[test]
    fn ports_dont_conflict() {
        // All default ports should be different
        assert_ne!(DEFAULT_TARPC_PORT, DEFAULT_JSONRPC_PORT);
        assert_ne!(DEFAULT_TARPC_PORT, DEFAULT_DISCOVERY_PORT);
        assert_ne!(DEFAULT_JSONRPC_PORT, DEFAULT_DISCOVERY_PORT);
    }
}

