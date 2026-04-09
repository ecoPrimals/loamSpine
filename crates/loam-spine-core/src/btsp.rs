// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP (BearDog Secure Tunnel Protocol) handshake integration.
//!
//! Implements the **consumer side** of BTSP Phase 2 for LoamSpine's UDS listener.
//! LoamSpine does NOT implement cryptographic operations directly — all crypto
//! is delegated to BearDog via JSON-RPC ("handshake-as-a-service").
//!
//! ## Architecture
//!
//! ```text
//! Client ──connect──▶ LoamSpine UDS
//!                        │
//!                        ├─ Read ClientHello (length-prefixed frame)
//!                        ├─ Call BearDog btsp.session.create → get server keys
//!                        ├─ Send ServerHello to client
//!                        ├─ Read ChallengeResponse from client
//!                        ├─ Call BearDog btsp.session.verify → verify HMAC
//!                        ├─ Call BearDog btsp.negotiate → cipher suite
//!                        ├─ Send HandshakeComplete / HandshakeError
//!                        └─ Return BtspSession on success
//! ```
//!
//! ## Wire Format
//!
//! All BTSP frames use 4-byte big-endian length prefix per
//! `BTSP_PROTOCOL_STANDARD.md` §Wire Framing.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::debug;

use crate::error::{IpcErrorPhase, LoamSpineError};

/// Maximum BTSP frame size (16 MiB) per `BTSP_PROTOCOL_STANDARD.md`.
const MAX_FRAME_SIZE: u32 = 0x0100_0000;

/// BTSP protocol version.
const BTSP_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// Wire types
// ---------------------------------------------------------------------------

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
    /// Random 32-byte challenge (base64).
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

// ---------------------------------------------------------------------------
// BearDog JSON-RPC request/response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct SessionCreateResult {
    session_id: String,
    server_ephemeral_pub: String,
    #[expect(dead_code, reason = "reserved for Phase 3 encrypted framing")]
    handshake_key: String,
}

#[derive(Debug, Deserialize)]
struct SessionVerifyResult {
    verified: bool,
    #[expect(dead_code, reason = "reserved for Phase 3 encrypted framing")]
    session_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NegotiateResult {
    cipher: String,
    allowed: bool,
}

// ---------------------------------------------------------------------------
// Session state
// ---------------------------------------------------------------------------

/// An authenticated BTSP session.
#[derive(Debug, Clone)]
pub struct BtspSession {
    /// Unique session identifier (hex).
    pub session_id: String,
    /// Negotiated cipher suite (e.g. `"null"`, `"chacha20_poly1305"`).
    pub cipher: String,
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// BTSP handshake configuration, derived from environment.
///
/// When `required` is `true`, every incoming UDS connection must complete the
/// BTSP handshake before any JSON-RPC methods are exposed. When `false`,
/// raw newline-delimited JSON-RPC is accepted (development mode).
#[derive(Debug, Clone)]
pub struct BtspHandshakeConfig {
    /// Whether BTSP handshake is mandatory.
    pub required: bool,
    /// Path to the BearDog UDS socket for handshake-as-a-service calls.
    pub beardog_socket: PathBuf,
    /// Family ID (for logging/diagnostics).
    pub family_id: String,
}

impl BtspHandshakeConfig {
    /// Derive BTSP configuration from explicit values (pure, no env reads).
    ///
    /// BTSP is required when `family_id` is set and not `"default"`.
    #[must_use]
    pub fn from_values(
        family_id: Option<&str>,
        beardog_socket_override: Option<&str>,
        socket_dir: Option<&str>,
    ) -> Option<Self> {
        let fid = family_id.filter(|s| !s.is_empty() && *s != "default")?;

        let beardog_socket = if let Some(s) = beardog_socket_override {
            PathBuf::from(s)
        } else {
            resolve_beardog_socket_with(Some(fid), socket_dir)
        };

        Some(Self {
            required: true,
            beardog_socket,
            family_id: fid.to_string(),
        })
    }

