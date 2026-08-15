// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for rootPulse graph step handlers (`rootpulse.ledger_commit`,
//! `rootpulse.query_commit`) and the `ledger.append` wire alias.

use super::*;
use crate::types::{
    RootpulseLedgerCommitRequest, RootpulseLedgerCommitResponse, RootpulseQueryCommitRequest,
    RootpulseQueryCommitResponse,
};
use tests::rpc_call;

fn valid_cas_ref() -> String {
    "a".repeat(64)
}

fn signed_provenance_with_sig() -> serde_json::Value {
    serde_json::json!({
        "signature": "base64-ed25519-sig-placeholder",
        "payload": { "session": "test" }
    })
}

#[tokio::test]
async fn ledger_commit_basic() {
    let server = LoamSpineJsonRpc::default_server();

    let request = RootpulseLedgerCommitRequest {
        cas_ref: valid_cas_ref(),
        signed_provenance: signed_provenance_with_sig(),
        content_hash: None,
        session_id: None,
        spine_id: None,
        committer: None,
    };

    let response: RootpulseLedgerCommitResponse =
        rpc_call(&server, "rootpulse.ledger_commit", &request)
            .await
            .unwrap();

    assert!(!response.ledger_ref.is_empty());
    assert_eq!(response.index, 1, "first commit is index 1 (genesis is 0)");
    assert!(response.committed_at > loam_spine_core::types::Timestamp(0));

    let parts: Vec<&str> = response.ledger_ref.split(':').collect();
    assert_eq!(parts.len(), 3, "ledger_ref should be spine_id:hash:index");
    assert_eq!(parts[2], "1");
}

#[tokio::test]
async fn ledger_commit_with_explicit_committer() {
    let server = LoamSpineJsonRpc::default_server();

    let request = RootpulseLedgerCommitRequest {
        cas_ref: valid_cas_ref(),
        signed_provenance: serde_json::Value::Null,
        content_hash: Some("b".repeat(64)),
        session_id: None,
        spine_id: None,
        committer: Some("did:key:z6MkTestCommitter".into()),
    };

    let response: RootpulseLedgerCommitResponse =
        rpc_call(&server, "rootpulse.ledger_commit", &request)
            .await
            .unwrap();

    assert_eq!(response.index, 1);
}

#[tokio::test]
async fn ledger_commit_with_explicit_session_id() {
    let server = LoamSpineJsonRpc::default_server();

    let session_id = uuid::Uuid::now_v7().to_string();
    let request = RootpulseLedgerCommitRequest {
        cas_ref: valid_cas_ref(),
        signed_provenance: serde_json::Value::Null,
        content_hash: None,
        session_id: Some(session_id),
        spine_id: None,
        committer: None,
    };

    let response: RootpulseLedgerCommitResponse =
        rpc_call(&server, "rootpulse.ledger_commit", &request)
            .await
            .unwrap();

    assert_eq!(response.index, 1);
}

#[tokio::test]
async fn ledger_commit_invalid_cas_ref_rejects() {
    let server = LoamSpineJsonRpc::default_server();

    let request = RootpulseLedgerCommitRequest {
        cas_ref: "too-short".into(),
        signed_provenance: serde_json::Value::Null,
        content_hash: None,
        session_id: None,
        spine_id: None,
        committer: None,
    };

    let result: Result<RootpulseLedgerCommitResponse, _> =
        rpc_call(&server, "rootpulse.ledger_commit", &request).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cas_ref"));
}

#[tokio::test]
async fn ledger_commit_invalid_spine_id_rejects() {
    let server = LoamSpineJsonRpc::default_server();

    let request = RootpulseLedgerCommitRequest {
        cas_ref: valid_cas_ref(),
        signed_provenance: serde_json::Value::Null,
        content_hash: None,
        session_id: None,
        spine_id: Some("not-a-uuid".into()),
        committer: None,
    };

    let result: Result<RootpulseLedgerCommitResponse, _> =
        rpc_call(&server, "rootpulse.ledger_commit", &request).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("spine_id"));
}

#[tokio::test]
async fn ledger_commit_invalid_session_id_rejects() {
    let server = LoamSpineJsonRpc::default_server();

    let request = RootpulseLedgerCommitRequest {
        cas_ref: valid_cas_ref(),
        signed_provenance: serde_json::Value::Null,
        content_hash: None,
        session_id: Some("not-a-uuid".into()),
        spine_id: None,
        committer: None,
    };

    let result: Result<RootpulseLedgerCommitResponse, _> =
        rpc_call(&server, "rootpulse.ledger_commit", &request).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("session_id"));
}

