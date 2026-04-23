// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP server-side handshake protocol.
//!
//! Implements the 4-step handshake sequence per `BTSP_PROTOCOL_STANDARD.md`:
//!
//! 1. Read `ClientHello` and validate version
//! 2. Call `btsp.session.create` on BTSP provider → exchange keys and challenge
//! 3. Read `ChallengeResponse` and verify via `btsp.session.verify`
//! 4. Negotiate cipher via `btsp.negotiate`, send `HandshakeComplete`

use std::path::Path;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tracing::debug;

use super::frame::{deserialize_btsp_msg, read_frame, serialize_btsp_msg, write_frame};
use super::provider_client::provider_call;
use super::wire::{
    BtspSession, ChallengeResponse, ClientHello, HandshakeComplete, HandshakeError,
    NdjsonClientHello, NdjsonServerHello, NegotiateResult, ServerHello, SessionCreateResult,
    SessionVerifyResult,
};
use crate::error::{IpcErrorPhase, LoamSpineError};

/// BTSP protocol version.
const BTSP_VERSION: u32 = 1;

/// JSON-RPC request IDs for BTSP provider delegation calls.
mod rpc_id {
    pub(super) const SESSION_CREATE: u64 = 1;
    pub(super) const SESSION_VERIFY: u64 = 2;
    pub(super) const NEGOTIATE: u64 = 3;
}

/// Perform the BTSP server-side handshake on an accepted UDS connection.
///
/// Accepts separate reader and writer halves so the caller can use
/// `BufReader` for wire-format peeking before routing here.
///
/// # Errors
///
/// Returns `LoamSpineError` if the handshake fails at any step. The caller
/// should close the connection on error.
pub async fn perform_server_handshake<R, W>(
    reader: &mut R,
    writer: &mut W,
    provider_socket: &Path,
) -> Result<BtspSession, LoamSpineError>
where
    R: AsyncReadExt + Unpin + Send,
    W: AsyncWriteExt + Unpin + Send,
{
    let client_hello = read_and_validate_client_hello(reader, writer).await?;

    let create_result = create_provider_session(provider_socket).await?;

    send_server_hello(writer, &create_result).await?;

    let challenge_response = read_challenge_response(reader).await?;

    verify_and_complete(
        writer,
        provider_socket,
        &client_hello,
        &create_result,
        &challenge_response,
    )
    .await
}

