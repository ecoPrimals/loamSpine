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

    let challenge = generate_challenge();
    let create_result = create_provider_session(provider_socket, &client_hello, &challenge).await?;

    send_server_hello(writer, &create_result, &challenge).await?;

    let challenge_response = read_challenge_response(reader).await?;

    verify_and_complete(
        writer,
        provider_socket,
        &client_hello,
        &create_result,
        &challenge,
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
async fn create_provider_session(
    provider_socket: &Path,
    client_hello: &ClientHello,
    challenge: &str,
) -> Result<SessionCreateResult, LoamSpineError> {
    let result: SessionCreateResult = provider_call(
        provider_socket,
        "btsp.session.create",
        serde_json::json!({
            "family_seed_ref": "env:FAMILY_SEED",
            "client_ephemeral_pub": client_hello.client_ephemeral_pub,
            "challenge": challenge,
        }),
        rpc_id::SESSION_CREATE,
    )
    .await?;
    debug!("BTSP: session created: {}", result.session_id);
    Ok(result)
}

/// Step 3: Send `ServerHello` with the provider-generated ephemeral key and challenge.
async fn send_server_hello<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    create_result: &SessionCreateResult,
    challenge: &str,
) -> Result<(), LoamSpineError> {
    let hello = ServerHello {
        version: BTSP_VERSION,
        server_ephemeral_pub: create_result.server_ephemeral_pub.clone(),
        challenge: challenge.to_string(),
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
async fn verify_and_complete<W: AsyncWriteExt + Unpin + Send>(
    writer: &mut W,
    provider_socket: &Path,
    client_hello: &ClientHello,
    create_result: &SessionCreateResult,
    challenge: &str,
    challenge_response: &ChallengeResponse,
) -> Result<BtspSession, LoamSpineError> {
    let verify: SessionVerifyResult = provider_call(
        provider_socket,
        "btsp.session.verify",
        serde_json::json!({
            "session_id": create_result.session_id,
            "client_response": challenge_response.response,
            "client_ephemeral_pub": client_hello.client_ephemeral_pub,
            "server_ephemeral_pub": create_result.server_ephemeral_pub,
            "challenge": challenge,
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

    let negotiate: NegotiateResult = provider_call(
        provider_socket,
        "btsp.negotiate",
        serde_json::json!({
            "session_id": create_result.session_id,
            "preferred_cipher": challenge_response.preferred_cipher,
            "bond_type": "Covalent",
        }),
        rpc_id::NEGOTIATE,
    )
    .await?;

    if !negotiate.allowed {
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
        cipher: negotiate.cipher.clone(),
        session_id: create_result.session_id.clone(),
    };
    let bytes = serialize_btsp_msg(&complete, "HandshakeComplete")?;
    write_frame(writer, &bytes).await?;

    debug!(
        "BTSP: handshake complete (session={}, cipher={})",
        create_result.session_id, negotiate.cipher
    );

    Ok(BtspSession {
        session_id: create_result.session_id.clone(),
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

    let challenge = generate_challenge();
    let create_result = create_provider_session(
        provider_socket,
        &ClientHello {
            version: hello.version,
            client_ephemeral_pub: hello.client_ephemeral_pub.clone(),
        },
        &challenge,
    )
    .await?;

    let server_hello = NdjsonServerHello {
        version: BTSP_VERSION,
        server_ephemeral_pub: create_result.server_ephemeral_pub.clone(),
        challenge: challenge.clone(),
        session_id: create_result.session_id.clone(),
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
        &challenge,
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
    challenge: &str,
    challenge_response: &ChallengeResponse,
) -> Result<BtspSession, LoamSpineError> {
    let verify: SessionVerifyResult = provider_call(
        provider_socket,
        "btsp.session.verify",
        serde_json::json!({
            "session_id": create_result.session_id,
            "client_response": challenge_response.response,
            "client_ephemeral_pub": client_hello.client_ephemeral_pub,
            "server_ephemeral_pub": create_result.server_ephemeral_pub,
            "challenge": challenge,
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

    let negotiate: NegotiateResult = provider_call(
        provider_socket,
        "btsp.negotiate",
        serde_json::json!({
            "session_id": create_result.session_id,
            "preferred_cipher": challenge_response.preferred_cipher,
            "bond_type": "Covalent",
        }),
        rpc_id::NEGOTIATE,
    )
    .await?;

    if !negotiate.allowed {
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
        cipher: negotiate.cipher.clone(),
        session_id: create_result.session_id.clone(),
    };
    ndjson_send(writer, &complete, "HandshakeComplete").await?;

    debug!(
        "BTSP NDJSON: handshake complete (session={}, cipher={})",
        create_result.session_id, negotiate.cipher
    );

    Ok(BtspSession {
        session_id: create_result.session_id.clone(),
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

/// Generate a hex-encoded 32-byte challenge from OS entropy.
///
/// Uses `blake3(uuid_v7_a || uuid_v7_b)` to produce a full 32-byte challenge
/// from UUID v7's OS-sourced randomness (74 random bits per UUID via `getrandom`).
/// The BTSP provider remains the authority for session key material — this challenge seeds
/// the `btsp.session.create` call which may augment it with its own entropy.
pub(crate) fn generate_challenge() -> String {
    use std::fmt::Write;

    let a = uuid::Uuid::now_v7();
    let b = uuid::Uuid::now_v7();
    let hash = blake3::Hasher::new()
        .update(a.as_bytes())
        .update(b.as_bytes())
        .finalize();

    let mut s = String::with_capacity(64);
    for byte in hash.as_bytes() {
        let _ = write!(s, "{byte:02x}");
    }
    s
}
