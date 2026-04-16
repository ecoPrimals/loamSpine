// SPDX-License-Identifier: AGPL-3.0-or-later

//! JSON-RPC crypto provider adapter for production signing and verification.
//!
//! Implements the `Signer` and `Verifier` traits by delegating to a
//! capability-discovered crypto provider via the `crypto.sign_ed25519`
//! and `crypto.verify_ed25519` JSON-RPC wire contract defined in
//! `CRYPTO_WIRE_CONTRACT.md`.
//!
//! This is the **production signing path** — the crypto provider is
//! discovered at runtime via capability-based discovery. The `CliSigner`
//! remains available as a development fallback.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use base64::Engine;
use serde::Deserialize;

use crate::error::{IpcErrorPhase, LoamSpineError, LoamSpineResult};
use crate::traits::{SignatureVerification, Signer, Verifier};
use crate::types::{Did, Signature};

/// JSON-RPC signer backed by a crypto capability provider.
///
/// Connects to the provider via UDS and calls `crypto.sign_ed25519`
/// per `CRYPTO_WIRE_CONTRACT.md`. Each call opens a fresh connection
/// (stateless request/response over NDJSON framing).
pub struct JsonRpcCryptoSigner {
    socket_path: PathBuf,
    key_id: Option<String>,
    did: Did,
    request_counter: AtomicU64,
}

/// JSON-RPC verifier backed by a crypto capability provider.
///
/// Calls `crypto.verify_ed25519` on the provider with base64-encoded
/// message, signature, and public key.
pub struct JsonRpcCryptoVerifier {
    socket_path: PathBuf,
    request_counter: AtomicU64,
}

#[derive(Deserialize)]
struct SignResponse {
    signature: String,
    algorithm: Option<String>,
}

#[derive(Deserialize)]
struct VerifyResponse {
    valid: bool,
}

impl JsonRpcCryptoSigner {
    /// Create a new JSON-RPC crypto signer.
    ///
    /// `socket_path` is the UDS endpoint of the crypto capability provider
    /// (discovered at runtime, never hardcoded).
    #[must_use]
    pub const fn new(socket_path: PathBuf, did: Did, key_id: Option<String>) -> Self {
        Self {
            socket_path,
            key_id,
            did,
            request_counter: AtomicU64::new(1),
        }
    }

    fn next_id(&self) -> u64 {
        self.request_counter.fetch_add(1, Ordering::Relaxed)
    }
}

impl JsonRpcCryptoVerifier {
    /// Create a new JSON-RPC crypto verifier.
    #[must_use]
    pub const fn new(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            request_counter: AtomicU64::new(1),
        }
    }

    fn next_id(&self) -> u64 {
        self.request_counter.fetch_add(1, Ordering::Relaxed)
    }
}

impl Signer for JsonRpcCryptoSigner {
    async fn sign(&self, data: &[u8]) -> LoamSpineResult<Signature> {
        let b64 = base64::engine::general_purpose::STANDARD;
        let message_b64 = b64.encode(data);

        let mut params = serde_json::json!({ "message": message_b64 });
        if let Some(ref kid) = self.key_id {
            params["key_id"] = serde_json::Value::String(kid.clone());
        }

        let resp: SignResponse = crypto_provider_call(
            &self.socket_path,
            "crypto.sign_ed25519",
            params,
            self.next_id(),
        )
        .await?;

        if let Some(ref algo) = resp.algorithm {
            tracing::trace!(algorithm = %algo, "crypto provider signed with algorithm");
        }

        let sig_bytes = b64.decode(&resp.signature).map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::InvalidJson,
                format!("crypto provider signature base64 decode: {e}"),
            )
        })?;

        Ok(Signature::from_vec(sig_bytes))
    }

    fn did(&self) -> &Did {
        &self.did
    }
}

impl Verifier for JsonRpcCryptoVerifier {
    async fn verify(
        &self,
        data: &[u8],
        signature: &Signature,
        signer: &Did,
    ) -> LoamSpineResult<SignatureVerification> {
        let b64 = base64::engine::general_purpose::STANDARD;

        let params = serde_json::json!({
            "message": b64.encode(data),
            "signature": b64.encode(&signature.0),
            "public_key": b64.encode(signer.as_str().as_bytes()),
        });

        let resp: VerifyResponse = crypto_provider_call(
            &self.socket_path,
            "crypto.verify_ed25519",
            params,
            self.next_id(),
        )
        .await?;

        Ok(if resp.valid {
            SignatureVerification::valid()
        } else {
            SignatureVerification::invalid("crypto provider: signature invalid")
        })
    }

