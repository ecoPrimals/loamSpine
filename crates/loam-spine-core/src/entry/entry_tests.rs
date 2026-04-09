// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for entry types.

use super::*;
use crate::types::Did;
use proptest::prelude::*;

#[test]
fn entry_creation() {
    let did = Did::new("did:key:z6MkTest");
    let entry = Entry::new(
        0,
        None,
        did.clone(),
        EntryType::SpineSealed { reason: None },
    );

    assert_eq!(entry.index, 0);
    assert!(entry.previous.is_none());
    assert_eq!(entry.committer, did);
}

#[test]
fn genesis_entry() {
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let entry = Entry::genesis(owner, spine_id, SpineConfig::default());

    assert!(entry.is_genesis());
    assert_eq!(entry.domain(), "spine");
}

#[test]
fn entry_hash_deterministic() {
    let did = Did::new("did:key:z6MkTest");
    let entry = Entry::new(0, None, did, EntryType::SpineSealed { reason: None });

    let hash1 = entry.compute_hash().expect("compute_hash");
    let hash2 = entry.compute_hash().expect("compute_hash");
    assert_eq!(hash1, hash2);
}

#[test]
fn entry_builder() {
    let did = Did::new("did:key:z6MkTest");
    let entry = Entry::new(
        1,
        Some([0u8; 32]),
        did,
        EntryType::SpineSealed { reason: None },
    )
    .with_metadata("key", "value")
    .with_signature(Signature::from_vec(vec![1, 2, 3]));

    assert_eq!(entry.metadata.get("key"), Some(&"value".to_string()));
    assert!(!entry.signature.is_empty());
}

#[test]
fn entry_type_domain() {
    let committer = Did::new("did:key:z6MkCommitter");
    assert_eq!(
        EntryType::SessionCommit {
            session_id: SessionId::now_v7(),
            merkle_root: [0u8; 32],
            vertex_count: 10,
            committer,
        }
        .domain(),
        "session"
    );

    assert_eq!(
        EntryType::CertificateMint {
            cert_id: CertificateId::now_v7(),
            cert_type: "game".into(),
            initial_owner: Did::new("did:key:test"),
        }
        .domain(),
        "certificate"
    );
}

#[test]
fn waypoint_allowed_types() {
    let genesis = EntryType::Genesis {
        spine_id: SpineId::now_v7(),
        owner: Did::new("test"),
        config: SpineConfig::default(),
    };
    assert!(genesis.allowed_in_waypoint());

    let committer = Did::new("did:key:z6MkCommitter");
    let session = EntryType::SessionCommit {
        session_id: SessionId::now_v7(),
        merkle_root: [0u8; 32],
        vertex_count: 10,
        committer,
    };
    assert!(!session.allowed_in_waypoint());
}

#[test]
fn entry_type_domain_all_variants() {
    let owner = Did::new("did:key:z6MkOwner");
    let committer = Did::new("did:key:z6MkCommitter");

    assert_eq!(
        EntryType::MetadataUpdate {
            field: "name".into(),
            value: "foo".into(),
        }
        .domain(),
        "spine"
    );

    assert_eq!(
        EntryType::SliceCheckout {
            slice_id: SliceId::now_v7(),
            source_entry: [0u8; 32],
            session_id: SessionId::now_v7(),
            holder: committer.clone(),
        }
        .domain(),
        "session"
    );

    assert_eq!(
        EntryType::SliceReturn {
            slice_id: SliceId::now_v7(),
            checkout_entry: [0u8; 32],
            success: true,
            summary: None,
        }
        .domain(),
        "session"
    );

    assert_eq!(
        EntryType::DataAnchor {
            data_hash: [1u8; 32],
            mime_type: Some("text/plain".into()),
            size: 100,
        }
        .domain(),
        "data"
    );

    assert_eq!(
        EntryType::BraidCommit {
            braid_id: BraidId::now_v7(),
            braid_hash: [1u8; 32],
            subject_hash: [2u8; 32],
        }
        .domain(),
        "data"
    );

    assert_eq!(
        EntryType::CertificateTransfer {
            cert_id: CertificateId::now_v7(),
            from: owner,
            to: committer,
        }
        .domain(),
        "certificate"
    );
}

