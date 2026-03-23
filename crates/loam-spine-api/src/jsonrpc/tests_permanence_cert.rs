// SPDX-License-Identifier: AGPL-3.0-or-later

//! JSON-RPC tests: permanence operations, certificate transfer/loan lifecycle,
//! slice checkout flows, and legacy method aliases.
//!
//! Split from `tests.rs` by domain: core CRUD and protocol tests stay there,
//! permanence + certificate lifecycle + legacy aliases live here.

use super::*;
use crate::types::{
    AnchorSliceRequest, CertificateType, CheckoutSliceRequest, CommitSessionRequest,
    CreateSpineRequest, Did, LoanCertificateRequest, LoanTerms, MintCertificateRequest,
    PermanentStorageCommitRequest, PermanentStorageGetCommitRequest, PermanentStorageVerifyRequest,
    ReturnCertificateRequest, TransferCertificateRequest,
};

async fn rpc_call<Req: serde::Serialize + Sync, Resp: serde::de::DeserializeOwned>(
    server: &LoamSpineJsonRpc,
    method: &str,
    request: &Req,
) -> Result<Resp, String> {
    let params = serde_json::to_value(request).unwrap();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id: serde_json::Value::Number(1.into()),
    };
    let rpc_resp = server.handle_request(rpc_req).await;
    if let Some(err) = rpc_resp.error {
        return Err(err.message);
    }
    serde_json::from_value(rpc_resp.result.unwrap_or_default()).map_err(|e| e.to_string())
}

async fn rpc_call_no_params<Resp: serde::de::DeserializeOwned>(
    server: &LoamSpineJsonRpc,
    method: &str,
) -> Result<Resp, String> {
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Number(1.into()),
    };
    let rpc_resp = server.handle_request(rpc_req).await;
    if let Some(err) = rpc_resp.error {
        return Err(err.message);
    }
    serde_json::from_value(rpc_resp.result.unwrap_or_default()).map_err(|e| e.to_string())
}

// ========================================================================
// Permanence operations
// ========================================================================

#[tokio::test]
async fn test_jsonrpc_permanence_commit_and_verify() {
    use crate::types::PermanentStorageDehydrationSummary;

    let server = LoamSpineJsonRpc::default_server();

    let commit_request = PermanentStorageCommitRequest {
        session_id: uuid::Uuid::now_v7().to_string(),
        merkle_root: "ab".repeat(32),
        committer_did: Some("did:key:z6MkTest".to_string()),
        summary: PermanentStorageDehydrationSummary {
            session_type: "test".to_string(),
            vertex_count: 10,
            leaf_count: 5,
            started_at: 0,
            ended_at: 1,
            outcome: "success".to_string(),
        },
    };

    let response: crate::types::PermanentStorageCommitResponse =
        rpc_call(&server, "permanence.commit_session", &commit_request)
            .await
            .unwrap();
    assert!(response.accepted);
    assert!(response.commit_id.is_some());
    assert!(response.spine_id.is_some());

    let spine_id_str = response.spine_id.clone().unwrap();
    let entry_hash_str = response.spine_entry_hash.clone().unwrap();
    let index = response.entry_index.unwrap_or(0);

    let verify_request = PermanentStorageVerifyRequest {
        spine_id: spine_id_str.clone(),
        entry_hash: entry_hash_str.clone(),
        index,
    };

    let verified: bool = rpc_call(&server, "permanence.verify_commit", &verify_request)
        .await
        .unwrap();
    assert!(verified);

    let get_request = PermanentStorageGetCommitRequest {
        spine_id: spine_id_str,
        entry_hash: entry_hash_str,
        index,
    };

    let value: serde_json::Value = rpc_call(&server, "permanence.get_commit", &get_request)
        .await
        .unwrap();
    assert!(!value.is_null());
}

#[tokio::test]
async fn test_jsonrpc_permanence_health_check() {
    let server = LoamSpineJsonRpc::default_server();

    let healthy: bool = rpc_call_no_params(&server, "permanence.health_check")
        .await
        .unwrap();
    assert!(healthy);
}

#[tokio::test]
async fn test_jsonrpc_legacy_permanence_delegates() {
    use crate::types::PermanentStorageDehydrationSummary;

    let server = LoamSpineJsonRpc::default_server();

    let commit_request = PermanentStorageCommitRequest {
        session_id: uuid::Uuid::now_v7().to_string(),
        merkle_root: "cd".repeat(32),
        committer_did: Some("did:key:z6MkLegacy".to_string()),
        summary: PermanentStorageDehydrationSummary {
            session_type: "test".to_string(),
            vertex_count: 5,
            leaf_count: 2,
            started_at: 0,
            ended_at: 1,
            outcome: "success".to_string(),
        },
    };

    let response: crate::types::PermanentStorageCommitResponse =
        rpc_call(&server, "permanent-storage.commitSession", &commit_request)
            .await
            .unwrap();
    assert!(response.accepted);

    let healthy: bool = rpc_call_no_params(&server, "permanent-storage.healthCheck")
        .await
        .unwrap();
    assert!(healthy);
}

// ========================================================================
// Certificate transfer and loan lifecycle
// ========================================================================