    /// Derive BTSP configuration from environment variables.
    ///
    /// Returns `Some` when `BIOMEOS_FAMILY_ID` is set to a non-default value,
    /// meaning BTSP is required. Returns `None` in development mode.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        Self::from_values(
            std::env::var("BIOMEOS_FAMILY_ID").ok().as_deref(),
            std::env::var("BEARDOG_SOCKET").ok().as_deref(),
            std::env::var("BIOMEOS_SOCKET_DIR").ok().as_deref(),
        )
    }
}

// ---------------------------------------------------------------------------
// BearDog socket resolution
// ---------------------------------------------------------------------------

/// Resolve the BearDog UDS socket path from explicit values.
///
/// Resolution order:
/// 1. `$BIOMEOS_SOCKET_DIR/beardog-{family_id}.sock`
/// 2. `/run/user/{uid}/biomeos/beardog-{family_id}.sock` (Linux)
/// 3. `$TMPDIR/biomeos/beardog-{family_id}.sock`
#[must_use]
pub fn resolve_beardog_socket_with(family_id: Option<&str>, socket_dir: Option<&str>) -> PathBuf {
    let sock_name = beardog_socket_name(family_id);

    if let Some(dir) = socket_dir {
        return PathBuf::from(dir).join(&sock_name);
    }

    #[cfg(target_os = "linux")]
    if let Some(base) = crate::constants::network::linux_run_user_biomeos() {
        return base.join(&sock_name);
    }

    std::env::temp_dir()
        .join(crate::primal_names::BIOMEOS_SOCKET_DIR)
        .join(sock_name)
}

/// Build the BearDog socket filename.
///
/// - With family: `beardog-{family_id}.sock`
/// - Without family: `beardog.sock`
#[must_use]
fn beardog_socket_name(family_id: Option<&str>) -> String {
    match family_id {
        Some(fid) if !fid.is_empty() && fid != "default" => format!("beardog-{fid}.sock"),
        _ => "beardog.sock".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Frame I/O
// ---------------------------------------------------------------------------

/// Read a length-prefixed BTSP frame from the stream.
///
/// # Errors
///
/// Returns `LoamSpineError::Ipc` if the frame exceeds the maximum size or
/// the stream is closed prematurely.
pub async fn read_frame<R: AsyncReadExt + Unpin>(
    reader: &mut R,
) -> Result<Vec<u8>, LoamSpineError> {
    let len = reader.read_u32().await.map_err(|e| {
        LoamSpineError::ipc(IpcErrorPhase::Read, format!("BTSP frame length read: {e}"))
    })?;

    if len > MAX_FRAME_SIZE {
        return Err(LoamSpineError::ipc(
            IpcErrorPhase::Read,
            format!("BTSP frame too large: {len} bytes (max {MAX_FRAME_SIZE})"),
        ));
    }

    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf).await.map_err(|e| {
        LoamSpineError::ipc(IpcErrorPhase::Read, format!("BTSP frame body read: {e}"))
    })?;

    Ok(buf)
}

/// Write a length-prefixed BTSP frame to the stream.
///
/// # Errors
///
/// Returns `LoamSpineError::Ipc` on write failure.
pub async fn write_frame<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    data: &[u8],
) -> Result<(), LoamSpineError> {
    let len = u32::try_from(data.len()).map_err(|_| {
        LoamSpineError::ipc(
            IpcErrorPhase::Write,
            format!("BTSP frame too large: {} bytes", data.len()),
        )
    })?;

    if len > MAX_FRAME_SIZE {
        return Err(LoamSpineError::ipc(
            IpcErrorPhase::Write,
            format!("BTSP frame too large: {len} bytes (max {MAX_FRAME_SIZE})"),
        ));
    }

    writer.write_u32(len).await.map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Write,
            format!("BTSP frame length write: {e}"),
        )
    })?;
    writer.write_all(data).await.map_err(|e| {
        LoamSpineError::ipc(IpcErrorPhase::Write, format!("BTSP frame body write: {e}"))
    })?;
    writer
        .flush()
        .await
        .map_err(|e| LoamSpineError::ipc(IpcErrorPhase::Write, format!("BTSP frame flush: {e}")))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// BearDog JSON-RPC client (UDS)
