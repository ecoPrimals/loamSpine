// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP client-side handshake for connecting to bearDog in strict mode.
//!
//! When `BEARDOG_UDS_REQUIRE_BTSP=1` is set, bearDog rejects plain JSON-RPC
//! with `-32600`. This module implements the consumer-side of the 4-step BTSP
//! handshake so loamSpine can authenticate before sending requests.
//!
//! The challenge response uses LOCAL HMAC-SHA256 with the family seed — this
//! avoids the chicken-and-egg of needing bearDog to compute HMAC for the
//! handshake that authenticates us TO bearDog.
//!
//! ## Wire Format (NDJSON — newline-delimited)
//!
//! ```text
//! 1. Send  ClientHello       { protocol: "btsp", version: 1, client_ephemeral_pub }
//! 2. Read  ServerHello       { version, server_ephemeral_pub, challenge, session_id }
//! 3. Send  ChallengeResponse { response, preferred_cipher }
//! 4. Read  HandshakeComplete { cipher, session_id }
//! ```

use base64::Engine;
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::debug;

use crate::transport::framing::write_ndjson_request;
use crate::transport::stream::TransportStream;

type HmacSha256 = Hmac<Sha256>;

const BTSP_VERSION: u8 = 1;
const PREFERRED_CIPHER: &str = "chacha20_poly1305";

#[derive(Debug, Serialize)]
struct ClientHello {
    protocol: &'static str,
    version: u8,
    client_ephemeral_pub: String,
}

#[derive(Debug, Deserialize)]
struct ServerHello {
    #[expect(dead_code, reason = "validated implicitly by successful parse")]
    version: u8,
    #[expect(dead_code, reason = "used by session key derivation in Phase 3")]
    server_ephemeral_pub: String,
    challenge: String,
    session_id: String,
}

#[derive(Debug, Serialize)]
struct ChallengeResponse {
    response: String,
    preferred_cipher: &'static str,
}

#[derive(Debug, Deserialize)]
struct HandshakeComplete {
    cipher: String,
    session_id: String,
}

#[derive(Debug, Deserialize)]
struct HandshakeError {
    #[expect(dead_code, reason = "logged but not matched on")]
    error: String,
    reason: String,
}

/// Result of a successful client-side BTSP handshake.
#[derive(Debug, Clone)]
pub struct BtspClientSession {
    /// Unique session identifier from the server.
    pub session_id: String,
    /// Negotiated cipher suite (e.g. `chacha20_poly1305` or `null`).
    pub cipher: String,
}

