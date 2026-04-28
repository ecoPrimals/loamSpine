// SPDX-License-Identifier: AGPL-3.0-or-later

//! PG-52: UDS trio lifecycle tests — `spine.create` / `entry.append` / `spine.seal`.
//!
//! Verifies the full create → append → seal cycle over UDS JSON-RPC,
//! covering persistent connections, BTSP config coexistence, and
//! one-shot connection patterns (socat/nc composition scripts).

use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Send a JSON-RPC request over a UDS stream and return the parsed response.
#[cfg(unix)]
async fn uds_rpc(stream: &mut tokio::net::UnixStream, request: &str) -> serde_json::Value {
    stream
        .write_all(request.as_bytes())
        .await
        .expect("write request");
    stream.write_all(b"\n").await.expect("write newline");
    stream.flush().await.expect("flush");

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await.expect("read response");
    assert!(n > 0, "empty response from server");
    let text = std::str::from_utf8(&buf[..n]).expect("utf8");
    serde_json::from_str(text.trim()).expect("parse JSON")
}

#[cfg(unix)]
#[tokio::test]
async fn uds_trio_lifecycle_create_append_seal() {
    let tmp = tempfile::tempdir().unwrap();
    let sock_path = tmp.path().join("trio-lifecycle.sock");
    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_uds_server(&sock_path, service, None)
        .await
        .unwrap();

    let mut stream = tokio::net::UnixStream::connect(&sock_path).await.unwrap();

    // spine.create
    let create_resp = uds_rpc(
        &mut stream,
        r#"{"jsonrpc":"2.0","method":"spine.create","params":{"name":"pg52-test","owner":"did:key:z6MkPG52"},"id":1}"#,
    )
    .await;

    assert_eq!(create_resp["jsonrpc"], "2.0");
    assert_eq!(create_resp["id"], 1);
    assert!(
        create_resp["result"]["spine_id"].is_string(),
        "spine.create must return spine_id string: {create_resp}"
    );
    let spine_id = create_resp["result"]["spine_id"].as_str().unwrap();

    // entry.append (DataAnchor variant with required struct fields)
    let append_req = format!(
        r#"{{"jsonrpc":"2.0","method":"entry.append","params":{{"spine_id":"{spine_id}","entry_type":{{"DataAnchor":{{"data_hash":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],"mime_type":"application/octet-stream","size":1024}}}},"committer":"did:key:z6MkPG52"}},"id":2}}"#,
    );
    let append_resp = uds_rpc(&mut stream, &append_req).await;

    assert_eq!(append_resp["jsonrpc"], "2.0");
    assert_eq!(append_resp["id"], 2);
    assert!(
        append_resp["result"]["entry_hash"].is_array(),
        "entry.append must return entry_hash: {append_resp}"
    );
    assert!(
        append_resp["result"]["index"].is_number(),
        "entry.append must return index: {append_resp}"
    );

    // spine.seal
    let seal_req = format!(
        r#"{{"jsonrpc":"2.0","method":"spine.seal","params":{{"spine_id":"{spine_id}","sealer":"did:key:z6MkPG52"}},"id":3}}"#,
    );
    let seal_resp = uds_rpc(&mut stream, &seal_req).await;

    assert_eq!(seal_resp["jsonrpc"], "2.0");
    assert_eq!(seal_resp["id"], 3);
    assert_eq!(
        seal_resp["result"]["success"], true,
        "spine.seal must succeed: {seal_resp}"
    );

    handle.stop();
}