// ---------------------------------------------------------------------------

/// Send a JSON-RPC request to BearDog over UDS and return the result.
///
/// Uses newline-delimited JSON-RPC per `PRIMAL_IPC_PROTOCOL.md` v3.1.
/// Accepts a pre-serialized `serde_json::Value` so the future is `Send`.
async fn beardog_call<R: serde::de::DeserializeOwned>(
    socket: &Path,
    method: &str,
    params: serde_json::Value,
    request_id: u64,
) -> Result<R, LoamSpineError> {
    let request_bytes = serialize_beardog_request(method, &params, request_id)?;
    let response = beardog_roundtrip(socket, method, &request_bytes).await?;
    parse_beardog_response(&response, method)
}

/// Build the JSON-RPC request bytes from a pre-built params `Value`.
fn serialize_beardog_request(
    method: &str,
    params: &serde_json::Value,
    request_id: u64,
) -> Result<Vec<u8>, LoamSpineError> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": request_id,
    });
    serde_json::to_vec(&request).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Serialization,
            format!("BTSP beardog request serialize: {e}"),
        )
    })
}

/// Connect to BearDog UDS, send request bytes, return response line.
async fn beardog_roundtrip(
    socket: &Path,
    method: &str,
    request_bytes: &[u8],
) -> Result<serde_json::Value, LoamSpineError> {
    let stream = tokio::net::UnixStream::connect(socket).await.map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Connect,
            format!("BearDog socket {} unreachable: {e}", socket.display()),
        )
    })?;

    let (reader, mut writer) = stream.into_split();

    writer
        .write_all(request_bytes)
        .await
        .map_err(|e| LoamSpineError::ipc(IpcErrorPhase::Write, format!("BearDog write: {e}")))?;
    writer.write_all(b"\n").await.map_err(|e| {
        LoamSpineError::ipc(IpcErrorPhase::Write, format!("BearDog write newline: {e}"))
    })?;
    writer
        .flush()
        .await
        .map_err(|e| LoamSpineError::ipc(IpcErrorPhase::Write, format!("BearDog flush: {e}")))?;
    writer.shutdown().await.map_err(|e| {
        LoamSpineError::ipc(IpcErrorPhase::Write, format!("BearDog shutdown write: {e}"))
    })?;

    let mut response_line = String::new();
    let mut buf_reader = tokio::io::BufReader::new(reader);
    tokio::io::AsyncBufReadExt::read_line(&mut buf_reader, &mut response_line)
        .await
        .map_err(|e| LoamSpineError::ipc(IpcErrorPhase::Read, format!("BearDog read: {e}")))?;

    serde_json::from_str(response_line.trim()).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::InvalidJson,
            format!("BearDog {method} response parse: {e}"),
        )
    })
}

/// Parse a BearDog JSON-RPC response value into the expected result type.
fn parse_beardog_response<R: serde::de::DeserializeOwned>(
    response: &serde_json::Value,
    method: &str,
) -> Result<R, LoamSpineError> {
    if let Some(err) = response.get("error") {
        let code = err
            .get("code")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(-1);
        let msg = err
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown BearDog error");
        return Err(LoamSpineError::ipc(
            IpcErrorPhase::JsonRpcError(code),
            format!("BearDog {method}: {msg}"),
        ));
    }

    let result = response.get("result").ok_or_else(|| {
        LoamSpineError::ipc(
            IpcErrorPhase::NoResult,
            format!("BearDog {method}: missing result field"),
        )
    })?;

    R::deserialize(result).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::InvalidJson,
            format!("BearDog {method} result deserialize: {e}"),
        )
    })
}

// ---------------------------------------------------------------------------
// Server-side handshake
// ---------------------------------------------------------------------------

/// Serialize a `HandshakeError` and send it as a length-prefixed frame.
async fn send_handshake_error<S: AsyncWriteExt + Unpin>(
    stream: &mut S,
    error: &str,
    reason: &str,
) -> Result<(), LoamSpineError> {
    let err = HandshakeError {
        error: error.to_string(),
        reason: reason.to_string(),
    };
    let bytes = serialize_btsp_msg(&err, "HandshakeError")?;
    write_frame(stream, &bytes).await
}

