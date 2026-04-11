// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP server-side handshake protocol.
//!
//! Implements the 4-step handshake sequence per `BTSP_PROTOCOL_STANDARD.md`:
//!
//! 1. Read `ClientHello` and validate version
//! 2. Call `btsp.session.create` on BearDog → exchange keys and challenge
//! 3. Read `ChallengeResponse` and verify via `btsp.session.verify`
//! 4. Negotiate cipher via `btsp.negotiate`, send `HandshakeComplete`

use std::path::Path;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::debug;

use super::beardog_client::beardog_call;
use super::frame::{deserialize_btsp_msg, read_frame, serialize_btsp_msg, write_frame};
use super::wire::{
    BtspSession, ChallengeResponse, ClientHello, HandshakeComplete, HandshakeError,
    NegotiateResult, ServerHello, SessionCreateResult, SessionVerifyResult,
};
use crate::error::{IpcErrorPhase, LoamSpineError};

/// BTSP protocol version.
const BTSP_VERSION: u32 = 1;

/// Perform the BTSP server-side handshake on an accepted UDS connection.
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

    let challenge = generate_challenge();
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

/// Generate a hex-encoded 32-byte challenge from OS entropy.
///
/// Uses `blake3(uuid_v7_a || uuid_v7_b)` to produce a full 32-byte challenge
/// from UUID v7's OS-sourced randomness (74 random bits per UUID via `getrandom`).
/// BearDog remains the authority for session key material — this challenge seeds
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
