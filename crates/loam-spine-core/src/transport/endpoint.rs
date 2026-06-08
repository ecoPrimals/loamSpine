// SPDX-License-Identifier: AGPL-3.0-or-later

//! Transport endpoint abstraction for launcher-injected transport.
//!
//! Wire-compatible with `sourdough_core::TransportEndpoint` (same serde
//! tagged JSON format). Primals accept a `TRANSPORT_ENDPOINT` env var and
//! let the launcher or Songbird decide the transport — not the primal.
//!
//! # Wire Format
//!
//! ```json
//! { "transport": "uds", "path": "/run/membrane/loamspine.sock" }
//! { "transport": "tcp", "host": "127.0.0.1", "port": 8080 }
//! { "transport": "mesh_relay", "peer_id": "strand-gate", "capability": "ledger" }
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// Structured transport endpoint — describes how to reach a service.
///
/// Wire-compatible with `sourdough_core::TransportEndpoint` and
/// `songbird_types::TransportEndpoint`. No cross-primal dependency —
/// the wire format is the contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "transport")]
pub enum TransportEndpoint {
    /// Unix Domain Socket — local primal on same host.
    #[serde(rename = "uds")]
    Uds {
        /// Filesystem path to the socket.
        path: String,
    },

    /// TCP — direct network connection.
    #[serde(rename = "tcp")]
    Tcp {
        /// Host address (IPv4, IPv6, or hostname).
        host: String,
        /// TCP port number.
        port: u16,
    },

    /// Mesh relay — reachable via Songbird's mesh network.
    #[serde(rename = "mesh_relay")]
    MeshRelay {
        /// Mesh peer identifier (e.g. `"strand-gate"`).
        peer_id: String,
        /// Capability being resolved on the remote peer.
        capability: String,
    },
}

impl TransportEndpoint {
    /// Construct a UDS endpoint.
    #[must_use]
    pub fn uds(path: impl Into<String>) -> Self {
        Self::Uds { path: path.into() }
    }

    /// Construct a TCP endpoint.
    #[must_use]
    pub fn tcp(host: impl Into<String>, port: u16) -> Self {
        Self::Tcp {
            host: host.into(),
            port,
        }
    }

    /// Construct a mesh relay endpoint.
    #[must_use]
    pub fn mesh_relay(peer_id: impl Into<String>, capability: impl Into<String>) -> Self {
        Self::MeshRelay {
            peer_id: peer_id.into(),
            capability: capability.into(),
        }
    }

    /// Whether this endpoint is local (same host, no network hop).
    #[must_use]
    pub fn is_local(&self) -> bool {
        match self {
            Self::Uds { .. } => true,
            Self::Tcp { host, .. } => host == "127.0.0.1" || host == "::1" || host == "localhost",
            Self::MeshRelay { .. } => false,
        }
    }

    /// Transport name as it appears in the wire format.
    #[must_use]
    pub const fn transport_name(&self) -> &'static str {
        match self {
            Self::Uds { .. } => "uds",
            Self::Tcp { .. } => "tcp",
            Self::MeshRelay { .. } => "mesh_relay",
        }
    }
}

impl fmt::Display for TransportEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uds { path } => write!(f, "unix://{path}"),
            Self::Tcp { host, port } => {
                if host.contains(':') {
                    write!(f, "tcp://[{host}]:{port}")
                } else {
                    write!(f, "tcp://{host}:{port}")
                }
            }
            Self::MeshRelay {
                peer_id,
                capability,
            } => write!(f, "mesh://{peer_id}/{capability}"),
        }
    }
}

/// Parse a `TRANSPORT_ENDPOINT` env var value into a [`TransportEndpoint`].
///
/// The value must be a JSON string matching the tagged serde format.
///
/// # Errors
///
/// Returns an error if the value is not valid JSON or doesn't match the
/// expected wire format.
pub fn parse_transport_endpoint(json: &str) -> Result<TransportEndpoint, serde_json::Error> {
    serde_json::from_str(json)
}

