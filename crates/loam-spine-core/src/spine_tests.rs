// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn spine_creation() {
    let owner = Did::new("did:key:z6MkOwner");
    let spine = Spine::new(owner.clone(), Some("Test".into()), SpineConfig::default());

    assert!(spine.is_ok());
    let spine = spine.unwrap_or_else(|_| unreachable!());

    assert_eq!(spine.owner, owner);
    assert_eq!(spine.height, 1);
    assert!(spine.is_active());
    assert!(spine.genesis_entry().is_some());
}

#[test]
fn spine_builder() {
    let owner = Did::new("did:key:z6MkOwner");
    let spine = Spine::builder(owner)
        .with_name("My Spine")
        .personal()
        .with_auto_rollup(10_000)
        .with_metadata("created_by", "test")
        .build();

    assert!(spine.is_ok());
    let spine = spine.unwrap_or_else(|_| unreachable!());

    assert_eq!(spine.name, Some("My Spine".to_string()));
    assert_eq!(spine.spine_type, SpineType::Personal);
    assert_eq!(spine.metadata.get("created_by"), Some(&"test".to_string()));
}

#[test]
fn spine_append() {
    let owner = Did::new("did:key:z6MkOwner");
    let mut spine =
        Spine::new(owner, None, SpineConfig::default()).unwrap_or_else(|_| unreachable!());

    let entry = spine.create_entry(EntryType::DataAnchor {
        data_hash: [1u8; 32],
        mime_type: Some("text/plain".into()),
        size: 100,
    });

    let result = spine.append(entry);
    assert!(result.is_ok());
    assert_eq!(spine.height, 2);
}

#[test]
fn spine_seal() {
    let owner = Did::new("did:key:z6MkOwner");
    let mut spine =
        Spine::new(owner, None, SpineConfig::default()).unwrap_or_else(|_| unreachable!());

    let result = spine.seal(Some("Test complete".into()));
    assert!(result.is_ok());
    assert!(spine.is_sealed());

    // Cannot append after sealing
    let entry = spine.create_entry(EntryType::SpineSealed { reason: None });
    let result = spine.append(entry);
    assert!(result.is_err());
}

#[test]
fn spine_verify() {
    let owner = Did::new("did:key:z6MkOwner");
    let mut spine =
        Spine::new(owner, None, SpineConfig::default()).unwrap_or_else(|_| unreachable!());

    // Add some entries
    for i in 0u8..5 {
        let entry = spine.create_entry(EntryType::DataAnchor {
            data_hash: [i; 32],
            mime_type: None,
            size: u64::from(i),
        });
        spine.append(entry).ok();
    }

    let verification = spine.verify();
    assert!(verification.valid);
    assert!(verification.errors.is_empty());
    assert_eq!(verification.entries_verified, 6); // genesis + 5
}

#[test]
fn waypoint_spine() {
    let owner = Did::new("did:key:z6MkOwner");
    let spine = Spine::builder(owner)
        .waypoint(Some(3))
        .with_name("Borrowed Games")
        .build();

    assert!(spine.is_ok());
    let spine = spine.unwrap_or_else(|_| unreachable!());

    assert!(matches!(
        spine.spine_type,
        SpineType::Waypoint {
            max_anchor_depth: Some(3)
        }
    ));
}

#[test]
fn spine_state_checks() {
    assert!(SpineState::Active.is_active());
    assert!(!SpineState::Active.is_sealed());
    assert!(!SpineState::Active.is_terminal());

    let sealed = SpineState::Sealed {
        sealed_at: Timestamp::now(),
        final_entry: [0u8; 32],
    };
    assert!(!sealed.is_active());
    assert!(sealed.is_sealed());
    assert!(sealed.is_terminal());
}

#[test]
fn spine_get_entry_and_entries() {
    let owner = Did::new("did:key:z6MkOwner");
    let mut spine =
        Spine::new(owner, None, SpineConfig::default()).unwrap_or_else(|_| unreachable!());

    let entry = spine.create_entry(EntryType::DataAnchor {
        data_hash: [1u8; 32],
        mime_type: None,
        size: 42,
    });
    spine.append(entry).ok();

    assert!(spine.get_entry(0).is_some());
    assert!(spine.get_entry(1).is_some());
    assert!(spine.get_entry(2).is_none());
    assert!(spine.get_entry(u64::MAX).is_none());

    assert_eq!(spine.entries().len(), 2);
    assert_eq!(spine.tip_entry().map(|e| e.index), Some(1));
}

#[test]
fn spine_append_index_mismatch() {
    let owner = Did::new("did:key:z6MkOwner");
    let mut spine =
        Spine::new(owner.clone(), None, SpineConfig::default()).unwrap_or_else(|_| unreachable!());

    let _wrong_index_entry = spine.create_entry(EntryType::DataAnchor {
        data_hash: [1u8; 32],
        mime_type: None,
        size: 1,
    });
    let mut bad_entry = Entry::new(
        999,
        Some(spine.tip),
        owner,
        EntryType::DataAnchor {
            data_hash: [1u8; 32],
            mime_type: None,
            size: 1,
        },
    )
    .with_spine_id(spine.id);
    bad_entry.index = 999;

    let result = spine.append(bad_entry);
    assert!(result.is_err());
    if let Err(LoamSpineError::ChainValidation { index, reason }) = result {
        assert_eq!(index, 999);
        assert!(reason.contains("expected index"));
    }
}

