// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::types::{
    AppendEntryRequest, CreateSpineRequest, GetCertificateRequest, GetEntryRequest,
    GetSpineRequest, GetTipRequest, MintCertificateRequest, SealSpineRequest,
};
use crate::types::{CertificateType, Did, EntryType};
use tests::{rpc_call, rpc_call_no_params};

#[tokio::test]
async fn test_jsonrpc_create_spine() {
    let server = LoamSpineJsonRpc::default_server();
    let request = CreateSpineRequest {
        owner: Did::new("did:key:z6MkTest"),
        name: "Test Spine".to_string(),
        config: None,
    };

    let response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &request).await.unwrap();
    assert!(!response.spine_id.is_nil());
}

#[tokio::test]
async fn test_jsonrpc_get_nonexistent_spine() {
    let server = LoamSpineJsonRpc::default_server();
    let request = GetSpineRequest {
        spine_id: uuid::Uuid::nil(),
    };

    let response: crate::types::GetSpineResponse =
        rpc_call(&server, "spine.get", &request).await.unwrap();
    assert!(response.spine.is_none());
}

#[tokio::test]
async fn test_jsonrpc_seal_spine() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Test".to_string(),
        config: None,
    };
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

    let seal_request = SealSpineRequest {
        spine_id: create_response.spine_id,
        sealer: owner,
        reason: None,
    };

    let result: Result<crate::types::SealSpineResponse, _> =
        rpc_call(&server, "spine.seal", &seal_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jsonrpc_mint_and_get_certificate() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Cert Test".to_string(),
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
            game_id: "hl3".to_string(),
            edition: None,
        },
        owner: owner.clone(),
        metadata: None,
    };

    let mint_response: crate::types::MintCertificateResponse =
        rpc_call(&server, "certificate.mint", &mint_request)
            .await
            .unwrap();

    let get_request = GetCertificateRequest {
        certificate_id: mint_response.certificate_id,
    };

    let result: Result<crate::types::GetCertificateResponse, _> =
        rpc_call(&server, "certificate.get", &get_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jsonrpc_append_entry() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Entry Test".to_string(),
        config: None,
    };
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

    let append_request = AppendEntryRequest {
        spine_id: create_response.spine_id,
        entry_type: EntryType::DataAnchor {
            data_hash: [1u8; 32],
            mime_type: Some("text/plain".to_string()),
            size: 50,
        },
        committer: Some(owner.clone()),
        payload: None,
    };
    let response: crate::types::AppendEntryResponse =
        rpc_call(&server, "entry.append", &append_request)
            .await
            .unwrap();
    assert!(!response.entry_hash.iter().all(|&b| b == 0));
}

#[tokio::test]
async fn test_jsonrpc_get_entry_and_tip() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Get Entry Test".to_string(),
        config: None,
    };
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

    let append_request = AppendEntryRequest {
        spine_id: create_response.spine_id,
        entry_type: EntryType::DataAnchor {
            data_hash: [2u8; 32],
            mime_type: Some("text/plain".to_string()),
            size: 10,
        },
        committer: Some(owner.clone()),
        payload: None,
    };
    let append_response: crate::types::AppendEntryResponse =
        rpc_call(&server, "entry.append", &append_request)
            .await
            .unwrap();

    let get_entry_request = GetEntryRequest {
        spine_id: create_response.spine_id,
        entry_hash: append_response.entry_hash,
    };
    let response: crate::types::GetEntryResponse =
        rpc_call(&server, "entry.get", &get_entry_request)
            .await
            .unwrap();
    assert!(response.found);

    let get_tip_request = GetTipRequest {
        spine_id: create_response.spine_id,
    };
    let result: Result<crate::types::GetTipResponse, _> =
        rpc_call(&server, "entry.get_tip", &get_tip_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jsonrpc_anchor_slice() {
    use crate::types::AnchorSliceRequest;

    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let waypoint_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Waypoint".to_string(),
        config: None,
    };
    let waypoint_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &waypoint_request)
            .await
            .unwrap();

    let origin_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Origin".to_string(),
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
        committer: owner,
    };
    let result: Result<crate::types::AnchorSliceResponse, _> =
        rpc_call(&server, "slice.anchor", &anchor_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jsonrpc_liveness_and_readiness() {
    let server = LoamSpineJsonRpc::default_server();

    let liveness: crate::health::LivenessProbe = rpc_call_no_params(&server, "health.liveness")
        .await
        .unwrap();
    assert_eq!(liveness.status, "alive");

    let readiness: crate::health::ReadinessProbe = rpc_call_no_params(&server, "health.readiness")
        .await
        .unwrap();
    assert!(readiness.ready);
}