/// Step 1: Read `ClientHello` and validate the protocol version.
async fn read_and_validate_client_hello<R: AsyncReadExt + Unpin, W: AsyncWriteExt + Unpin>(
    reader: &mut R,
    writer: &mut W,
) -> Result<ClientHello, LoamSpineError> {
    let bytes = read_frame(reader).await?;
    let hello: ClientHello = deserialize_btsp_msg(&bytes, "ClientHello")?;

    if hello.version != BTSP_VERSION {
        send_handshake_error(
            writer,
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

/// Step 2: Create a BTSP session on the handshake provider.
///
/// Sends only `family_seed` (base64-encoded) per BearDog's
/// `SessionCreateParams`. BearDog generates the challenge and ephemeral
/// keys server-side, returning them in `SessionCreateResponse`.
async fn create_provider_session(
    provider_socket: &Path,
) -> Result<SessionCreateResult, LoamSpineError> {
    let family_seed = resolve_family_seed()?;
    let result: SessionCreateResult = provider_call(
        provider_socket,
        "btsp.session.create",
        serde_json::json!({
            "family_seed": family_seed,
        }),
        rpc_id::SESSION_CREATE,
    )
    .await?;
    debug!("BTSP: session created: {}", result.session_token);
    Ok(result)
}

/// Step 3: Send `ServerHello` with the provider-generated ephemeral key and challenge.
async fn send_server_hello<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    create_result: &SessionCreateResult,
) -> Result<(), LoamSpineError> {
    let hello = ServerHello {
        version: BTSP_VERSION,
        server_ephemeral_pub: create_result.server_ephemeral_pub.clone(),
        challenge: create_result.challenge.clone(),
    };
    let bytes = serialize_btsp_msg(&hello, "ServerHello")?;
    write_frame(writer, &bytes).await?;
    debug!("BTSP: sent ServerHello");
    Ok(())
}

/// Step 4: Read the client's `ChallengeResponse`.
async fn read_challenge_response<R: AsyncReadExt + Unpin>(
    reader: &mut R,
) -> Result<ChallengeResponse, LoamSpineError> {
    let bytes = read_frame(reader).await?;
    let cr: ChallengeResponse = deserialize_btsp_msg(&bytes, "ChallengeResponse")?;
    debug!("BTSP: received ChallengeResponse");
    Ok(cr)
}

/// Steps 5–7: Verify via BTSP provider, negotiate cipher, send completion or error.
///
/// Params aligned with BearDog's `SessionVerifyParams` / `SessionNegotiateParams`:
/// - verify: `session_token`, `client_ephemeral_pub`, `response`, `preferred_cipher`
/// - negotiate: `session_token`, `cipher`
async fn verify_and_complete<W: AsyncWriteExt + Unpin + Send>(
    writer: &mut W,
    provider_socket: &Path,
    client_hello: &ClientHello,
    create_result: &SessionCreateResult,
    challenge_response: &ChallengeResponse,
) -> Result<BtspSession, LoamSpineError> {
    let verify: SessionVerifyResult = provider_call(
        provider_socket,
        "btsp.session.verify",
        serde_json::json!({
            "session_token": create_result.session_token,
            "client_ephemeral_pub": client_hello.client_ephemeral_pub,
            "response": challenge_response.response,
            "preferred_cipher": challenge_response.preferred_cipher,
        }),
        rpc_id::SESSION_VERIFY,
    )
    .await?;

    if !verify.verified {
        send_handshake_error(writer, "handshake_failed", "family_verification").await?;
        return Err(LoamSpineError::ipc(
            IpcErrorPhase::Read,
            "BTSP handshake failed: family verification",
        ));
    }
    debug!("BTSP: client verified");

    let session_id = verify
        .session_id
        .unwrap_or_else(|| create_result.session_token.clone());

    let negotiate: NegotiateResult = provider_call(
        provider_socket,
        "btsp.negotiate",
        serde_json::json!({
            "session_token": create_result.session_token,
            "cipher": challenge_response.preferred_cipher,
        }),
        rpc_id::NEGOTIATE,
    )
    .await?;

    if !negotiate.accepted {
        send_handshake_error(
            writer,
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
        status: "ok".into(),
        cipher: negotiate.cipher.clone(),
        session_id: session_id.clone(),
    };
    let bytes = serialize_btsp_msg(&complete, "HandshakeComplete")?;
    write_frame(writer, &bytes).await?;

    debug!(
        "BTSP: handshake complete (session={session_id}, cipher={})",
        negotiate.cipher
    );

    Ok(BtspSession {
        session_id,
        cipher: negotiate.cipher,
    })
}

/// Serialize a `HandshakeError` and send it as a length-prefixed frame.
async fn send_handshake_error<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    error: &str,
    reason: &str,
) -> Result<(), LoamSpineError> {
    let err = HandshakeError {
        error: error.to_string(),
        reason: reason.to_string(),
    };
    let bytes = serialize_btsp_msg(&err, "HandshakeError")?;
    write_frame(writer, &bytes).await
}

// ---------------------------------------------------------------------------
// NDJSON BTSP handshake (primalSpring-compatible)
// ---------------------------------------------------------------------------

/// Perform the BTSP server-side handshake using newline-delimited JSON.
///
/// This is the primalSpring-compatible path: the client sends
/// `{"protocol":"btsp","version":1,"client_ephemeral_pub":"..."}\n` as the
/// first line on a UDS connection. All crypto is still delegated to the
/// BTSP provider — only the wire framing differs from the length-prefixed
/// variant.
///
/// `first_line` is the already-read first line from the connection (the
/// `NdjsonClientHello`). The caller has already peeked it to detect BTSP.
///
/// # Errors
///
/// Returns `LoamSpineError` if the handshake fails at any step.
pub async fn perform_ndjson_server_handshake<R, W>(
    reader: &mut R,
    writer: &mut W,
    provider_socket: &Path,
    first_line: &str,
) -> Result<BtspSession, LoamSpineError>
where
    R: AsyncBufReadExt + Unpin + Send,
    W: AsyncWriteExt + Unpin + Send,
{
    let hello: NdjsonClientHello = serde_json::from_str(first_line.trim()).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::InvalidJson,
            format!("BTSP NDJSON ClientHello parse: {e}"),
        )
    })?;

    if hello.version != BTSP_VERSION {
        ndjson_send_error(
            writer,
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
    debug!("BTSP NDJSON: received ClientHello v{}", hello.version);

    let create_result = create_provider_session(provider_socket).await?;

    let server_hello = NdjsonServerHello {
        version: BTSP_VERSION,
        server_ephemeral_pub: create_result.server_ephemeral_pub.clone(),
        challenge: create_result.challenge.clone(),
        session_id: create_result.session_token.clone(),
    };
    ndjson_send(writer, &server_hello, "ServerHello").await?;
    debug!("BTSP NDJSON: sent ServerHello");

    let mut cr_line = String::new();
    reader.read_line(&mut cr_line).await.map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Read,
            format!("BTSP NDJSON ChallengeResponse read: {e}"),
        )
    })?;
    let challenge_response: ChallengeResponse =
        serde_json::from_str(cr_line.trim()).map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::InvalidJson,
                format!("BTSP NDJSON ChallengeResponse parse: {e}"),
            )
        })?;
    debug!("BTSP NDJSON: received ChallengeResponse");

    let original_hello = ClientHello {
        version: hello.version,
        client_ephemeral_pub: hello.client_ephemeral_pub,
    };

    ndjson_verify_and_complete(
        writer,
        provider_socket,
        &original_hello,
        &create_result,
        &challenge_response,
    )
    .await
}