/// PG-52: trio lifecycle with BTSP config present (plain JSON-RPC still routes correctly).
#[cfg(unix)]
#[tokio::test]
async fn uds_trio_lifecycle_with_btsp_config() {
    let tmp = tempfile::tempdir().unwrap();

    let btsp_config = loam_spine_core::btsp::BtspHandshakeConfig {
        required: true,
        provider_socket: tmp.path().join("no-provider.sock"),
        family_id: "pg52-test-fam".into(),
    };

    let sock_path = tmp.path().join("trio-btsp.sock");
    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_uds_server(&sock_path, service, Some(btsp_config))
        .await
        .unwrap();

    let mut stream = tokio::net::UnixStream::connect(&sock_path).await.unwrap();

    // spine.create with BTSP configured — plain JSON-RPC should still route correctly
    let create_resp = uds_rpc(
        &mut stream,
        r#"{"jsonrpc":"2.0","method":"spine.create","params":{"name":"btsp-trio","owner":"did:key:z6MkBtsp"},"id":10}"#,
    )
    .await;

    assert_eq!(create_resp["jsonrpc"], "2.0");
    assert_eq!(create_resp["id"], 10);
    assert!(
        create_resp["result"]["spine_id"].is_string(),
        "spine.create with BTSP config must still work for plain JSON-RPC: {create_resp}"
    );
    let spine_id = create_resp["result"]["spine_id"].as_str().unwrap();

    // entry.append
    let append_req = format!(
        r#"{{"jsonrpc":"2.0","method":"entry.append","params":{{"spine_id":"{spine_id}","entry_type":{{"DataAnchor":{{"data_hash":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],"mime_type":"text/plain","size":512}}}},"committer":"did:key:z6MkBtsp"}},"id":11}}"#,
    );
    let append_resp = uds_rpc(&mut stream, &append_req).await;

    assert_eq!(append_resp["id"], 11);
    assert!(
        append_resp["result"]["entry_hash"].is_array(),
        "entry.append with BTSP config: {append_resp}"
    );

    // spine.seal
    let seal_req = format!(
        r#"{{"jsonrpc":"2.0","method":"spine.seal","params":{{"spine_id":"{spine_id}","sealer":"did:key:z6MkBtsp"}},"id":12}}"#,
    );
    let seal_resp = uds_rpc(&mut stream, &seal_req).await;

    assert_eq!(seal_resp["id"], 12);
    assert_eq!(seal_resp["result"]["success"], true, "seal: {seal_resp}");

    handle.stop();
}

/// PG-52: verify one-shot UDS connections work (composition scripts reconnect per call).
#[cfg(unix)]
#[tokio::test]
async fn uds_trio_oneshot_connections() {
    let tmp = tempfile::tempdir().unwrap();
    let sock_path = tmp.path().join("oneshot.sock");
    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_uds_server(&sock_path, service, None)
        .await
        .unwrap();

    // Each call opens a fresh connection (matches socat/nc composition pattern)
    let mut s1 = tokio::net::UnixStream::connect(&sock_path).await.unwrap();
    let create_resp = uds_rpc(
        &mut s1,
        r#"{"jsonrpc":"2.0","method":"spine.create","params":{"name":"oneshot","owner":"did:key:z6MkOne"},"id":1}"#,
    )
    .await;
    drop(s1);

    let spine_id = create_resp["result"]["spine_id"].as_str().unwrap();

    let mut s2 = tokio::net::UnixStream::connect(&sock_path).await.unwrap();
    let append_req = format!(
        r#"{{"jsonrpc":"2.0","method":"entry.append","params":{{"spine_id":"{spine_id}","entry_type":{{"DataAnchor":{{"data_hash":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3],"mime_type":"application/json","size":256}}}},"committer":"did:key:z6MkOne"}},"id":2}}"#,
    );
    let append_resp = uds_rpc(&mut s2, &append_req).await;
    drop(s2);

    assert!(
        append_resp["result"]["entry_hash"].is_array(),
        "oneshot append: {append_resp}"
    );

    let mut s3 = tokio::net::UnixStream::connect(&sock_path).await.unwrap();
    let seal_req = format!(
        r#"{{"jsonrpc":"2.0","method":"spine.seal","params":{{"spine_id":"{spine_id}","sealer":"did:key:z6MkOne"}},"id":3}}"#,
    );
    let seal_resp = uds_rpc(&mut s3, &seal_req).await;
    drop(s3);

    assert_eq!(
        seal_resp["result"]["success"], true,
        "oneshot seal: {seal_resp}"
    );

    handle.stop();
}
