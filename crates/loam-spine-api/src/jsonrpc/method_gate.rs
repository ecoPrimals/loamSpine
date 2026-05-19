// SPDX-License-Identifier: AGPL-3.0-or-later

//! JH-0 Method Gate — pre-dispatch access control for JSON-RPC methods.
//!
//! Classifies every canonical method as **Public** or **Protected** and
//! enforces access based on the configured [`AuthMode`].
//!
//! # Error codes
//!
//! | Code     | Meaning |
//! |----------|---------|
//! | `-32001` | Unauthorized — method requires authentication |
//! | `-32000` | Auth configuration / internal auth error |
//!
//! # Auth modes
//!
//! | Mode         | Public | Protected |
//! |--------------|--------|-----------|
//! | `permissive` | allow  | allow     |
//! | `enforced`   | allow  | reject    |

use std::sync::Arc;

/// JSON-RPC error code: unauthorized (method requires authentication).
pub const AUTH_UNAUTHORIZED: i32 = -32001;

/// JSON-RPC error code: auth configuration / internal auth error.
///
/// Reserved for future use when auth verification returns errors
/// (e.g. malformed tokens, expired credentials).
#[expect(dead_code, reason = "JH-0 standard error code, reserved for auth verification")]
pub const AUTH_ERROR: i32 = -32000;

/// Access classification for a JSON-RPC method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodAccess {
    /// Callable without authentication (health, identity, capabilities, auth).
    Public,
    /// Requires authentication in `enforced` mode.
    Protected,
}

/// Authentication mode governing protected-method access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMode {
    /// All methods allowed regardless of authentication (default).
    Permissive,
    /// Protected methods rejected with `-32001` unless authenticated.
    Enforced,
}

impl AuthMode {
    /// Parse from the `LOAMSPINE_AUTH_MODE` environment variable.
    ///
    /// Recognized values (case-insensitive): `permissive`, `enforced`.
    /// Unset or unrecognized defaults to `Permissive`.
    #[must_use]
    pub fn from_env() -> Self {
        std::env::var("LOAMSPINE_AUTH_MODE")
            .ok()
            .and_then(|v| match v.to_ascii_lowercase().as_str() {
                "enforced" => Some(Self::Enforced),
                "permissive" => Some(Self::Permissive),
                _ => None,
            })
            .unwrap_or(Self::Permissive)
    }

    /// String representation for wire responses.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Permissive => "permissive",
            Self::Enforced => "enforced",
        }
    }
}

/// Classify a canonical method name as Public or Protected.
///
/// Public methods (always allowed):
/// - `health.*`            — liveness / readiness / health check
/// - `auth.*`              — auth introspection
/// - `identity.get`        — primal identity discovery
/// - `capabilities.list`   — capability discovery
/// - `lifecycle.status`    — service lifecycle status
/// - `primal.announce`     — self-registration
/// - `btsp.capabilities`   — BTSP cipher/feature discovery
/// - `tools.list`          — MCP tool discovery
///
/// Everything else is Protected.
#[must_use]
pub fn classify_method(method: &str) -> MethodAccess {
    if method.starts_with("health.") || method.starts_with("auth.") {
        return MethodAccess::Public;
    }
    match method {
        "identity.get"
        | "capabilities.list"
        | "lifecycle.status"
        | "primal.announce"
        | "btsp.capabilities"
        | "tools.list" => MethodAccess::Public,
        _ => MethodAccess::Protected,
    }
}

/// Pre-dispatch method gate.
///
/// Wraps an [`AuthMode`] and provides the gate check called before dispatch.
/// Thread-safe via `Arc` sharing across connections.
#[derive(Debug, Clone)]
pub struct MethodGate {
    mode: Arc<std::sync::atomic::AtomicU8>,
}

// Pack AuthMode into u8 for AtomicU8
const MODE_PERMISSIVE: u8 = 0;
const MODE_ENFORCED: u8 = 1;

impl MethodGate {
    /// Create a new gate with the given initial mode.
    #[must_use]
    pub fn new(mode: AuthMode) -> Self {
        let val = match mode {
            AuthMode::Permissive => MODE_PERMISSIVE,
            AuthMode::Enforced => MODE_ENFORCED,
        };
        Self {
            mode: Arc::new(std::sync::atomic::AtomicU8::new(val)),
        }
    }

    /// Create from the `LOAMSPINE_AUTH_MODE` environment variable.
    #[must_use]
    pub fn from_env() -> Self {
        Self::new(AuthMode::from_env())
    }