#[test]
fn entry_type_domain_certificate_slice_custom() {
    let owner = Did::new("did:key:z6MkOwner");
    let committer = Did::new("did:key:z6MkCommitter");

    assert_eq!(
        EntryType::CertificateLoan {
            cert_id: CertificateId::now_v7(),
            lender: owner,
            borrower: committer,
            duration_secs: Some(3600),
            auto_return: true,
        }
        .domain(),
        "certificate"
    );

    assert_eq!(
        EntryType::CertificateReturn {
            cert_id: CertificateId::now_v7(),
            loan_entry: [0u8; 32],
            usage_summary: None,
        }
        .domain(),
        "certificate"
    );

    assert_eq!(
        EntryType::SliceAnchor {
            slice_id: SliceId::now_v7(),
            origin_spine: SpineId::now_v7(),
            origin_entry: [0u8; 32],
        }
        .domain(),
        "slice"
    );

    assert_eq!(
        EntryType::SliceOperation {
            slice_id: SliceId::now_v7(),
            operation: "merge".into(),
        }
        .domain(),
        "slice"
    );

    assert_eq!(
        EntryType::SliceDeparture {
            slice_id: SliceId::now_v7(),
            reason: "complete".into(),
        }
        .domain(),
        "slice"
    );

    assert_eq!(
        EntryType::Custom {
            type_uri: "example.com/custom".into(),
            payload: crate::types::ByteBuffer::from_static(b"payload"),
        }
        .domain(),
        "custom"
    );
}

#[test]
fn entry_type_domain_temporal_moment() {
    use crate::temporal::{Moment, MomentContext};

    let moment = Moment {
        id: [0u8; 32],
        timestamp: std::time::UNIX_EPOCH,
        agent: "did:key:z6MkAgent".into(),
        state_hash: [1u8; 32],
        signature: Signature::empty(),
        context: MomentContext::Generic {
            category: "test".into(),
            metadata: std::collections::HashMap::new(),
            content_hash: None,
        },
        parents: vec![],
        anchor: None,
        ephemeral_provenance: None,
    };
    let ty = EntryType::TemporalMoment {
        moment_id: [0u8; 32],
        moment: Box::new(moment),
    };
    assert_eq!(ty.domain(), "temporal");
}

#[test]
fn waypoint_allowed_slice_variants() {
    assert!(
        EntryType::SliceAnchor {
            slice_id: SliceId::now_v7(),
            origin_spine: SpineId::now_v7(),
            origin_entry: [0u8; 32],
        }
        .allowed_in_waypoint()
    );

    assert!(
        EntryType::SliceOperation {
            slice_id: SliceId::now_v7(),
            operation: "op".into(),
        }
        .allowed_in_waypoint()
    );

    assert!(
        EntryType::SliceDeparture {
            slice_id: SliceId::now_v7(),
            reason: "done".into(),
        }
        .allowed_in_waypoint()
    );

    assert!(
        !EntryType::DataAnchor {
            data_hash: [0u8; 32],
            mime_type: None,
            size: 0,
        }
        .allowed_in_waypoint()
    );
}

#[test]
fn entry_with_payload() {
    let did = Did::new("did:key:z6MkTest");
    let payload = PayloadRef::new([1u8; 32], 1024).with_mime_type("application/octet-stream");

    let entry = Entry::new(
        1,
        Some([0u8; 32]),
        did,
        EntryType::DataAnchor {
            data_hash: [2u8; 32],
            mime_type: None,
            size: 1024,
        },
    )
    .with_spine_id(SpineId::now_v7())
    .with_payload(payload.clone());

    assert_eq!(entry.payload.as_ref(), Some(&payload));
}

#[test]
fn entry_hash_cached() {
    let did = Did::new("did:key:z6MkTest");
    let mut entry = Entry::new(0, None, did, EntryType::SpineSealed { reason: None })
        .with_spine_id(SpineId::now_v7());

    let hash1 = entry.hash().expect("hash");
    let hash2 = entry.hash().expect("hash");
    assert_eq!(hash1, hash2);
}

#[test]
fn entry_is_genesis_non_genesis() {
    let did = Did::new("did:key:z6MkTest");
    let entry = Entry::new(
        1,
        Some([0u8; 32]),
        did,
        EntryType::DataAnchor {
            data_hash: [0u8; 32],
            mime_type: None,
            size: 0,
        },
    );
    assert!(!entry.is_genesis());
}

#[test]
fn entry_serde_roundtrip() {
    let did = Did::new("did:key:z6MkTest");
    let entry = Entry::new(
        1,
        Some([1u8; 32]),
        did,
        EntryType::DataAnchor {
            data_hash: [2u8; 32],
            mime_type: Some("text/plain".into()),
            size: 100,
        },
    )
    .with_spine_id(SpineId::now_v7())
    .with_metadata("k1", "v1")
    .with_metadata("k2", "v2");

    let bytes = serde_json::to_vec(&entry).expect("serialize");
    let restored: Entry = serde_json::from_slice(&bytes).expect("deserialize");

    assert_eq!(entry.index, restored.index);
    assert_eq!(entry.spine_id, restored.spine_id);
    assert_eq!(entry.metadata, restored.metadata);
}