/// Serialize a BTSP wire message to JSON bytes.
fn serialize_btsp_msg<T: Serialize>(msg: &T, label: &str) -> Result<Vec<u8>, LoamSpineError> {
    serde_json::to_vec(msg).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Serialization,
            format!("BTSP {label} serialize: {e}"),
        )
    })
}

/// Deserialize a BTSP wire message from JSON bytes.
fn deserialize_btsp_msg<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    label: &str,
) -> Result<T, LoamSpineError> {
    serde_json::from_slice(bytes).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::InvalidJson,
            format!("BTSP {label} parse: {e}"),
        )
    })
}

/// Perform the BTSP server-side handshake on an accepted UDS connection.
///
/// Implements the 4-step handshake sequence per `BTSP_PROTOCOL_STANDARD.md`:
///
/// 1. Read `ClientHello` and validate version
/// 2. Call `btsp.session.create` on BearDog → exchange keys and challenge
/// 3. Read `ChallengeResponse` and verify via `btsp.session.verify`
/// 4. Negotiate cipher via `btsp.negotiate`, send `HandshakeComplete`
///
/// # Errors
///
/// Returns `LoamSpineError` if the handshake fails at any step. The caller
/// should close the connection on error.
pub async fn perform_server_handshake<S>(
    stream: &mut S,
    beardog_socket: &Path,
) -> Result<BtspSession, LoamSpineError>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    let client_hello = read_and_validate_client_hello(stream).await?;

    let challenge = generate_challenge_placeholder();
    let create_result = create_beardog_session(beardog_socket, &client_hello, &challenge).await?;

    send_server_hello(stream, &create_result, &challenge).await?;

    let challenge_response = read_challenge_response(stream).await?;

    verify_and_complete(
        stream,
        beardog_socket,
        &client_hello,
        &create_result,
        &challenge,
        &challenge_response,
    )
    .await
}

/// Step 1: Read `ClientHello` and validate the protocol version.
async fn read_and_validate_client_hello<S: AsyncReadExt + AsyncWriteExt + Unpin>(
    stream: &mut S,
) -> Result<ClientHello, LoamSpineError> {
    let bytes = read_frame(stream).await?;
    let hello: ClientHello = deserialize_btsp_msg(&bytes, "ClientHello")?;

    if hello.version != BTSP_VERSION {
        send_handshake_error(
            stream,
            "unsupported_version",
            &format!(
                "server supports BTSP v{BTSP_VERSION}, client sent v{}",
                hello.version
            ),
        )
        .await?;
        return Err(LoamSpineError::ipc(
            IpcErrorPhase::Read,
            format!("BTSP version mismatch: {}", hello.version),
        ));
    }

    debug!("BTSP: received ClientHello v{}", hello.version);
    Ok(hello)
}

/// Step 2: Create a BTSP session on BearDog.
async fn create_beardog_session(
    beardog_socket: &Path,
    client_hello: &ClientHello,
    challenge: &str,
) -> Result<SessionCreateResult, LoamSpineError> {
    let result: SessionCreateResult = beardog_call(
        beardog_socket,
        "btsp.session.create",
        serde_json::json!({
            "family_seed_ref": "env:FAMILY_SEED",
            "client_ephemeral_pub": client_hello.client_ephemeral_pub,
            "challenge": challenge,
        }),
        1,
    )
    .await?;
    debug!("BTSP: session created: {}", result.session_id);
    Ok(result)
}

/// Step 3: Send `ServerHello` with the BearDog-generated ephemeral key and challenge.
async fn send_server_hello<S: AsyncWriteExt + Unpin>(
    stream: &mut S,
    create_result: &SessionCreateResult,
    challenge: &str,
) -> Result<(), LoamSpineError> {
    let hello = ServerHello {
        version: BTSP_VERSION,
        server_ephemeral_pub: create_result.server_ephemeral_pub.clone(),
        challenge: challenge.to_string(),
    };
    let bytes = serialize_btsp_msg(&hello, "ServerHello")?;
    write_frame(stream, &bytes).await?;
    debug!("BTSP: sent ServerHello");
    Ok(())
}

