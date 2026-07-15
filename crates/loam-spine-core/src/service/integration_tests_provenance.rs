// SPDX-License-Identifier: AGPL-3.0-or-later

//! ProvenanceSource tests + Nest depth convergence verification.

use super::*;
use crate::types::Timestamp;

#[tokio::test]
async fn test_get_entries_for_data() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("DataTest".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let data_hash = [0xFEu8; 32];
    service
        .append_entry(
            spine_id,
            EntryType::DataAnchor {
                data_hash,
                mime_type: Some("application/json".into()),
                size: 1024,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let entries = service
        .get_entries_for_data(data_hash)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(entries.len(), 1);

    let no_entries = service
        .get_entries_for_data([0u8; 32])
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(no_entries.is_empty());
}

#[tokio::test]
async fn test_get_certificate_history() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("CertHistory".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert_id = crate::types::CertificateId::now_v7();

    service
        .append_entry(
            spine_id,
            EntryType::CertificateMint {
                cert_id,
                cert_type: "game".into(),
                initial_owner: owner.clone(),
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let new_owner = Did::new("did:key:z6MkNewOwner");
    service
        .append_entry(
            spine_id,
            EntryType::CertificateTransfer {
                cert_id,
                from: owner.clone(),
                to: new_owner,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let history = service
        .get_certificate_history(cert_id)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(history.len(), 2);

    let no_history = service
        .get_certificate_history(crate::types::CertificateId::now_v7())
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(no_history.is_empty());
}

#[tokio::test]
async fn test_get_attribution() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkCreator");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("AttrTest".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let data_hash = [0xBBu8; 32];
    service
        .append_entry(
            spine_id,
            EntryType::DataAnchor {
                data_hash,
                mime_type: Some("text/plain".into()),
                size: 256,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let attr = service
        .get_attribution(data_hash)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(attr.is_some());

    let record = attr.unwrap_or_else(|| unreachable!());
    assert_eq!(record.content_hash, data_hash);
    assert_eq!(record.creator, owner);

    let no_attr = service
        .get_attribution([0u8; 32])
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(no_attr.is_none());
}

#[tokio::test]
async fn test_get_attribution_with_contributors() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkCreatorContrib");
    let contributor = Did::new("did:key:z6MkContributor");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("ContribAttrTest".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let data_hash = [0xCBu8; 32];

    service
        .append_entry(
            spine_id,
            EntryType::DataAnchor {
                data_hash,
                mime_type: Some("text/plain".into()),
                size: 100,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    service
        .append_entry(
            spine_id,
            EntryType::SessionCommit {
                session_id: crate::types::SessionId::now_v7(),
                merkle_root: data_hash,
                vertex_count: 5,
                committer: contributor.clone(),
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let attr = service
        .get_attribution(data_hash)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(attr.is_some());

    let record = attr.unwrap_or_else(|| unreachable!());
    assert_eq!(record.creator, owner);
    assert!(
        record.contributors.contains(&contributor),
        "expected contributor in attribution record"
    );
}

#[tokio::test]
async fn test_get_provenance_chain() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("ProvenanceTest".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let content_hash = [0xCCu8; 32];

    service
        .append_entry(
            spine_id,
            EntryType::DataAnchor {
                data_hash: content_hash,
                mime_type: Some("text/plain".into()),
                size: 128,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    service
        .append_entry(
            spine_id,
            EntryType::BraidCommit {
                braid_id: BraidId::now_v7(),
                braid_hash: [0xDDu8; 32],
                subject_hash: content_hash,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let chain = service
        .get_provenance_chain(content_hash)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(chain.len(), 2);
    assert_eq!(chain[0].relationship, "anchored-by");
    assert_eq!(chain[0].depth, 0);
    assert_eq!(chain[1].relationship, "attributed-to");
    assert_eq!(chain[1].depth, 1);

    let empty_chain = service
        .get_provenance_chain([0u8; 32])
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(empty_chain.is_empty());
}

/// Build a 6-link provenance chain across two spines for Nest depth testing.
async fn build_six_link_provenance(
    service: &LoamSpineService,
    content_hash: ContentHash,
) -> (SpineId, SpineId) {
    let owner = Did::new("did:key:z6MkNestOwner");
    let owner_b = Did::new("did:key:z6MkNestOwnerB");

    let spine_a = service
        .ensure_spine(owner.clone(), Some("NestLedgerA".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let spine_b = service
        .ensure_spine(owner_b, Some("NestLedgerB".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let entries: Vec<(SpineId, EntryType)> = vec![
        (
            spine_a,
            EntryType::DataAnchor {
                data_hash: content_hash,
                mime_type: Some("application/octet-stream".into()),
                size: 256,
            },
        ),
        (
            spine_a,
            EntryType::SessionCommit {
                session_id: crate::types::SessionId::now_v7(),
                merkle_root: content_hash,
                vertex_count: 10,
                committer: owner.clone(),
            },
        ),
        (
            spine_a,
            EntryType::BraidCommit {
                braid_id: BraidId::now_v7(),
                braid_hash: [0xBBu8; 32],
                subject_hash: content_hash,
            },
        ),
        (
            spine_b,
            EntryType::PublicChainAnchor {
                anchor_target: crate::entry::AnchorTarget::Bitcoin,
                state_hash: content_hash,
                tx_ref: "tx_nest_depth_test".into(),
                block_height: 850_000,
                anchor_timestamp: Timestamp::now(),
                aggregate_root: None,
                inclusion_proof: None,
            },
        ),
        (
            spine_b,
            EntryType::SessionCommit {
                session_id: crate::types::SessionId::now_v7(),
                merkle_root: content_hash,
                vertex_count: 20,
                committer: owner.clone(),
            },
        ),
        (
            spine_b,
            EntryType::BraidCommit {
                braid_id: BraidId::now_v7(),
                braid_hash: [0xEEu8; 32],
                subject_hash: content_hash,
            },
        ),
    ];

    for (sid, etype) in entries {
        service
            .append_entry(sid, etype)
            .await
            .unwrap_or_else(|_| unreachable!());
    }

    (spine_a, spine_b)
}

/// Nest provenance depth convergence test: builds a 6-link chain
/// for a single content hash, verifying depth >= 5.
#[tokio::test]
async fn test_provenance_chain_depth_five_plus() {
    let service = LoamSpineService::new();
    let content_hash = [0xAAu8; 32];

    let (_, _) = build_six_link_provenance(&service, content_hash).await;

    let chain = service
        .get_provenance_chain(content_hash)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(
        chain.len() >= 6,
        "expected 6+ links for Nest provenance depth 5+, got {}",
        chain.len()
    );

    for (i, link) in chain.iter().enumerate() {
        let expected = u32::try_from(i).unwrap_or(u32::MAX);
        assert_eq!(link.depth, expected, "depth mismatch at position {i}");
    }

    let relationships: Vec<&str> = chain.iter().map(|l| l.relationship.as_str()).collect();
    assert!(
        relationships.contains(&"anchored-by"),
        "missing anchored-by"
    );
    assert!(
        relationships.contains(&"committed-from"),
        "missing committed-from"
    );
    assert!(
        relationships.contains(&"attributed-to"),
        "missing attributed-to"
    );
    assert!(
        relationships.contains(&"chain-anchored"),
        "missing chain-anchored"
    );

    let spine_ids: std::collections::HashSet<_> = chain.iter().map(|l| l.spine_id).collect();
    assert_eq!(spine_ids.len(), 2, "expected links from 2 spines");

    let max_depth = chain.last().map_or(0, |l| l.depth);
    assert!(
        max_depth >= 5,
        "max provenance depth {max_depth} is below the 5+ target"
    );
}

/// Provenance chain depth with `certified-by` relationship via certificate mint.
#[tokio::test]
async fn test_provenance_chain_includes_certificate_mint() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkCertOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("CertProvenance".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert_id = crate::types::CertificateId::now_v7();

    let mint_hash = service
        .append_entry(
            spine_id,
            EntryType::CertificateMint {
                cert_id,
                cert_type: "provenance-test".into(),
                initial_owner: owner.clone(),
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let chain = service
        .get_provenance_chain(mint_hash)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0].relationship, "certified-by");
    assert_eq!(chain[0].depth, 0);
}

/// Provenance chain returns deterministic ordering when entries share timestamps.
#[tokio::test]
async fn test_provenance_chain_deterministic_ordering() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkDeterministic");
    let content_hash = [0xDDu8; 32];

    let spine_id = service
        .ensure_spine(owner.clone(), Some("OrderTest".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    for _ in 0..3 {
        service
            .append_entry(
                spine_id,
                EntryType::DataAnchor {
                    data_hash: content_hash,
                    mime_type: None,
                    size: 0,
                },
            )
            .await
            .unwrap_or_else(|_| unreachable!());
    }

    let chain = service
        .get_provenance_chain(content_hash)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_eq!(chain.len(), 3);
    for (i, link) in chain.iter().enumerate() {
        let expected = u32::try_from(i).unwrap_or(u32::MAX);
        assert_eq!(link.depth, expected);
    }
    for pair in chain.windows(2) {
        assert!(pair[0].index < pair[1].index);
    }
}