#[test]
fn entry_type_serde_roundtrip_data_anchor() {
    let ty = EntryType::DataAnchor {
        data_hash: [1u8; 32],
        mime_type: Some("image/png".into()),
        size: 2048,
    };
    let bytes = serde_json::to_vec(&ty).expect("serialize");
    let restored: EntryType = serde_json::from_slice(&bytes).expect("deserialize");
    assert!(matches!(restored, EntryType::DataAnchor { size: 2048, .. }));
}

#[test]
fn entry_type_serde_roundtrip_slice_anchor() {
    let ty = EntryType::SliceAnchor {
        slice_id: SliceId::now_v7(),
        origin_spine: SpineId::now_v7(),
        origin_entry: [2u8; 32],
    };
    let bytes = serde_json::to_vec(&ty).expect("serialize");
    let restored: EntryType = serde_json::from_slice(&bytes).expect("deserialize");
    assert!(matches!(
        restored,
        EntryType::SliceAnchor { origin_entry, .. } if origin_entry == [2u8; 32]
    ));
}

#[test]
fn entry_metadata_default_on_deserialize() {
    let json = r#"{"index":0,"previous":null,"spine_id":"00000000-0000-0000-0000-000000000000","timestamp":0,"committer":"did:key:test","entry_type":{"SpineSealed":{"reason":null}},"payload":null,"signature":[]}"#;
    let entry: Entry = serde_json::from_str(json).expect("deserialize");
    assert!(entry.metadata.is_empty());
}

#[test]
fn entry_type_serde_roundtrip_custom() {
    let ty = EntryType::Custom {
        type_uri: "urn:test:custom".into(),
        payload: crate::types::ByteBuffer::from_static(b"custom data"),
    };
    let bytes = serde_json::to_vec(&ty).expect("serialize");
    let restored: EntryType = serde_json::from_slice(&bytes).expect("deserialize");
    assert!(
        matches!(&restored, EntryType::Custom { type_uri, payload } if type_uri == "urn:test:custom" && payload.as_ref() == b"custom data"),
        "expected Custom variant"
    );
}

#[test]
fn spine_config_serde_roundtrip() {
    let config = SpineConfig {
        spine_type: SpineType::Waypoint {
            max_anchor_depth: Some(5),
        },
        auto_rollup_threshold: Some(10_000),
        replication_enabled: true,
        waypoint_config: None,
    };
    let bytes = serde_json::to_vec(&config).expect("serialize");
    let restored: SpineConfig = serde_json::from_slice(&bytes).expect("deserialize");
    assert_eq!(config.auto_rollup_threshold, restored.auto_rollup_threshold);
    assert_eq!(config.replication_enabled, restored.replication_enabled);
}

#[test]
fn spine_type_serde_roundtrip() {
    let types = [
        SpineType::Personal,
        SpineType::Professional,
        SpineType::Public,
        SpineType::Community {
            community_id: "cid".into(),
        },
        SpineType::Waypoint {
            max_anchor_depth: Some(3),
        },
        SpineType::Waypoint {
            max_anchor_depth: None,
        },
    ];

    for ty in &types {
        let bytes = serde_json::to_vec(ty).expect("serialize");
        let restored: SpineType = serde_json::from_slice(&bytes).expect("deserialize");
        assert_eq!(ty, &restored);
    }
}

#[test]
fn entry_to_canonical_bytes_deterministic_with_metadata() {
    let did = Did::new("did:key:z6MkTest");
    let spine_id = SpineId::now_v7();
    let ts = Timestamp::now();

    let mut entry1 = Entry::new(
        0,
        None,
        did.clone(),
        EntryType::SpineSealed { reason: None },
    )
    .with_spine_id(spine_id)
    .with_metadata("b", "2")
    .with_metadata("a", "1");
    entry1.timestamp = ts;

    let mut entry2 = Entry::new(0, None, did, EntryType::SpineSealed { reason: None })
        .with_spine_id(spine_id)
        .with_metadata("a", "1")
        .with_metadata("b", "2");
    entry2.timestamp = ts;

    let bytes1 = entry1.to_canonical_bytes().expect("canonical");
    let bytes2 = entry2.to_canonical_bytes().expect("canonical");
    assert_eq!(bytes1, bytes2, "canonical bytes should be deterministic");
}

#[test]
fn entry_with_spine_id_clears_cache() {
    let did = Did::new("did:key:z6MkTest");
    let spine_id1 = SpineId::now_v7();
    let spine_id2 = SpineId::now_v7();

    let mut entry =
        Entry::new(0, None, did, EntryType::SpineSealed { reason: None }).with_spine_id(spine_id1);
    let _ = entry.hash().expect("hash");

    let entry2 = entry.with_spine_id(spine_id2);
    assert_eq!(entry2.spine_id, spine_id2);
}