    /// Current auth mode.
    #[must_use]
    pub fn current_mode(&self) -> AuthMode {
        match self
            .mode
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            MODE_ENFORCED => AuthMode::Enforced,
            _ => AuthMode::Permissive,
        }
    }

    /// Check whether a method is allowed under the current mode.
    ///
    /// Returns `Ok(())` if allowed, or `Err` with an appropriate
    /// JSON-RPC error if the method is blocked.
    pub fn check(&self, method: &str) -> Result<(), super::wire::JsonRpcError> {
        let access = classify_method(method);
        match (self.current_mode(), access) {
            (_, MethodAccess::Public) | (AuthMode::Permissive, _) => Ok(()),
            (AuthMode::Enforced, MethodAccess::Protected) => {
                Err(super::wire::JsonRpcError {
                    code: AUTH_UNAUTHORIZED,
                    message: format!(
                        "method \"{method}\" requires authentication (auth mode: enforced)"
                    ),
                    data: None,
                })
            }
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests {
    use super::*;

    #[test]
    fn classify_public_methods() {
        assert_eq!(classify_method("health.check"), MethodAccess::Public);
        assert_eq!(classify_method("health.liveness"), MethodAccess::Public);
        assert_eq!(classify_method("health.readiness"), MethodAccess::Public);
        assert_eq!(classify_method("identity.get"), MethodAccess::Public);
        assert_eq!(classify_method("capabilities.list"), MethodAccess::Public);
        assert_eq!(classify_method("tools.list"), MethodAccess::Public);
        assert_eq!(classify_method("auth.check"), MethodAccess::Public);
        assert_eq!(classify_method("auth.mode"), MethodAccess::Public);
        assert_eq!(classify_method("auth.peer_info"), MethodAccess::Public);
    }

    #[test]
    fn classify_protected_methods() {
        assert_eq!(classify_method("spine.create"), MethodAccess::Protected);
        assert_eq!(classify_method("spine.get"), MethodAccess::Protected);
        assert_eq!(classify_method("spine.list"), MethodAccess::Protected);
        assert_eq!(classify_method("entry.append"), MethodAccess::Protected);
        assert_eq!(classify_method("entry.list"), MethodAccess::Protected);
        assert_eq!(classify_method("session.commit"), MethodAccess::Protected);
        assert_eq!(classify_method("certificate.mint"), MethodAccess::Protected);
        assert_eq!(classify_method("braid.commit"), MethodAccess::Protected);
        assert_eq!(classify_method("btsp.negotiate"), MethodAccess::Protected);
        assert_eq!(
            classify_method("bonding.ledger.store"),
            MethodAccess::Protected
        );
        assert_eq!(
            classify_method("permanence.commit_session"),
            MethodAccess::Protected
        );
        assert_eq!(classify_method("tools.call"), MethodAccess::Protected);
    }

    #[test]
    fn gate_permissive_allows_all() {
        let gate = MethodGate::new(AuthMode::Permissive);
        assert!(gate.check("spine.create").is_ok());
        assert!(gate.check("entry.append").is_ok());
        assert!(gate.check("health.check").is_ok());
        assert!(gate.check("auth.mode").is_ok());
    }

    #[test]
    fn gate_enforced_allows_public() {
        let gate = MethodGate::new(AuthMode::Enforced);
        assert!(gate.check("health.check").is_ok());
        assert!(gate.check("health.liveness").is_ok());
        assert!(gate.check("identity.get").is_ok());
        assert!(gate.check("capabilities.list").is_ok());
        assert!(gate.check("auth.check").is_ok());
        assert!(gate.check("auth.mode").is_ok());
    }

    #[test]
    fn gate_enforced_blocks_protected() {
        let gate = MethodGate::new(AuthMode::Enforced);
        let err = gate.check("spine.create").unwrap_err();
        assert_eq!(err.code, AUTH_UNAUTHORIZED);
        assert!(err.message.contains("requires authentication"));

        let err = gate.check("entry.append").unwrap_err();
        assert_eq!(err.code, AUTH_UNAUTHORIZED);

        let err = gate.check("session.commit").unwrap_err();
        assert_eq!(err.code, AUTH_UNAUTHORIZED);
    }

    #[test]
    fn auth_mode_as_str() {
        assert_eq!(AuthMode::Permissive.as_str(), "permissive");
        assert_eq!(AuthMode::Enforced.as_str(), "enforced");
    }

    #[test]
    fn auth_mode_roundtrip_str() {
        assert_eq!(AuthMode::Permissive.as_str(), "permissive");
        assert_eq!(AuthMode::Enforced.as_str(), "enforced");
        let gate_p = MethodGate::new(AuthMode::Permissive);
        assert_eq!(gate_p.current_mode(), AuthMode::Permissive);
        let gate_e = MethodGate::new(AuthMode::Enforced);
        assert_eq!(gate_e.current_mode(), AuthMode::Enforced);
    }
}
