// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for public chain anchor service methods.

use crate::entry::AnchorTarget;
use crate::service::LoamSpineService;
use crate::types::{Did, Timestamp};

#[tokio::test]
async fn anchor_roundtrip() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkAnchorTest");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Anchor Test".into()))
        .await
        .expect("spine creation");

    let receipt = service
        .anchor_to_public_chain(
            spine_id,
            AnchorTarget::DataCommons {
                commons_id: "ipfs-commons-v1".into(),
            },
            "bafybeihash123".into(),
            0,
            Timestamp::now(),
        )
        .await
        .expect("anchor_to_public_chain");

    assert_ne!(receipt.state_hash, crate::types::ContentHash::default());

    let verification = service
        .verify_anchor(spine_id, Some(receipt.entry_hash))
        .await
        .expect("verify_anchor");

    assert!(verification.verified);
    assert_eq!(
        verification.anchor_target,
        AnchorTarget::DataCommons {
            commons_id: "ipfs-commons-v1".into(),
        }
    );
    assert_eq!(verification.tx_ref, "bafybeihash123");
    assert_eq!(verification.block_height, 0);
}

#[tokio::test]
async fn verify_latest_anchor() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkAnchorLatest");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Latest Anchor".into()))
        .await
        .expect("spine creation");

    let _first = service
        .anchor_to_public_chain(
            spine_id,
            AnchorTarget::Bitcoin,
            "tx_aaa".into(),
            100,
            Timestamp::now(),
        )
        .await
        .expect("first anchor");

    let second = service
        .anchor_to_public_chain(
            spine_id,
            AnchorTarget::Ethereum,
            "0xdeadbeef".into(),
            42,
            Timestamp::now(),
        )
        .await
        .expect("second anchor");

    let verification = service
        .verify_anchor(spine_id, None)
        .await
        .expect("verify latest");

    assert!(verification.verified);
    assert_eq!(verification.anchor_target, AnchorTarget::Ethereum);
    assert_eq!(verification.state_hash, second.state_hash);
}

#[tokio::test]
async fn anchor_on_missing_spine_fails() {
    let service = LoamSpineService::new();
    let result = service
        .anchor_to_public_chain(
            crate::types::SpineId::now_v7(),
            AnchorTarget::Bitcoin,
            "tx_xxx".into(),
            0,
            Timestamp::now(),
        )
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn verify_non_anchor_entry_fails() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkNonAnchor");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Non Anchor".into()))
        .await
        .expect("spine creation");

    let entry_hash = service
        .append_entry(
            spine_id,
            crate::entry::EntryType::MetadataUpdate {
                field: "test".into(),
                value: "value".into(),
            },
        )
        .await
        .expect("append metadata");

    let result = service.verify_anchor(spine_id, Some(entry_hash)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn anchor_target_serde_roundtrip() {
    let targets = vec![
        AnchorTarget::Bitcoin,
        AnchorTarget::Ethereum,
        AnchorTarget::FederatedSpine {
            peer_id: "spine-peer-abc".into(),
        },
        AnchorTarget::DataCommons {
            commons_id: "commons-xyz".into(),
        },
        AnchorTarget::Other {
            name: "custom-ledger".into(),
        },
    ];

    for target in targets {
        let json = serde_json::to_string(&target).expect("serialize");
        let back: AnchorTarget = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(target, back);
    }
}
