// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for public chain anchor service methods.

use crate::entry::{AnchorTarget, EntryType};
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
        AnchorTarget::Rfc3161Tsa {
            tsa_url: "https://freetsa.org/tsr".into(),
        },
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

#[tokio::test]
async fn anchor_rfc3161_tsa_roundtrip() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkTsaAnchor");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("TSA Anchor".into()))
        .await
        .expect("spine creation");

    let receipt = service
        .anchor_to_public_chain(
            spine_id,
            AnchorTarget::Rfc3161Tsa {
                tsa_url: "https://freetsa.org/tsr".into(),
            },
            "base64-encoded-tst-token-data".into(),
            0,
            Timestamp::now(),
        )
        .await
        .expect("tsa anchor");

    assert_ne!(receipt.state_hash, crate::types::ContentHash::default());

    let verification = service
        .verify_anchor(spine_id, Some(receipt.entry_hash))
        .await
        .expect("verify tsa anchor");

    assert!(verification.verified);
    assert_eq!(
        verification.anchor_target,
        AnchorTarget::Rfc3161Tsa {
            tsa_url: "https://freetsa.org/tsr".into(),
        }
    );
    assert_eq!(verification.tx_ref, "base64-encoded-tst-token-data");
    assert_eq!(verification.block_height, 0);
}

#[tokio::test]
async fn anchor_batch_requires_at_least_two_spines() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkBatchSingle");

    let spine_id = service
        .ensure_spine(owner, Some("SingleBatch".into()))
        .await
        .expect("ensure");

    service
        .append_entry(
            spine_id,
            EntryType::DataAnchor {
                data_hash: [0x01u8; 32],
                mime_type: None,
                size: 10,
            },
        )
        .await
        .expect("append");

    let result = service
        .anchor_batch(
            &[spine_id],
            AnchorTarget::Bitcoin,
            "tx_single".into(),
            900_000,
            Timestamp::now(),
        )
        .await;

    let Err(e) = result else {
        unreachable!("expected batch minimum error");
    };
    assert!(
        e.to_string().contains("at least 2 spines"),
        "expected batch minimum error, got: {e}"
    );
}

#[tokio::test]
async fn anchor_batch_success_two_spines() {
    let service = LoamSpineService::new();
    let owner_a = Did::new("did:key:z6MkBatchA");
    let owner_b = Did::new("did:key:z6MkBatchB");

    let spine_a = service
        .ensure_spine(owner_a, Some("BatchA".into()))
        .await
        .expect("ensure a");
    let spine_b = service
        .ensure_spine(owner_b, Some("BatchB".into()))
        .await
        .expect("ensure b");

    service
        .append_entry(
            spine_a,
            EntryType::DataAnchor {
                data_hash: [0xA1u8; 32],
                mime_type: None,
                size: 10,
            },
        )
        .await
        .expect("append a");
    service
        .append_entry(
            spine_b,
            EntryType::DataAnchor {
                data_hash: [0xB1u8; 32],
                mime_type: None,
                size: 20,
            },
        )
        .await
        .expect("append b");

    let receipt = service
        .anchor_batch(
            &[spine_a, spine_b],
            AnchorTarget::Bitcoin,
            "tx_batch_test".into(),
            900_001,
            Timestamp::now(),
        )
        .await
        .expect("anchor batch");

    assert_eq!(receipt.entries.len(), 2);
    assert_ne!(receipt.aggregate_root, [0u8; 32]);

    // Verify each anchored spine
    let ver_a = service
        .verify_anchor(spine_a, None)
        .await
        .expect("verify a");
    assert!(ver_a.verified);
    assert_eq!(ver_a.aggregate_verified, Some(true));

    let ver_b = service
        .verify_anchor(spine_b, None)
        .await
        .expect("verify b");
    assert!(ver_b.verified);
    assert_eq!(ver_b.aggregate_verified, Some(true));
}

#[tokio::test]
async fn verify_anchor_no_preceding_entry() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkVerNoPreceding");

    let spine_id = service
        .ensure_spine(owner, Some("NoPreceding".into()))
        .await
        .expect("ensure");

    // The spine has only a genesis entry at index 0.
    // Anchor it — the PublicChainAnchor will be at index 1,
    // and the preceding entry (index 0, genesis) should verify OK.
    // But if we could put the anchor at index 0, there'd be no preceding.
    // Instead, test the None aggregate path:
    let receipt = service
        .anchor_to_public_chain(
            spine_id,
            AnchorTarget::Bitcoin,
            "tx_no_preceding".into(),
            1,
            Timestamp::now(),
        )
        .await
        .expect("anchor");

    let ver = service
        .verify_anchor(spine_id, Some(receipt.entry_hash))
        .await
        .expect("verify");

    assert!(ver.verified);
    assert!(ver.aggregate_verified.is_none());
}
