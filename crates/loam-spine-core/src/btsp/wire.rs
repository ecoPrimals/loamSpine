// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP wire types per `BTSP_PROTOCOL_STANDARD.md`.
//!
//! These are the JSON-serialized messages exchanged between client and server
//! during the BTSP handshake. All crypto is delegated to the BTSP provider — these types
//! carry opaque key material, not raw secrets.

use serde::{Deserialize, Serialize};

/// `ClientHello` — first message from connecting client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHello {
    /// Protocol version (must be 1).
    pub version: u32,
    /// Client's ephemeral X25519 public key (base64).
    pub client_ephemeral_pub: String,
}

/// `ServerHello` — server's response with challenge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHello {
    /// Protocol version.
    pub version: u32,
    /// Server's ephemeral X25519 public key (base64).
    pub server_ephemeral_pub: String,
    /// Random 32-byte challenge (hex).
    pub challenge: String,
}

/// `ChallengeResponse` — client proves family membership.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    /// HMAC-SHA256 response (base64).
    pub response: String,
    /// Client's preferred cipher suite.
    pub preferred_cipher: String,
}

/// `HandshakeComplete` — server confirms authentication and cipher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeComplete {
    /// Negotiated cipher suite.
    pub cipher: String,
    /// Session identifier (hex, 16 bytes).
    pub session_id: String,
}

/// `HandshakeError` — server rejects the handshake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeError {
    /// Error category.
    pub error: String,
    /// Human-readable reason.
    pub reason: String,
}

/// BTSP provider `btsp.session.create` response per `beardog_types::btsp::rpc`.
#[derive(Debug, Deserialize)]
pub(crate) struct SessionCreateResult {
    /// Opaque session token referencing BearDog's server-side state.
    pub session_token: String,
    /// Server's ephemeral X25519 public key (base64).
    pub server_ephemeral_pub: String,
    /// BearDog-generated challenge (base64, 32 bytes).
    pub challenge: String,
}

/// BTSP provider `btsp.session.verify` response per `beardog_types::btsp::rpc`.
#[derive(Debug, Deserialize)]
pub(crate) struct SessionVerifyResult {
    pub verified: bool,
    /// Session ID assigned by BearDog on successful verification.
    pub session_id: Option<String>,
    /// Negotiated cipher (set on success).
    #[expect(dead_code, reason = "reserved for Phase 3 encrypted framing")]
    pub cipher: Option<String>,
}

/// BTSP provider `btsp.negotiate` response per `beardog_types::btsp::rpc`.
#[derive(Debug, Deserialize)]
pub(crate) struct NegotiateResult {
    pub cipher: String,
    pub accepted: bool,
}

/// An authenticated BTSP session.
#[derive(Debug, Clone)]
pub struct BtspSession {
    /// Unique session identifier (hex).
    pub session_id: String,
    /// Negotiated cipher suite (e.g. `"null"`, `"chacha20_poly1305"`).
    pub cipher: String,
}

// --- NDJSON wire types (primalSpring-compatible) ---
//
// primalSpring sends newline-delimited JSON with a `protocol` discriminator
// field. These types match the wire format from
// `primalSpring/ecoPrimal/src/ipc/btsp_handshake.rs`.

/// NDJSON `ClientHello` — first line from a primalSpring-style BTSP client.
///
/// Distinguished from JSON-RPC by the `protocol: "btsp"` field (no `jsonrpc`
/// or `method` fields present).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdjsonClientHello {
    /// Protocol discriminator — always `"btsp"`.
    pub protocol: String,
    /// BTSP version (primalSpring sends `1` as u8).
    pub version: u32,
    /// Client's ephemeral public key (base64, 32 bytes).
    pub client_ephemeral_pub: String,
}

/// NDJSON `ServerHello` — response to primalSpring-style BTSP client.
///
/// Includes `session_id` which the length-prefixed `ServerHello` omits
/// (primalSpring expects it for logging and session tracking).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdjsonServerHello {
    /// BTSP version.
    pub version: u32,
    /// Server's ephemeral public key (base64).
    pub server_ephemeral_pub: String,
    /// Challenge (base64-encoded random bytes).
    pub challenge: String,
    /// Session identifier.
    pub session_id: String,
}