#[tokio::test]
async fn ledger_append_alias_resolves() {
    let server = LoamSpineJsonRpc::default_server();

    let request = RootpulseLedgerCommitRequest {
        cas_ref: valid_cas_ref(),
        signed_provenance: serde_json::Value::Null,
        content_hash: None,
        session_id: None,
        spine_id: None,
        committer: None,
    };

    let response: RootpulseLedgerCommitResponse =
        rpc_call(&server, "ledger.append", &request).await.unwrap();

    assert!(!response.ledger_ref.is_empty());
}

#[tokio::test]
async fn query_commit_empty_spine() {
    let server = LoamSpineJsonRpc::default_server();

    let request = RootpulseQueryCommitRequest {
        wave_id: None,
        target_triple: None,
        primal_name: None,
        limit: None,
    };

    let response: RootpulseQueryCommitResponse =
        rpc_call(&server, "rootpulse.query_commit", &request)
            .await
            .unwrap();

    assert!(response.commits.is_empty());
    assert_eq!(response.total, 0);
}

#[tokio::test]
async fn query_commit_after_ledger_commit() {
    let server = LoamSpineJsonRpc::default_server();

    let commit_request = RootpulseLedgerCommitRequest {
        cas_ref: valid_cas_ref(),
        signed_provenance: signed_provenance_with_sig(),
        content_hash: Some("c".repeat(64)),
        session_id: None,
        spine_id: None,
        committer: None,
    };

    let commit_response: RootpulseLedgerCommitResponse =
        rpc_call(&server, "rootpulse.ledger_commit", &commit_request)
            .await
            .unwrap();

    let query_request = RootpulseQueryCommitRequest {
        wave_id: None,
        target_triple: None,
        primal_name: None,
        limit: None,
    };

    let query_response: RootpulseQueryCommitResponse =
        rpc_call(&server, "rootpulse.query_commit", &query_request)
            .await
            .unwrap();

    assert_eq!(query_response.total, 1);
    assert_eq!(query_response.commits.len(), 1);

    let entry = &query_response.commits[0];
    assert_eq!(entry.ledger_ref, commit_response.ledger_ref);
    assert_eq!(entry.spine_id, commit_response.spine_id);
    assert_eq!(entry.index, commit_response.index);
}

#[tokio::test]
async fn query_commit_respects_limit() {
    let server = LoamSpineJsonRpc::default_server();

    for _ in 0..5 {
        let request = RootpulseLedgerCommitRequest {
            cas_ref: valid_cas_ref(),
            signed_provenance: serde_json::Value::Null,
            content_hash: None,
            session_id: None,
            spine_id: None,
            committer: None,
        };
        let _: RootpulseLedgerCommitResponse =
            rpc_call(&server, "rootpulse.ledger_commit", &request)
                .await
                .unwrap();
    }

    let query_request = RootpulseQueryCommitRequest {
        wave_id: None,
        target_triple: None,
        primal_name: None,
        limit: Some(2),
    };

    let response: RootpulseQueryCommitResponse =
        rpc_call(&server, "rootpulse.query_commit", &query_request)
            .await
            .unwrap();

    assert_eq!(response.commits.len(), 2);
}

#[tokio::test]
async fn multiple_commits_increment_index() {
    let server = LoamSpineJsonRpc::default_server();

    let mut indices = Vec::new();
    for _ in 0..3 {
        let request = RootpulseLedgerCommitRequest {
            cas_ref: valid_cas_ref(),
            signed_provenance: serde_json::Value::Null,
            content_hash: None,
            session_id: None,
            spine_id: None,
            committer: None,
        };
        let response: RootpulseLedgerCommitResponse =
            rpc_call(&server, "rootpulse.ledger_commit", &request)
                .await
                .unwrap();
        indices.push(response.index);
    }

    assert_eq!(indices, vec![1, 2, 3]);
}

#[tokio::test]
async fn normalize_method_ledger_append() {
    assert_eq!(normalize_method("ledger.append"), "rootpulse.ledger_commit");
}

#[tokio::test]
async fn rootpulse_methods_in_niche() {
    use loam_spine_core::niche::METHODS;
    assert!(METHODS.contains(&"rootpulse.ledger_commit"));
    assert!(METHODS.contains(&"rootpulse.query_commit"));
}

#[tokio::test]
async fn rootpulse_domain_in_niche() {
    use loam_spine_core::niche::DOMAINS;
    assert!(DOMAINS.contains(&"rootpulse"));
}
