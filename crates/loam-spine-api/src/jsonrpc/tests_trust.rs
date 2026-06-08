// SPDX-License-Identifier: AGPL-3.0-or-later

//! JSON-RPC integration tests for the `trust.*` wire contract.

use super::*;
use crate::types::{
    TrustAnchorRequest, TrustAnchorResponse, TrustEventCountRequest, TrustEventCountResponse,
    TrustQueryRequest, TrustQueryResponse,
};
use loam_spine_core::entry::EntryType;
use loam_spine_core::types::Did;

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

#[tokio::test]
async fn trust_anchor_key_exchange() {
    let server = LoamSpineJsonRpc::default_server();
    let req = TrustAnchorRequest {
        entry_type: EntryType::KeyExchange {
            local_gate: Did::new("did:key:z6MkStrandGate"),
            remote_gate: Did::new("did:key:z6MkBiomeGate"),
            public_key_hash: [0xab; 32],
            direction: "initiated".into(),
            family_id: Some("alpha".into()),
        },
    };

    let resp: TrustAnchorResponse = rpc_call(&server, "trust.anchor", &req).await.unwrap();
    assert_ne!(resp.entry_hash, [0u8; 32]);
    assert_eq!(resp.index, 1);
}

#[tokio::test]
async fn trust_anchor_issuer_registration() {
    let server = LoamSpineJsonRpc::default_server();
    let req = TrustAnchorRequest {
        entry_type: EntryType::TrustIssuerRegistration {
            issuer_did: Did::new("did:key:z6MkBearDogIssuer"),
            registering_gate: Did::new("did:key:z6MkSouthGate"),
            trust_scope: "cross-gate".into(),
            capabilities: vec!["signing".into(), "verification".into()],
            expires_at: None,
        },
    };

    let resp: TrustAnchorResponse = rpc_call(&server, "trust.anchor", &req).await.unwrap();
    assert_eq!(resp.index, 1);
}

#[tokio::test]
async fn trust_anchor_token_verification() {
    let server = LoamSpineJsonRpc::default_server();
    let req = TrustAnchorRequest {
        entry_type: EntryType::TokenVerificationCrossGate {
            issuer_gate: Did::new("did:key:z6MkBiomeGate"),
            verifier_gate: Did::new("did:key:z6MkStrandGate"),
            token_hash: [0xcd; 32],
            verified: true,
            failure_reason: None,
        },
    };

    let resp: TrustAnchorResponse = rpc_call(&server, "trust.anchor", &req).await.unwrap();
    assert_eq!(resp.index, 1);
}

#[tokio::test]
async fn trust_anchor_rejects_non_trust_type() {
    let server = LoamSpineJsonRpc::default_server();
    let req = TrustAnchorRequest {
        entry_type: EntryType::MetadataUpdate {
            field: "name".into(),
            value: "sneaky".into(),
        },
    };

    let err = rpc_call::<_, TrustAnchorResponse>(&server, "trust.anchor", &req)
        .await
        .unwrap_err();
    assert!(err.contains("trust-domain"));
}

#[tokio::test]
async fn trust_event_count_starts_at_zero() {
    let server = LoamSpineJsonRpc::default_server();
    let resp: TrustEventCountResponse =
        rpc_call(&server, "trust.event_count", &TrustEventCountRequest {})
            .await
            .unwrap();
    assert_eq!(resp.count, 0);
}

#[tokio::test]
async fn trust_event_count_increments() {
    let server = LoamSpineJsonRpc::default_server();

    let anchor_req = TrustAnchorRequest {
        entry_type: EntryType::KeyExchange {
            local_gate: Did::new("did:key:z6MkA"),
            remote_gate: Did::new("did:key:z6MkB"),
            public_key_hash: [1; 32],
            direction: "initiated".into(),
            family_id: None,
        },
    };
    let _: TrustAnchorResponse = rpc_call(&server, "trust.anchor", &anchor_req)
        .await
        .unwrap();

    let resp: TrustEventCountResponse =
        rpc_call(&server, "trust.event_count", &TrustEventCountRequest {})
            .await
            .unwrap();
    assert_eq!(resp.count, 1);
}

#[tokio::test]
async fn trust_query_returns_matching_events() {
    let server = LoamSpineJsonRpc::default_server();
    let gate_a = Did::new("did:key:z6MkStrandGate");

    let anchor_req = TrustAnchorRequest {
        entry_type: EntryType::TrustIssuerRegistration {
            issuer_did: Did::new("did:key:z6MkIssuerX"),
            registering_gate: gate_a.clone(),
            trust_scope: "family".into(),
            capabilities: vec!["signing".into()],
            expires_at: None,
        },
    };
    let _: TrustAnchorResponse = rpc_call(&server, "trust.anchor", &anchor_req)
        .await
        .unwrap();

    let query_req = TrustQueryRequest { gate_did: gate_a };
    let resp: TrustQueryResponse = rpc_call(&server, "trust.query", &query_req).await.unwrap();
    assert_eq!(resp.events.len(), 1);
}

#[tokio::test]
async fn trust_query_empty_for_unknown_gate() {
    let server = LoamSpineJsonRpc::default_server();

    let query_req = TrustQueryRequest {
        gate_did: Did::new("did:key:z6MkNobody"),
    };
    let resp: TrustQueryResponse = rpc_call(&server, "trust.query", &query_req).await.unwrap();
    assert!(resp.events.is_empty());
}

#[tokio::test]
async fn trust_anchor_roundtrip_serde() {
    let server = LoamSpineJsonRpc::default_server();
    let original = EntryType::TrustIssuerRegistration {
        issuer_did: Did::new("did:key:z6MkBearDogAuth"),
        registering_gate: Did::new("did:key:z6MkSouthGate"),
        trust_scope: "cross-gate".into(),
        capabilities: vec!["signing".into(), "verification".into()],
        expires_at: None,
    };

    let anchor_req = TrustAnchorRequest {
        entry_type: original.clone(),
    };
    let _: TrustAnchorResponse = rpc_call(&server, "trust.anchor", &anchor_req)
        .await
        .unwrap();

    let query_req = TrustQueryRequest {
        gate_did: Did::new("did:key:z6MkSouthGate"),
    };
    let resp: TrustQueryResponse = rpc_call(&server, "trust.query", &query_req).await.unwrap();
    assert_eq!(resp.events.len(), 1);

    let json_original = serde_json::to_value(&original).unwrap();
    let json_retrieved = serde_json::to_value(&resp.events[0]).unwrap();
    assert_eq!(json_original, json_retrieved);
}