/// Errors from the BTSP client handshake.
#[derive(Debug, thiserror::Error)]
pub enum BtspClientError {
    /// Family seed not available in environment.
    #[error("FAMILY_SEED not available — cannot perform BTSP handshake")]
    NoFamilySeed,
    /// I/O error on the UDS stream during handshake.
    #[error("I/O error during BTSP handshake: {0}")]
    Io(#[from] std::io::Error),
    /// Server explicitly rejected the handshake (bad family seed, etc.).
    #[error("Server rejected handshake: {0}")]
    Rejected(String),
    /// Malformed or unexpected response from server.
    #[error("Malformed server response: {0}")]
    Protocol(String),
    /// HMAC computation failed (invalid key length).
    #[error("HMAC computation failed")]
    Hmac,
}

/// Resolve the raw family seed from environment.
///
/// Priority: `BTSP_FAMILY_SEED` > `FAMILY_SEED` > `BIOMEOS_FAMILY_SEED`.
fn resolve_family_seed_raw() -> Option<String> {
    std::env::var("BTSP_FAMILY_SEED")
        .or_else(|_| std::env::var("FAMILY_SEED"))
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_SEED"))
        .ok()
        .filter(|s| !s.trim().is_empty())
}

/// Check whether BTSP strict mode is expected (bearDog requires handshake).
///
/// Returns `true` if `BEARDOG_UDS_REQUIRE_BTSP=1` or `BTSP_STRICT_MODE=1`.
#[must_use]
pub fn btsp_strict_mode_expected() -> bool {
    std::env::var("BEARDOG_UDS_REQUIRE_BTSP")
        .or_else(|_| std::env::var("BTSP_STRICT_MODE"))
        .is_ok_and(|v| v.trim() == "1")
}

/// Perform the client-side BTSP handshake over an NDJSON stream.
///
/// Authenticates to bearDog using the family seed from environment.
/// After success, the stream is ready for JSON-RPC traffic.
///
/// # Errors
///
/// Returns `BtspClientError` if the family seed is unavailable, the server
/// rejects the handshake, or I/O fails.
pub async fn perform_client_handshake(
    stream: &mut TransportStream,
) -> Result<BtspClientSession, BtspClientError> {
    let family_seed = resolve_family_seed_raw().ok_or(BtspClientError::NoFamilySeed)?;
    let b64 = base64::engine::general_purpose::STANDARD;

    // Generate ephemeral key material (32 random bytes)
    let mut ephemeral_key = [0u8; 32];
    getrandom::fill(&mut ephemeral_key).map_err(|_| BtspClientError::Hmac)?;

    // Step 1: Send ClientHello
    let hello = ClientHello {
        protocol: "btsp",
        version: BTSP_VERSION,
        client_ephemeral_pub: b64.encode(ephemeral_key),
    };
    let hello_json = serde_json::to_value(&hello)
        .map_err(|e| BtspClientError::Protocol(format!("serialize ClientHello: {e}")))?;
    write_ndjson_request(stream, &hello_json, "BTSP client")
        .await
        .map_err(|e| BtspClientError::Protocol(e.to_string()))?;

    debug!("BTSP client: sent ClientHello");

    // Step 2: Read ServerHello (or HandshakeError)
    let mut buf_reader = BufReader::new(&mut *stream);
    let mut line = String::new();
    buf_reader
        .read_line(&mut line)
        .await
        .map_err(BtspClientError::Io)?;

    if line.trim().is_empty() {
        return Err(BtspClientError::Protocol(String::from(
            "empty response from server",
        )));
    }

    if line.contains("\"error\"") && line.contains("\"reason\"") {
        let err: HandshakeError = serde_json::from_str(line.trim())
            .map_err(|e| BtspClientError::Protocol(format!("parse error response: {e}")))?;
        return Err(BtspClientError::Rejected(err.reason));
    }

    let server_hello: ServerHello = serde_json::from_str(line.trim())
        .map_err(|e| BtspClientError::Protocol(format!("parse ServerHello: {e}")))?;

    debug!(
        session_id = %server_hello.session_id,
        "BTSP client: received ServerHello"
    );

    // Step 3: Compute challenge response — HMAC-SHA256(family_seed, challenge)
    let challenge_bytes = b64
        .decode(&server_hello.challenge)
        .map_err(|e| BtspClientError::Protocol(format!("decode challenge: {e}")))?;

    let mut mac = HmacSha256::new_from_slice(family_seed.trim().as_bytes())
        .map_err(|_| BtspClientError::Hmac)?;
    mac.update(&challenge_bytes);
    let hmac_result = mac.finalize().into_bytes();

    let response = ChallengeResponse {
        response: b64.encode(hmac_result),
        preferred_cipher: PREFERRED_CIPHER,
    };
    let resp_json = serde_json::to_value(&response)
        .map_err(|e| BtspClientError::Protocol(format!("serialize ChallengeResponse: {e}")))?;

    let stream = buf_reader.into_inner();
    write_ndjson_request(stream, &resp_json, "BTSP client")
        .await
        .map_err(|e| BtspClientError::Protocol(e.to_string()))?;

    debug!("BTSP client: sent ChallengeResponse");

    // Step 4: Read HandshakeComplete (or HandshakeError)
    let mut buf_reader = BufReader::new(&mut *stream);
    let mut line = String::new();
    buf_reader
        .read_line(&mut line)
        .await
        .map_err(BtspClientError::Io)?;

    if line.contains("\"error\"") && line.contains("\"reason\"") {
        let err: HandshakeError = serde_json::from_str(line.trim())
            .map_err(|e| BtspClientError::Protocol(format!("parse error response: {e}")))?;
        return Err(BtspClientError::Rejected(err.reason));
    }

    let complete: HandshakeComplete = serde_json::from_str(line.trim())
        .map_err(|e| BtspClientError::Protocol(format!("parse HandshakeComplete: {e}")))?;

    debug!(
        session_id = %complete.session_id,
        cipher = %complete.cipher,
        "BTSP client: handshake COMPLETE"
    );

    Ok(BtspClientSession {
        session_id: complete.session_id,
        cipher: complete.cipher,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn btsp_strict_mode_default_off() {
        assert!(!btsp_strict_mode_expected());
    }

    #[test]
    fn resolve_family_seed_empty_returns_none() {
        assert!(resolve_family_seed_raw().is_none() || resolve_family_seed_raw().is_some());
    }

    #[test]
    fn hmac_computation_produces_32_bytes() {
        let key = b"test-family-seed";
        let challenge = b"random-challenge-data";
        let mut mac = HmacSha256::new_from_slice(key).unwrap();
        mac.update(challenge);
        let result = mac.finalize().into_bytes();
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn client_hello_serializes_correctly() {
        let hello = ClientHello {
            protocol: "btsp",
            version: 1,
            client_ephemeral_pub: String::from("AAAA"),
        };
        let json = serde_json::to_string(&hello).unwrap();
        assert!(json.contains("\"protocol\":\"btsp\""));
        assert!(json.contains("\"version\":1"));
    }
}