/// Step 4: Read the client's `ChallengeResponse`.
async fn read_challenge_response<S: AsyncReadExt + Unpin>(
    stream: &mut S,
) -> Result<ChallengeResponse, LoamSpineError> {
    let bytes = read_frame(stream).await?;
    let cr: ChallengeResponse = deserialize_btsp_msg(&bytes, "ChallengeResponse")?;
    debug!("BTSP: received ChallengeResponse");
    Ok(cr)
}

/// Steps 5–7: Verify via BearDog, negotiate cipher, send completion or error.
async fn verify_and_complete<S: AsyncReadExt + AsyncWriteExt + Unpin>(
    stream: &mut S,
    beardog_socket: &Path,
    client_hello: &ClientHello,
    create_result: &SessionCreateResult,
    challenge: &str,
    challenge_response: &ChallengeResponse,
) -> Result<BtspSession, LoamSpineError> {
    let verify: SessionVerifyResult = beardog_call(
        beardog_socket,
        "btsp.session.verify",
        serde_json::json!({
            "session_id": create_result.session_id,
            "client_response": challenge_response.response,
            "client_ephemeral_pub": client_hello.client_ephemeral_pub,
            "server_ephemeral_pub": create_result.server_ephemeral_pub,
            "challenge": challenge,
        }),
        2,
    )
    .await?;

    if !verify.verified {
        send_handshake_error(stream, "handshake_failed", "family_verification").await?;
        return Err(LoamSpineError::ipc(
            IpcErrorPhase::Read,
            "BTSP handshake failed: family verification",
        ));
    }
    debug!("BTSP: client verified");

    let negotiate: NegotiateResult = beardog_call(
        beardog_socket,
        "btsp.negotiate",
        serde_json::json!({
            "session_id": create_result.session_id,
            "preferred_cipher": challenge_response.preferred_cipher,
            "bond_type": "Covalent",
        }),
        3,
    )
    .await?;

    if !negotiate.allowed {
        send_handshake_error(
            stream,
            "cipher_rejected",
            "requested cipher not allowed by bond policy",
        )
        .await?;
        return Err(LoamSpineError::ipc(
            IpcErrorPhase::Read,
            "BTSP cipher negotiation rejected",
        ));
    }

    let complete = HandshakeComplete {
        cipher: negotiate.cipher.clone(),
        session_id: create_result.session_id.clone(),
    };
    let bytes = serialize_btsp_msg(&complete, "HandshakeComplete")?;
    write_frame(stream, &bytes).await?;

    debug!(
        "BTSP: handshake complete (session={}, cipher={})",
        create_result.session_id, negotiate.cipher
    );

    Ok(BtspSession {
        session_id: create_result.session_id.clone(),
        cipher: negotiate.cipher,
    })
}

/// Generate a base64-encoded 32-byte challenge.
///
/// Uses a deterministic placeholder derived from the current timestamp.
/// BearDog is the true source of cryptographic randomness — this value
/// is passed to `btsp.session.create` which may replace or augment it.
fn generate_challenge_placeholder() -> String {
    use std::fmt::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let mut bytes = [0u8; 32];
    let nanos_bytes = nanos.to_le_bytes();
    bytes[..nanos_bytes.len().min(32)].copy_from_slice(&nanos_bytes[..nanos_bytes.len().min(32)]);

    let mut s = String::with_capacity(44);
    for b in &bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Check whether BTSP is required based on the environment.
///
/// Returns `true` when `BIOMEOS_FAMILY_ID` is set and not `"default"`.
#[must_use]
pub fn is_btsp_required() -> bool {
    is_btsp_required_with(std::env::var("BIOMEOS_FAMILY_ID").ok().as_deref())
}

/// Pure inner function: check BTSP requirement from explicit values.
#[must_use]
pub fn is_btsp_required_with(family_id: Option<&str>) -> bool {
    family_id.is_some_and(|fid| !fid.is_empty() && fid != "default")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "btsp_tests.rs"]
mod tests;