    async fn verify_entry(
        &self,
        entry: &crate::entry::Entry,
    ) -> LoamSpineResult<SignatureVerification> {
        let data = entry.to_canonical_bytes()?;
        self.verify(&data, &entry.signature, &entry.committer).await
    }
}

/// Send a JSON-RPC request to the crypto provider over UDS.
///
/// Reuses the same NDJSON-over-UDS pattern as `btsp::provider_client`.
async fn crypto_provider_call<R: serde::de::DeserializeOwned>(
    socket: &Path,
    method: &str,
    params: serde_json::Value,
    request_id: u64,
) -> Result<R, LoamSpineError> {
    use tokio::io::AsyncWriteExt;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": request_id,
    });
    let request_bytes = serde_json::to_vec(&request).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Serialization,
            format!("crypto provider request serialize: {e}"),
        )
    })?;

    let stream = tokio::net::UnixStream::connect(socket).await.map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Connect,
            format!(
                "crypto provider socket {} unreachable: {e}",
                socket.display()
            ),
        )
    })?;

    let (reader, mut writer) = stream.into_split();

    writer.write_all(&request_bytes).await.map_err(|e| {
        LoamSpineError::ipc(IpcErrorPhase::Write, format!("crypto provider write: {e}"))
    })?;
    writer.write_all(b"\n").await.map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Write,
            format!("crypto provider newline: {e}"),
        )
    })?;
    writer.flush().await.map_err(|e| {
        LoamSpineError::ipc(IpcErrorPhase::Write, format!("crypto provider flush: {e}"))
    })?;
    writer.shutdown().await.map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Write,
            format!("crypto provider shutdown: {e}"),
        )
    })?;

    let mut response_line = String::new();
    let mut buf_reader = tokio::io::BufReader::new(reader);
    tokio::io::AsyncBufReadExt::read_line(&mut buf_reader, &mut response_line)
        .await
        .map_err(|e| {
            LoamSpineError::ipc(IpcErrorPhase::Read, format!("crypto provider read: {e}"))
        })?;

    let response: serde_json::Value = serde_json::from_str(response_line.trim()).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::InvalidJson,
            format!("crypto provider {method} response parse: {e}"),
        )
    })?;

    if let Some(err) = response.get("error") {
        let code = err
            .get("code")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(-1);
        let msg = err
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown crypto provider error");
        return Err(LoamSpineError::ipc(
            IpcErrorPhase::JsonRpcError(code),
            format!("crypto provider {method}: {msg}"),
        ));
    }

    let result = response.get("result").ok_or_else(|| {
        LoamSpineError::ipc(
            IpcErrorPhase::NoResult,
            format!("crypto provider {method}: missing result field"),
        )
    })?;

    R::deserialize(result).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::InvalidJson,
            format!("crypto provider {method} result deserialize: {e}"),
        )
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "tests use unwrap for concise assertions"
)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    use tokio::net::UnixListener;

    async fn spawn_mock_crypto_provider(
        temp_dir: &std::path::Path,
    ) -> (PathBuf, tokio::task::JoinHandle<()>) {
        let socket_path = temp_dir.join("crypto-provider.sock");
        let _ = std::fs::remove_file(&socket_path);
        let listener = UnixListener::bind(&socket_path).unwrap();

        let path = socket_path.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..5 {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    handle_mock_crypto_request(stream).await;
                });
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        (path, handle)
    }

    async fn handle_mock_crypto_request(stream: tokio::net::UnixStream) {
        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = tokio::io::BufReader::new(reader);
        let mut line = String::new();
        let _ = tokio::io::AsyncBufReadExt::read_line(&mut buf_reader, &mut line).await;

        let request: serde_json::Value =
            serde_json::from_str(line.trim()).unwrap_or(serde_json::Value::Null);

        let method = request
            .get("method")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let id = request
            .get("id")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        let b64 = base64::engine::general_purpose::STANDARD;

        let response = match method {
            "crypto.sign_ed25519" => {
                let msg_b64 = request["params"]["message"].as_str().unwrap_or("");
                let msg_bytes = b64.decode(msg_b64).unwrap_or_default();
                let mock_sig = crate::types::hash_bytes(&msg_bytes);
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "signature": b64.encode(mock_sig),
                        "algorithm": "ed25519"
                    }
                })
            }
            "crypto.verify_ed25519" => {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "valid": true }
                })
            }
            _ => serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": "method not found" }
            }),
        };

        let mut response_bytes = serde_json::to_vec(&response).unwrap();
        response_bytes.push(b'\n');
        let _ = writer.write_all(&response_bytes).await;
        let _ = writer.flush().await;
    }

    #[tokio::test]
    async fn sign_via_crypto_provider() {
        let tmp = tempfile::tempdir().unwrap();
        let (socket, _handle) = spawn_mock_crypto_provider(tmp.path()).await;

        let signer = JsonRpcCryptoSigner::new(socket, Did::new("did:key:z6MkTest"), None);

        let sig = signer.sign(b"hello world").await.unwrap();
        assert!(!sig.0.is_empty());
    }

    #[tokio::test]
    async fn sign_with_key_id() {
        let tmp = tempfile::tempdir().unwrap();
        let (socket, _handle) = spawn_mock_crypto_provider(tmp.path()).await;

        let signer = JsonRpcCryptoSigner::new(
            socket,
            Did::new("did:key:z6MkTest"),
            Some("my-key-1".into()),
        );

        let sig = signer.sign(b"data").await.unwrap();
        assert!(!sig.0.is_empty());
    }

    #[tokio::test]
    async fn sign_deterministic_for_same_data() {
        let tmp = tempfile::tempdir().unwrap();
        let (socket, _handle) = spawn_mock_crypto_provider(tmp.path()).await;

        let signer = JsonRpcCryptoSigner::new(socket, Did::new("did:key:z6MkTest"), None);

        let sig1 = signer.sign(b"same").await.unwrap();
        let sig2 = signer.sign(b"same").await.unwrap();
        assert_eq!(sig1, sig2);
    }

    #[tokio::test]
    async fn sign_different_for_different_data() {
        let tmp = tempfile::tempdir().unwrap();
        let (socket, _handle) = spawn_mock_crypto_provider(tmp.path()).await;

        let signer = JsonRpcCryptoSigner::new(socket, Did::new("did:key:z6MkTest"), None);

        let sig_a = signer.sign(b"alpha").await.unwrap();
        let sig_b = signer.sign(b"beta").await.unwrap();
        assert_ne!(sig_a, sig_b);
    }

    #[tokio::test]
    async fn verify_via_crypto_provider() {
        let tmp = tempfile::tempdir().unwrap();
        let (socket, _handle) = spawn_mock_crypto_provider(tmp.path()).await;

        let verifier = JsonRpcCryptoVerifier::new(socket);

        let result = verifier
            .verify(
                b"hello",
                &Signature::from_vec(vec![1, 2, 3]),
                &Did::new("did:key:z6MkPeer"),
            )
            .await
            .unwrap();

        assert!(result.valid);
    }

    #[tokio::test]
    async fn signer_did_returns_configured_did() {
        let did = Did::new("did:key:z6MkMine");
        let signer =
            JsonRpcCryptoSigner::new(PathBuf::from("/tmp/nonexistent.sock"), did.clone(), None);
        assert_eq!(signer.did(), &did);
    }

    #[tokio::test]
    async fn sign_fails_when_provider_unreachable() {
        let signer = JsonRpcCryptoSigner::new(
            PathBuf::from("/tmp/no-such-crypto-provider.sock"),
            Did::new("did:key:z6MkTest"),
            None,
        );

        let result = signer.sign(b"data").await;
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(
            err_str.contains("unreachable") || err_str.contains("connect"),
            "unexpected error: {err_str}"
        );
    }

    #[tokio::test]
    async fn verify_fails_when_provider_unreachable() {
        let verifier = JsonRpcCryptoVerifier::new(PathBuf::from("/tmp/no-such.sock"));

        let result = verifier
            .verify(
                b"data",
                &Signature::from_vec(vec![1]),
                &Did::new("did:key:z6Mk"),
            )
            .await;
        assert!(result.is_err());
    }
}
