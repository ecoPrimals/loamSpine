// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::types::{
    AnchorPublishRequest, AnchorPublishResponse, AnchorVerifyRequest, AnchorVerifyResponse,
    AppendEntryRequest, CreateSpineRequest, GenerateInclusionProofRequest,
    VerifyInclusionProofRequest,
};
use crate::types::{Did, EntryType};
use loam_spine_core::entry::AnchorTarget;
use tests::rpc_call;

#[tokio::test]
async fn test_jsonrpc_generate_and_verify_inclusion_proof() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Proof Test".to_string(),
        config: None,
    };
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

    let append_request = AppendEntryRequest {
        spine_id: create_response.spine_id,
        entry_type: EntryType::DataAnchor {
            data_hash: [4u8; 32],
            mime_type: Some("text/plain".to_string()),
            size: 20,
        },
        committer: Some(owner),
        payload: None,
    };
    let append_response: crate::types::AppendEntryResponse =
        rpc_call(&server, "entry.append", &append_request)
            .await
            .unwrap();

    let gen_request = GenerateInclusionProofRequest {
        spine_id: create_response.spine_id,
        entry_hash: append_response.entry_hash,
    };
    let proof: crate::types::GenerateInclusionProofResponse =
        rpc_call(&server, "proof.generate_inclusion", &gen_request)
            .await
            .unwrap();

    let verify_request = VerifyInclusionProofRequest { proof: proof.proof };
    let response: crate::types::VerifyInclusionProofResponse =
        rpc_call(&server, "proof.verify_inclusion", &verify_request)
            .await
            .unwrap();
    assert!(response.valid);
}

#[tokio::test]
async fn anchor_publish_and_verify_dispatch() {
    let server = LoamSpineJsonRpc::default_server();

    let create_resp: crate::types::CreateSpineResponse = rpc_call(
        &server,
        "spine.create",
        &CreateSpineRequest {
            name: "anchor-dispatch-test".into(),
            owner: Did::new("did:key:z6MkAnchor"),
            config: None,
        },
    )
    .await
    .unwrap();

    let publish_resp: AnchorPublishResponse = rpc_call(
        &server,
        "anchor.publish",
        &AnchorPublishRequest {
            spine_id: create_resp.spine_id,
            anchor_target: AnchorTarget::DataCommons {
                commons_id: "test-commons".into(),
            },
            tx_ref: "bafytest123".into(),
            block_height: 0,
            anchor_timestamp: loam_spine_core::types::Timestamp::now(),
        },
    )
    .await
    .unwrap();

    assert_ne!(publish_resp.entry_hash, [0u8; 32]);
    assert_ne!(publish_resp.state_hash, [0u8; 32]);

    let verify_resp: AnchorVerifyResponse = rpc_call(
        &server,
        "anchor.verify",
        &AnchorVerifyRequest {
            spine_id: create_resp.spine_id,
            anchor_entry_hash: Some(publish_resp.entry_hash),
        },
    )
    .await
    .unwrap();

    assert!(verify_resp.verified);
    assert_eq!(verify_resp.tx_ref, "bafytest123");
    assert_eq!(verify_resp.state_hash, publish_resp.state_hash);
}

#[tokio::test]
async fn anchor_verify_latest_dispatch() {
    let server = LoamSpineJsonRpc::default_server();

    let create_resp: crate::types::CreateSpineResponse = rpc_call(
        &server,
        "spine.create",
        &CreateSpineRequest {
            name: "anchor-latest-test".into(),
            owner: Did::new("did:key:z6MkLatest"),
            config: None,
        },
    )
    .await
    .unwrap();

    let _: crate::types::AnchorPublishResponse = rpc_call(
        &server,
        "anchor.publish",
        &AnchorPublishRequest {
            spine_id: create_resp.spine_id,
            anchor_target: AnchorTarget::Ethereum,
            tx_ref: "0xdeadbeef".into(),
            block_height: 42,
            anchor_timestamp: loam_spine_core::types::Timestamp::now(),
        },
    )
    .await
    .unwrap();

    let verify_resp: AnchorVerifyResponse = rpc_call(
        &server,
        "anchor.verify",
        &AnchorVerifyRequest {
            spine_id: create_resp.spine_id,
            anchor_entry_hash: None,
        },
    )
    .await
    .unwrap();

    assert!(verify_resp.verified);
    assert_eq!(verify_resp.tx_ref, "0xdeadbeef");
    assert_eq!(verify_resp.block_height, 42);
}
