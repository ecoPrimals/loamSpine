// SPDX-License-Identifier: AGPL-3.0-or-later

//! JSON-RPC integration tests for the `bonding.ledger.*` wire contract.

use super::*;
use crate::types::{
    BondLedgerListRequest, BondLedgerListResponse, BondLedgerRetrieveRequest,
    BondLedgerRetrieveResponse, BondLedgerStoreRequest, BondLedgerStoreResponse,
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

#[tokio::test]
async fn bonding_ledger_store_returns_stored() {
    let server = LoamSpineJsonRpc::default_server();
    let req = BondLedgerStoreRequest {
        bond_id: "ionic-001".into(),
        data: serde_json::json!({
            "proposer": "tower_a",
            "acceptor": "tower_b",
            "state": "sealed",
            "terms_hash": "deadbeef",
        }),
    };

    let resp: BondLedgerStoreResponse = rpc_call(&server, "bonding.ledger.store", &req)
        .await
        .unwrap();
    assert_eq!(resp.status, "stored");
}

#[tokio::test]
async fn bonding_ledger_retrieve_after_store() {
    let server = LoamSpineJsonRpc::default_server();
    let data = serde_json::json!({
        "bond_id": "ionic-002",
        "trust_model": "dual_tower_enclave",
    });

    let store_req = BondLedgerStoreRequest {
        bond_id: "ionic-002".into(),
        data: data.clone(),
    };
    rpc_call::<_, BondLedgerStoreResponse>(&server, "bonding.ledger.store", &store_req)
        .await
        .unwrap();

    let retrieve_req = BondLedgerRetrieveRequest {
        bond_id: "ionic-002".into(),
    };
    let resp: BondLedgerRetrieveResponse =
        rpc_call(&server, "bonding.ledger.retrieve", &retrieve_req)
            .await
            .unwrap();
    assert_eq!(resp.data, Some(data));
}

#[tokio::test]
async fn bonding_ledger_retrieve_nonexistent_returns_none() {
    let server = LoamSpineJsonRpc::default_server();
    let req = BondLedgerRetrieveRequest {
        bond_id: "no-such-bond".into(),
    };
    let resp: BondLedgerRetrieveResponse = rpc_call(&server, "bonding.ledger.retrieve", &req)
        .await
        .unwrap();
    assert!(resp.data.is_none());
}

#[tokio::test]
async fn bonding_ledger_list_empty() {
    let server = LoamSpineJsonRpc::default_server();
    let req = BondLedgerListRequest {};
    let resp: BondLedgerListResponse = rpc_call(&server, "bonding.ledger.list", &req)
        .await
        .unwrap();
    assert!(resp.bonds.is_empty());
}

#[tokio::test]
async fn bonding_ledger_list_after_multiple_stores() {
    let server = LoamSpineJsonRpc::default_server();

    for id in &["bond-a", "bond-b", "bond-c"] {
        let req = BondLedgerStoreRequest {
            bond_id: (*id).to_string(),
            data: serde_json::json!({"id": id}),
        };
        rpc_call::<_, BondLedgerStoreResponse>(&server, "bonding.ledger.store", &req)
            .await
            .unwrap();
    }

    let req = BondLedgerListRequest {};
    let resp: BondLedgerListResponse = rpc_call(&server, "bonding.ledger.list", &req)
        .await
        .unwrap();

    let mut bonds = resp.bonds;
    bonds.sort();
    assert_eq!(bonds, vec!["bond-a", "bond-b", "bond-c"]);
}

#[tokio::test]
async fn bonding_ledger_store_overwrite_preserves_latest() {
    let server = LoamSpineJsonRpc::default_server();
    let v1 = serde_json::json!({"state": "active"});
    let v2 = serde_json::json!({"state": "sealed"});

    let req1 = BondLedgerStoreRequest {
        bond_id: "overwrite-test".into(),
        data: v1,
    };
    rpc_call::<_, BondLedgerStoreResponse>(&server, "bonding.ledger.store", &req1)
        .await
        .unwrap();

    let req2 = BondLedgerStoreRequest {
        bond_id: "overwrite-test".into(),
        data: v2.clone(),
    };
    rpc_call::<_, BondLedgerStoreResponse>(&server, "bonding.ledger.store", &req2)
        .await
        .unwrap();

    let retrieve_req = BondLedgerRetrieveRequest {
        bond_id: "overwrite-test".into(),
    };
    let resp: BondLedgerRetrieveResponse =
        rpc_call(&server, "bonding.ledger.retrieve", &retrieve_req)
            .await
            .unwrap();
    assert_eq!(resp.data, Some(v2));
}

#[tokio::test]
async fn bonding_ledger_stores_realistic_ionic_bond() {
    let server = LoamSpineJsonRpc::default_server();

    let ionic_bond = serde_json::json!({
        "bond_id": "ib-7f3c9a",
        "proposal_id": "prop-001",
        "terms_hash": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
        "proposer": "tower_crypto",
        "acceptor": "tower_compute",
        "trust_model": "dual_tower_enclave",
        "encryption_tier": "aead",
        "state": "sealed",
        "allowed_capabilities": ["crypto.sign", "crypto.verify"],
        "proposer_signature": "abcdef0123456789",
        "proposer_public_key": "deadbeef01234567",
        "acceptor_signature": "fedcba9876543210",
        "acceptor_public_key": "beefdead76543210",
        "created_at": "2026-04-15T12:00:00Z",
        "expires_at": null,
    });

    let store_req = BondLedgerStoreRequest {
        bond_id: "ib-7f3c9a".into(),
        data: ionic_bond.clone(),
    };
    let store_resp: BondLedgerStoreResponse = rpc_call(&server, "bonding.ledger.store", &store_req)
        .await
        .unwrap();
    assert_eq!(store_resp.status, "stored");

    let retrieve_req = BondLedgerRetrieveRequest {
        bond_id: "ib-7f3c9a".into(),
    };
    let resp: BondLedgerRetrieveResponse =
        rpc_call(&server, "bonding.ledger.retrieve", &retrieve_req)
            .await
            .unwrap();
    let data = resp.data.unwrap();
    assert_eq!(data["state"], "sealed");
    assert_eq!(data["proposer"], "tower_crypto");
    assert_eq!(data["acceptor"], "tower_compute");
    assert_eq!(data["trust_model"], "dual_tower_enclave");
}

#[tokio::test]
async fn bonding_ledger_dispatch_typed_ok() {
    let server = LoamSpineJsonRpc::default_server();
    let params = serde_json::json!({
        "bond_id": "dispatch-test",
        "data": {"x": 42},
    });

    let outcome = server.dispatch_typed("bonding.ledger.store", params).await;
    assert!(matches!(
        outcome,
        loam_spine_core::error::DispatchOutcome::Ok(_)
    ));
}

#[tokio::test]
async fn bonding_ledger_store_missing_bond_id_returns_invalid_params() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "bonding.ledger.store".to_string(),
        params: serde_json::json!({"data": {}}),
        id: serde_json::Value::Number(1.into()),
    };
    let rpc_resp = server.handle_request(rpc_req).await;
    assert!(rpc_resp.error.is_some());
    let err = rpc_resp.error.unwrap();
    assert_eq!(err.code, -32602);
}