#[tokio::test]
async fn test_jsonrpc_certificate_transfer() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTransferFrom");
    let buyer = Did::new("did:key:z6MkTransferTo");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Transfer Test".to_string(),
        config: None,
    };
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

    let mint_request = MintCertificateRequest {
        spine_id: create_response.spine_id,
        cert_type: CertificateType::DigitalGame {
            platform: "steam".to_string(),
            game_id: "transfer_test".to_string(),
            edition: None,
        },
        owner: owner.clone(),
        metadata: None,
    };
    let mint_response: crate::types::MintCertificateResponse =
        rpc_call(&server, "certificate.mint", &mint_request)
            .await
            .unwrap();

    let transfer_request = TransferCertificateRequest {
        certificate_id: mint_response.certificate_id,
        from: owner,
        to: buyer,
    };
    let result: Result<crate::types::TransferCertificateResponse, _> =
        rpc_call(&server, "certificate.transfer", &transfer_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jsonrpc_certificate_loan_and_return() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkLoanOwner");
    let borrower = Did::new("did:key:z6MkLoanBorrower");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Loan Test".to_string(),
        config: None,
    };
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

    let mint_request = MintCertificateRequest {
        spine_id: create_response.spine_id,
        cert_type: CertificateType::DigitalGame {
            platform: "epic".to_string(),
            game_id: "loan_test".to_string(),
            edition: None,
        },
        owner: owner.clone(),
        metadata: None,
    };
    let mint_response: crate::types::MintCertificateResponse =
        rpc_call(&server, "certificate.mint", &mint_request)
            .await
            .unwrap();

    let loan_request = LoanCertificateRequest {
        certificate_id: mint_response.certificate_id,
        lender: owner.clone(),
        borrower: borrower.clone(),
        terms: LoanTerms::new(),
    };
    let loan_result: Result<crate::types::LoanCertificateResponse, _> =
        rpc_call(&server, "certificate.loan", &loan_request).await;
    assert!(loan_result.is_ok());

    let return_request = ReturnCertificateRequest {
        certificate_id: mint_response.certificate_id,
        returner: borrower,
    };
    let return_result: Result<crate::types::ReturnCertificateResponse, _> =
        rpc_call(&server, "certificate.return", &return_request).await;
    assert!(return_result.is_ok());
}

// ========================================================================
// Slice checkout flow
// ========================================================================

#[tokio::test]
async fn test_jsonrpc_slice_checkout() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkCheckout");

    let waypoint_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Waypoint Checkout".to_string(),
        config: None,
    };
    let waypoint_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &waypoint_request)
            .await
            .unwrap();

    let origin_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Origin Checkout".to_string(),
        config: None,
    };
    let origin_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &origin_request)
            .await
            .unwrap();

    let anchor_request = AnchorSliceRequest {
        waypoint_spine_id: waypoint_response.spine_id,
        slice_id: uuid::Uuid::now_v7(),
        origin_spine_id: origin_response.spine_id,
        committer: owner.clone(),
    };
    let _anchor_response: crate::types::AnchorSliceResponse =
        rpc_call(&server, "slice.anchor", &anchor_request)
            .await
            .unwrap();

    let checkout_request = CheckoutSliceRequest {
        waypoint_spine_id: waypoint_response.spine_id,
        slice_id: anchor_request.slice_id,
        requester: owner,
    };
    let result: Result<crate::types::CheckoutSliceResponse, _> =
        rpc_call(&server, "slice.checkout", &checkout_request).await;
    assert!(result.is_ok());
}

// ========================================================================
// Semantic aliases and legacy methods
// ========================================================================

#[tokio::test]
async fn semantic_commit_session_alias() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkSemanticTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Semantic Test".to_string(),
        config: None,
    };
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

    let request = CommitSessionRequest {
        spine_id: create_response.spine_id,
        session_id: uuid::Uuid::now_v7(),
        session_hash: [0u8; 32],
        vertex_count: 10,
        committer: owner,
    };
    let result: Result<crate::types::CommitSessionResponse, _> =
        rpc_call(&server, "commit.session", &request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn permanence_legacy_verify_and_get_aliases() {
    use crate::types::PermanentStorageDehydrationSummary;

    let server = LoamSpineJsonRpc::default_server();

    let commit_request = PermanentStorageCommitRequest {
        session_id: uuid::Uuid::now_v7().to_string(),
        merkle_root: "ef".repeat(32),
        committer_did: Some("did:key:z6MkLegacyAlias".to_string()),
        summary: PermanentStorageDehydrationSummary {
            session_type: "test".to_string(),
            vertex_count: 3,
            leaf_count: 1,
            started_at: 0,
            ended_at: 1,
            outcome: "success".to_string(),
        },
    };

    let response: crate::types::PermanentStorageCommitResponse =
        rpc_call(&server, "permanent-storage.commitSession", &commit_request)
            .await
            .unwrap();
    assert!(response.accepted);

    let spine_id_str = response.spine_id.clone().unwrap();
    let entry_hash_str = response.spine_entry_hash.clone().unwrap();
    let index = response.entry_index.unwrap_or(0);

    let verify_request = PermanentStorageVerifyRequest {
        spine_id: spine_id_str.clone(),
        entry_hash: entry_hash_str.clone(),
        index,
    };
    let verified: bool = rpc_call(&server, "permanent-storage.verifyCommit", &verify_request)
        .await
        .unwrap();
    assert!(verified);

    let get_request = PermanentStorageGetCommitRequest {
        spine_id: spine_id_str,
        entry_hash: entry_hash_str,
        index,
    };
    let value: serde_json::Value = rpc_call(&server, "permanent-storage.getCommit", &get_request)
        .await
        .unwrap();
    assert!(!value.is_null());
}