/// NDJSON verify + complete: same logic as length-prefixed but line-delimited output.
async fn ndjson_verify_and_complete<W: AsyncWriteExt + Unpin + Send>(
    writer: &mut W,
    provider_socket: &Path,
    client_hello: &ClientHello,
    create_result: &SessionCreateResult,
    challenge_response: &ChallengeResponse,
) -> Result<BtspSession, LoamSpineError> {
    let verify: SessionVerifyResult = provider_call(
        provider_socket,
        "btsp.session.verify",
        serde_json::json!({
            "session_token": create_result.session_token,
            "client_ephemeral_pub": client_hello.client_ephemeral_pub,
            "response": challenge_response.response,
            "preferred_cipher": challenge_response.preferred_cipher,
        }),
        rpc_id::SESSION_VERIFY,
    )
    .await?;

    if !verify.verified {
        ndjson_send_error(writer, "handshake_failed", "family_verification").await?;
        return Err(LoamSpineError::ipc(
            IpcErrorPhase::Read,
            "BTSP NDJSON handshake failed: family verification",
        ));
    }
    debug!("BTSP NDJSON: client verified");

    let session_id = verify
        .session_id
        .unwrap_or_else(|| create_result.session_token.clone());

    let negotiate: NegotiateResult = provider_call(
        provider_socket,
        "btsp.negotiate",
        serde_json::json!({
            "session_token": create_result.session_token,
            "cipher": challenge_response.preferred_cipher,
        }),
        rpc_id::NEGOTIATE,
    )
    .await?;

    if !negotiate.accepted {
        ndjson_send_error(
            writer,
            "cipher_rejected",
            "requested cipher not allowed by bond policy",
        )
        .await?;
        return Err(LoamSpineError::ipc(
            IpcErrorPhase::Read,
            "BTSP NDJSON cipher negotiation rejected",
        ));
    }

    let complete = HandshakeComplete {
        status: "ok".into(),
        cipher: negotiate.cipher.clone(),
        session_id: session_id.clone(),
    };
    ndjson_send(writer, &complete, "HandshakeComplete").await?;

    debug!(
        "BTSP NDJSON: handshake complete (session={session_id}, cipher={})",
        negotiate.cipher
    );

    Ok(BtspSession {
        session_id,
        cipher: negotiate.cipher,
    })
}

/// Send a serialized JSON object followed by `\n` (NDJSON framing).
async fn ndjson_send<W: AsyncWriteExt + Unpin + Send, T: serde::Serialize + Sync>(
    stream: &mut W,
    msg: &T,
    label: &str,
) -> Result<(), LoamSpineError> {
    let mut line = serde_json::to_string(msg).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Serialization,
            format!("BTSP NDJSON {label} serialize: {e}"),
        )
    })?;
    line.push('\n');
    stream.write_all(line.as_bytes()).await.map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Write,
            format!("BTSP NDJSON {label} write: {e}"),
        )
    })?;
    stream.flush().await.map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Write,
            format!("BTSP NDJSON {label} flush: {e}"),
        )
    })?;
    Ok(())
}

/// Send a `HandshakeError` as NDJSON.
async fn ndjson_send_error<W: AsyncWriteExt + Unpin + Send>(
    stream: &mut W,
    error: &str,
    reason: &str,
) -> Result<(), LoamSpineError> {
    ndjson_send(
        stream,
        &HandshakeError {
            error: error.to_string(),
            reason: reason.to_string(),
        },
        "HandshakeError",
    )
    .await
}

/// Read the family seed from the environment and base64-encode it for
/// the `btsp.session.create` RPC.
///
/// Resolution order:
/// 1. `FAMILY_SEED` — canonical seed variable set by primalSpring guidestone
/// 2. `BEARDOG_FAMILY_SEED` — BearDog-scoped alias
///
/// The env value is typically a hex string (64 ASCII chars = 32 seed bytes).
/// BearDog expects the raw UTF-8 bytes base64-encoded in the `family_seed`
/// JSON-RPC param.
pub(crate) fn resolve_family_seed() -> Result<String, LoamSpineError> {
    use base64::Engine;

    let raw = std::env::var("FAMILY_SEED")
        .or_else(|_| std::env::var("BEARDOG_FAMILY_SEED"))
        .map_err(|_| {
            LoamSpineError::ipc(
                IpcErrorPhase::Connect,
                "FAMILY_SEED not set (checked FAMILY_SEED and BEARDOG_FAMILY_SEED)",
            )
        })?;

    Ok(base64::engine::general_purpose::STANDARD.encode(raw.as_bytes()))
}