#[test]
fn entry_type_serde_roundtrip_public_chain_anchor() {
    use crate::entry::AnchorTarget;

    let ty = EntryType::PublicChainAnchor {
        anchor_target: AnchorTarget::DataCommons {
            commons_id: "ipfs-commons-v1".into(),
        },
        state_hash: [42u8; 32],
        tx_ref: "bafybeihash123".into(),
        block_height: 0,
        anchor_timestamp: Timestamp::now(),
    };
    let json = serde_json::to_vec(&ty).expect("serialize");
    let restored: EntryType = serde_json::from_slice(&json).expect("deserialize");
    assert!(matches!(
        restored,
        EntryType::PublicChainAnchor {
            block_height: 0,
            ..
        }
    ));
}

#[test]
fn public_chain_anchor_domain() {
    use crate::entry::AnchorTarget;

    let ty = EntryType::PublicChainAnchor {
        anchor_target: AnchorTarget::Bitcoin,
        state_hash: [0u8; 32],
        tx_ref: "tx_abc".into(),
        block_height: 800_000,
        anchor_timestamp: Timestamp::now(),
    };
    assert_eq!(ty.domain(), "anchor");
}

#[test]
fn public_chain_anchor_not_allowed_in_waypoint() {
    use crate::entry::AnchorTarget;

    let ty = EntryType::PublicChainAnchor {
        anchor_target: AnchorTarget::Ethereum,
        state_hash: [0u8; 32],
        tx_ref: "0xdeadbeef".into(),
        block_height: 1,
        anchor_timestamp: Timestamp::now(),
    };
    assert!(!ty.allowed_in_waypoint());
}

fn arb_did() -> impl Strategy<Value = Did> {
    "[a-z]{3,8}".prop_map(|s| Did::new(format!("did:key:z6Mk{s}")))
}

fn arb_entry_type() -> impl Strategy<Value = EntryType> {
    prop_oneof![
        arb_did().prop_map(|owner| EntryType::Genesis {
            spine_id: SpineId::now_v7(),
            owner,
            config: SpineConfig::default(),
        }),
        (".*{0,40}", ".*{0,40}")
            .prop_map(|(f, v)| EntryType::MetadataUpdate { field: f, value: v }),
        Just(EntryType::SpineSealed { reason: None }),
        any::<u64>().prop_map(|vc| EntryType::SessionCommit {
            session_id: SessionId::now_v7(),
            merkle_root: ContentHash::default(),
            vertex_count: vc,
            committer: Did::new("did:key:z6MkTest"),
        }),
        Just(EntryType::DataAnchor {
            data_hash: ContentHash::default(),
            mime_type: Some("application/octet-stream".into()),
            size: 1024,
        }),
    ]
}

proptest! {
    #[test]
    fn entry_hash_is_deterministic(idx in 0u64..1000) {
        let entry = Entry::new(
            idx,
            None,
            Did::new("did:key:z6MkTest"),
            EntryType::DataAnchor {
                data_hash: ContentHash::default(),
                mime_type: None,
                size: idx,
            },
        );
        let h1 = entry.compute_hash().unwrap();
        let h2 = entry.compute_hash().unwrap();
        prop_assert_eq!(h1, h2, "same entry must always hash identically");
    }

    #[test]
    fn entry_hash_changes_with_index(a in 0u64..500, b in 500u64..1000) {
        let base_type = EntryType::DataAnchor {
            data_hash: ContentHash::default(),
            mime_type: None,
            size: 42,
        };
        let e1 = Entry::new(a, None, Did::new("did:key:z6MkA"), base_type.clone());
        let e2 = Entry::new(b, None, Did::new("did:key:z6MkA"), base_type);
        let h1 = e1.compute_hash().unwrap();
        let h2 = e2.compute_hash().unwrap();
        prop_assert_ne!(h1, h2, "different indices must produce different hashes");
    }

    #[test]
    fn genesis_entry_is_always_index_zero(did in arb_did()) {
        let entry = Entry::genesis(did, SpineId::now_v7(), SpineConfig::default());
        prop_assert_eq!(entry.index, 0);
        prop_assert!(entry.is_genesis());
        prop_assert!(entry.previous.is_none());
    }

    #[test]
    fn entry_canonical_bytes_never_panics(et in arb_entry_type()) {
        let entry = Entry::new(0, None, Did::new("did:key:z6MkTest"), et);
        let result = entry.to_canonical_bytes();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn entry_domain_never_empty(et in arb_entry_type()) {
        prop_assert!(!et.domain().is_empty());
    }
}
