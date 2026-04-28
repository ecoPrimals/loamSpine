// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tower signing integration tests for `entry.append` and `session.commit`.
//!
//! Validates that when `BEARDOG_SOCKET` is configured (Tower signer present),
//! entries are signed via `crypto.sign_ed25519` and the signature is stored
//! in entry metadata (`tower_signature`, `tower_signature_alg`).

use super::*;
use loam_spine_core::traits::crypto_provider::JsonRpcCryptoSigner;
use loam_spine_core::types::Did;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixListener;

/// Spawn a mock `BearDog` crypto provider that responds to `crypto.sign_ed25519`
/// with a blake3 hash of the message as the signature (deterministic mock).
async fn spawn_mock_tower_signer(
    tmp: &std::path::Path,
    name: &str,
) -> (std::path::PathBuf, tokio::task::JoinHandle<()>) {
    let socket_path = tmp.join(name);
    let listener = UnixListener::bind(&socket_path).expect("bind mock tower");

    let handle = tokio::spawn(async move {
        for _ in 0..5 {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                use base64::Engine;
                let (reader, mut writer) = stream.into_split();
                let mut buf_reader = tokio::io::BufReader::new(reader);
                let mut line = String::new();
                let _ = tokio::io::AsyncBufReadExt::read_line(&mut buf_reader, &mut line).await;
                let request: serde_json::Value =
                    serde_json::from_str(line.trim()).unwrap_or_default();
                let id = request.get("id").cloned().unwrap_or_default();
                let b64 = base64::engine::general_purpose::STANDARD;
                let msg_b64 = request["params"]["message"].as_str().unwrap_or("");
                let msg_bytes = b64.decode(msg_b64).unwrap_or_default();
                let mock_sig = loam_spine_core::types::hash_bytes(&msg_bytes);
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "signature": b64.encode(mock_sig),
                        "algorithm": "ed25519"
                    }
                });
                let mut resp_bytes = serde_json::to_vec(&response).expect("serialize");
                resp_bytes.push(b'\n');
                let _ = writer.write_all(&resp_bytes).await;
                let _ = writer.flush().await;
            });
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    (socket_path, handle)
}

fn tower_service(socket_path: std::path::PathBuf) -> LoamSpineRpcService {
    let signer = Arc::new(JsonRpcCryptoSigner::new(
        socket_path,
        Did::new("did:key:z6MkTower"),
        None,
    ));
    LoamSpineRpcService::default_service().with_tower_signer(signer)
}

#[tokio::test]
async fn test_tower_signed_entry_append() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (socket_path, mock_handle) = spawn_mock_tower_signer(tmp.path(), "mock-beardog.sock").await;
    let service = tower_service(socket_path);

    let create_resp = service
        .create_spine(CreateSpineRequest {
            name: "signed-spine".to_string(),
            owner: Did::new("did:key:z6MkOwner"),
            config: None,
        })
        .await
        .expect("create should succeed");

    let append_resp = service
        .append_entry(AppendEntryRequest {
            spine_id: create_resp.spine_id,
            entry_type: loam_spine_core::entry::EntryType::MetadataUpdate {
                field: "test".to_string(),
                value: "signed-value".to_string(),
            },
            committer: Did::new("did:key:z6MkOwner"),
            payload: None,
        })
        .await
        .expect("append should succeed");

    let entry_resp = service
        .get_entry(GetEntryRequest {
            spine_id: create_resp.spine_id,
            entry_hash: append_resp.entry_hash,
        })
        .await
        .expect("get_entry should succeed");

    assert!(entry_resp.found);
    let entry = entry_resp.entry.expect("entry present");
    assert!(
        entry.metadata.contains_key("tower_signature"),
        "entry should have tower_signature metadata"
    );
    assert_eq!(
        entry
            .metadata
            .get("tower_signature_alg")
            .map(String::as_str),
        Some("ed25519"),
        "entry should have tower_signature_alg=ed25519"
    );
    assert!(
        !entry.metadata["tower_signature"].is_empty(),
        "signature should be non-empty base64"
    );

    mock_handle.abort();
}

#[tokio::test]
async fn test_tower_signed_session_commit() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (socket_path, mock_handle) =
        spawn_mock_tower_signer(tmp.path(), "mock-beardog-session.sock").await;
    let service = tower_service(socket_path);

    let owner = Did::new("did:key:z6MkSessionOwner");
    let create_resp = service
        .create_spine(CreateSpineRequest {
            name: "session-signed-spine".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create should succeed");

    let commit_resp = service
        .commit_session(CommitSessionRequest {
            spine_id: create_resp.spine_id,
            session_id: uuid::Uuid::now_v7(),
            session_hash: [2u8; 32],
            vertex_count: 50,
            committer: owner,
        })
        .await
        .expect("commit_session should succeed");

    let entry_resp = service
        .get_entry(GetEntryRequest {
            spine_id: create_resp.spine_id,
            entry_hash: commit_resp.commit_hash,
        })
        .await
        .expect("get_entry should succeed");

    assert!(entry_resp.found);
    let entry = entry_resp.entry.expect("entry present");
    assert!(
        entry.metadata.contains_key("tower_signature"),
        "session commit entry should have tower_signature"
    );
    assert_eq!(
        entry
            .metadata
            .get("tower_signature_alg")
            .map(String::as_str),
        Some("ed25519")
    );
    assert!(commit_resp.committed_at.as_nanos() > 0);

    mock_handle.abort();
}

#[tokio::test]
async fn test_unsigned_entry_when_no_tower_signer() {
    let service = LoamSpineRpcService::default_service();

    let create_resp = service
        .create_spine(CreateSpineRequest {
            name: "unsigned-spine".to_string(),
            owner: Did::new("did:key:z6MkOwner"),
            config: None,
        })
        .await
        .expect("create should succeed");

    let append_resp = service
        .append_entry(AppendEntryRequest {
            spine_id: create_resp.spine_id,
            entry_type: loam_spine_core::entry::EntryType::MetadataUpdate {
                field: "test".to_string(),
                value: "unsigned-value".to_string(),
            },
            committer: Did::new("did:key:z6MkOwner"),
            payload: None,
        })
        .await
        .expect("append should succeed");

    let entry_resp = service
        .get_entry(GetEntryRequest {
            spine_id: create_resp.spine_id,
            entry_hash: append_resp.entry_hash,
        })
        .await
        .expect("get_entry should succeed");

    assert!(entry_resp.found);
    let entry = entry_resp.entry.expect("entry present");
    assert!(
        !entry.metadata.contains_key("tower_signature"),
        "unsigned entry should NOT have tower_signature metadata"
    );
}