#[test]
fn spine_append_previous_hash_mismatch() {
    let owner = Did::new("did:key:z6MkOwner");
    let mut spine =
        Spine::new(owner.clone(), None, SpineConfig::default()).unwrap_or_else(|_| unreachable!());

    let bad_entry = Entry::new(
        1,
        Some([0xFFu8; 32]),
        owner,
        EntryType::DataAnchor {
            data_hash: [1u8; 32],
            mime_type: None,
            size: 1,
        },
    )
    .with_spine_id(spine.id);

    let result = spine.append(bad_entry);
    assert!(result.is_err());
    if let Err(LoamSpineError::ChainValidation { reason, .. }) = result {
        assert!(reason.contains("previous hash mismatch"));
    }
}

#[test]
fn spine_state_frozen_and_archived() {
    let frozen = SpineState::Frozen {
        reason: "maintenance".into(),
        until: Some(Timestamp::from_nanos(1_000_000_000)),
    };
    assert!(!frozen.is_active());
    assert!(!frozen.is_sealed());
    assert!(!frozen.is_terminal());

    let archived = SpineState::Archived {
        archived_at: Timestamp::now(),
        archive_location: "cold://bucket/path".into(),
    };
    assert!(!archived.is_active());
    assert!(!archived.is_sealed());
    assert!(archived.is_terminal());
}

#[test]
fn spine_verify_invalid_chain() {
    let owner = Did::new("did:key:z6MkOwner");
    let mut spine =
        Spine::new(owner, None, SpineConfig::default()).unwrap_or_else(|_| unreachable!());

    let entry = spine.create_entry(EntryType::DataAnchor {
        data_hash: [1u8; 32],
        mime_type: None,
        size: 1,
    });
    spine.append(entry).ok();

    spine.entries[1].index = 99;

    let verification = spine.verify();
    assert!(!verification.valid);
    assert!(!verification.errors.is_empty());
}

#[test]
fn spine_builder_professional_and_community() {
    let owner = Did::new("did:key:z6MkOwner");

    let spine = Spine::builder(owner.clone())
        .with_type(SpineType::Professional)
        .build();
    assert!(spine.is_ok());
    assert_eq!(
        spine.unwrap_or_else(|_| unreachable!()).spine_type,
        SpineType::Professional
    );

    let spine = Spine::builder(owner)
        .with_type(SpineType::Community {
            community_id: "comm-123".into(),
        })
        .build();
    assert!(spine.is_ok());
    assert!(matches!(
        spine.unwrap_or_else(|_| unreachable!()).spine_type,
        SpineType::Community {
            community_id: ref id
        } if id == "comm-123"
    ));
}

#[test]
fn spine_builder_waypoint_none() {
    let owner = Did::new("did:key:z6MkOwner");
    let spine = Spine::builder(owner).waypoint(None).build();
    assert!(spine.is_ok());
    assert!(matches!(
        spine.unwrap_or_else(|_| unreachable!()).spine_type,
        SpineType::Waypoint {
            max_anchor_depth: None
        }
    ));
}

#[test]
fn spine_builder_with_replication() {
    let owner = Did::new("did:key:z6MkOwner");
    let spine = Spine::builder(owner).with_replication(true).build();
    assert!(spine.is_ok());
    assert!(
        spine
            .unwrap_or_else(|_| unreachable!())
            .config
            .replication_enabled
    );
}

#[test]
fn spine_serde_roundtrip() {
    let owner = Did::new("did:key:z6MkOwner");
    let spine = Spine::new(owner, Some("Roundtrip".into()), SpineConfig::default())
        .unwrap_or_else(|_| unreachable!());

    let bytes = serde_json::to_vec(&spine).unwrap_or_else(|_| unreachable!());
    let restored: Spine = serde_json::from_slice(&bytes).unwrap_or_else(|_| unreachable!());

    assert_eq!(spine.id, restored.id);
    assert_eq!(spine.name, restored.name);
    assert_eq!(spine.height, restored.height);
}

#[test]
fn chain_error_debug() {
    let err = ChainError::IndexGap {
        expected: 0,
        actual: 1,
    };
    let s = format!("{err:?}");
    assert!(s.contains("IndexGap"));
}

#[test]
fn chain_error_hash_mismatch_and_invalid_signature() {
    let err = ChainError::HashMismatch {
        index: 1,
        expected: Some([0u8; 32]),
        actual: Some([1u8; 32]),
    };
    let s = format!("{err:?}");
    assert!(s.contains("HashMismatch"));

    let err2 = ChainError::InvalidSignature { index: 2 };
    let s2 = format!("{err2:?}");
    assert!(s2.contains("InvalidSignature"));
}

#[test]
fn spine_seal_with_no_reason() {
    let owner = Did::new("did:key:z6MkOwner");
    let mut spine =
        Spine::new(owner, None, SpineConfig::default()).unwrap_or_else(|_| unreachable!());
    let result = spine.seal(None);
    assert!(result.is_ok());
    assert!(spine.is_sealed());
}

#[test]
fn chain_verification_with_errors() {
    let v = ChainVerification {
        spine_id: SpineId::now_v7(),
        entries_verified: 3,
        valid: false,
        errors: vec![ChainError::IndexGap {
            expected: 1,
            actual: 2,
        }],
    };
    assert!(!v.valid);
    assert_eq!(v.errors.len(), 1);
}