/// The env var name for transport endpoint injection.
pub const TRANSPORT_ENDPOINT_ENV: &str = "TRANSPORT_ENDPOINT";

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests {
    use super::*;

    #[test]
    fn uds_roundtrip() {
        let ep = TransportEndpoint::uds("/run/membrane/loamspine.sock");
        let json = serde_json::to_string(&ep).unwrap();
        assert!(json.contains(r#""transport":"uds"#));
        assert!(json.contains("loamspine.sock"));
        let parsed: TransportEndpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(ep, parsed);
    }

    #[test]
    fn tcp_roundtrip() {
        let ep = TransportEndpoint::tcp("192.168.1.132", 8080);
        let json = serde_json::to_string(&ep).unwrap();
        assert!(json.contains(r#""transport":"tcp"#));
        let parsed: TransportEndpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(ep, parsed);
    }

    #[test]
    fn mesh_relay_roundtrip() {
        let ep = TransportEndpoint::mesh_relay("strand-gate", "ledger");
        let json = serde_json::to_string(&ep).unwrap();
        assert!(json.contains(r#""transport":"mesh_relay"#));
        let parsed: TransportEndpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(ep, parsed);
    }

    #[test]
    fn wire_compat_sourdough_uds() {
        let json = r#"{"transport":"uds","path":"/run/user/1000/biomeos/beardog.sock"}"#;
        let ep: TransportEndpoint = serde_json::from_str(json).unwrap();
        assert_eq!(
            ep,
            TransportEndpoint::uds("/run/user/1000/biomeos/beardog.sock")
        );
    }

    #[test]
    fn wire_compat_sourdough_tcp() {
        let json = r#"{"transport":"tcp","host":"127.0.0.1","port":9100}"#;
        let ep: TransportEndpoint = serde_json::from_str(json).unwrap();
        assert_eq!(ep, TransportEndpoint::tcp("127.0.0.1", 9100));
    }

    #[test]
    fn wire_compat_sourdough_mesh() {
        let json = r#"{"transport":"mesh_relay","peer_id":"strand-gate","capability":"security"}"#;
        let ep: TransportEndpoint = serde_json::from_str(json).unwrap();
        assert_eq!(
            ep,
            TransportEndpoint::mesh_relay("strand-gate", "security")
        );
    }

    #[test]
    fn is_local_uds() {
        assert!(TransportEndpoint::uds("/tmp/test.sock").is_local());
    }

    #[test]
    fn is_local_tcp_localhost() {
        assert!(TransportEndpoint::tcp("127.0.0.1", 8080).is_local());
        assert!(TransportEndpoint::tcp("::1", 8080).is_local());
        assert!(TransportEndpoint::tcp("localhost", 8080).is_local());
    }

    #[test]
    fn is_not_local_tcp_remote() {
        assert!(!TransportEndpoint::tcp("192.168.1.100", 8080).is_local());
    }

    #[test]
    fn is_not_local_mesh_relay() {
        assert!(!TransportEndpoint::mesh_relay("peer", "cap").is_local());
    }

    #[test]
    fn display_format() {
        assert_eq!(
            TransportEndpoint::uds("/tmp/test.sock").to_string(),
            "unix:///tmp/test.sock"
        );
        assert_eq!(
            TransportEndpoint::tcp("127.0.0.1", 9100).to_string(),
            "tcp://127.0.0.1:9100"
        );
        assert_eq!(
            TransportEndpoint::tcp("::1", 9100).to_string(),
            "tcp://[::1]:9100"
        );
        assert_eq!(
            TransportEndpoint::mesh_relay("gate", "ledger").to_string(),
            "mesh://gate/ledger"
        );
    }

    #[test]
    fn parse_env_var_value() {
        let json = r#"{"transport":"uds","path":"/run/membrane/loamspine.sock"}"#;
        let ep = parse_transport_endpoint(json).unwrap();
        assert_eq!(
            ep,
            TransportEndpoint::uds("/run/membrane/loamspine.sock")
        );
    }

    #[test]
    fn parse_invalid_json_fails() {
        assert!(parse_transport_endpoint("not json").is_err());
    }

    #[test]
    fn transport_name_values() {
        assert_eq!(TransportEndpoint::uds("/tmp/x").transport_name(), "uds");
        assert_eq!(TransportEndpoint::tcp("h", 1).transport_name(), "tcp");
        assert_eq!(
            TransportEndpoint::mesh_relay("p", "c").transport_name(),
            "mesh_relay"
        );
    }
}
