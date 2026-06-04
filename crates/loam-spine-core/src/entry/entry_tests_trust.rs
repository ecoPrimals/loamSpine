// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for cross-gate trust entry types (KeyExchange, TrustIssuerRegistration,
//! TokenVerificationCrossGate).

use super::*;
use crate::types::Did;

#[test]
fn entry_type_domain_cross_gate_trust() {
    let local = Did::new("did:key:z6MkLocalGate");
    let remote = Did::new("did:key:z6MkRemoteGate");

    assert_eq!(
        EntryType::KeyExchange {
            local_gate: local.clone(),
            remote_gate: remote.clone(),
            public_key_hash: [1u8; 32],
            direction: "initiated".into(),
            family_id: Some("family-alpha".into()),
        }
        .domain(),
        "trust"
    );

    assert_eq!(
        EntryType::TrustIssuerRegistration {
            issuer_did: remote.clone(),
            registering_gate: local.clone(),
            trust_scope: "cross-gate".into(),
            capabilities: vec!["signing".into(), "verification".into()],
            expires_at: None,
        }
        .domain(),
        "trust"
    );

    assert_eq!(
        EntryType::TokenVerificationCrossGate {
            issuer_gate: remote,
            verifier_gate: local,
            token_hash: [2u8; 32],
            verified: true,
            failure_reason: None,
        }
        .domain(),
        "trust"
    );
}

#[test]
fn cross_gate_trust_not_allowed_in_waypoint() {
    let gate = Did::new("did:key:z6MkGate");
    assert!(!EntryType::KeyExchange {
        local_gate: gate.clone(),
        remote_gate: gate.clone(),
        public_key_hash: [0u8; 32],
        direction: "initiated".into(),
        family_id: None,
    }
    .allowed_in_waypoint());

    assert!(!EntryType::TrustIssuerRegistration {
        issuer_did: gate.clone(),
        registering_gate: gate.clone(),
        trust_scope: "family".into(),
        capabilities: vec![],
        expires_at: None,
    }
    .allowed_in_waypoint());

    assert!(!EntryType::TokenVerificationCrossGate {
        issuer_gate: gate.clone(),
        verifier_gate: gate,
        token_hash: [0u8; 32],
        verified: false,
        failure_reason: Some("expired".into()),
    }
    .allowed_in_waypoint());
}

#[test]
fn entry_type_serde_roundtrip_key_exchange() {
    let ty = EntryType::KeyExchange {
        local_gate: Did::new("did:key:z6MkStrandGate"),
        remote_gate: Did::new("did:key:z6MkBiomeGate"),
        public_key_hash: [0xab; 32],
        direction: "initiated".into(),
        family_id: Some("family-alpha".into()),
    };
    let json = serde_json::to_vec(&ty).expect("serialize");
    let restored: EntryType = serde_json::from_slice(&json).expect("deserialize");
    assert!(matches!(
        restored,
        EntryType::KeyExchange { direction, .. } if direction == "initiated"
    ));
}

#[test]
fn entry_type_serde_roundtrip_trust_issuer_registration() {
    let ty = EntryType::TrustIssuerRegistration {
        issuer_did: Did::new("did:key:z6MkIssuer"),
        registering_gate: Did::new("did:key:z6MkStrandGate"),
        trust_scope: "cross-gate".into(),
        capabilities: vec!["signing".into(), "verification".into()],
        expires_at: Some(Timestamp::now()),
    };
    let json = serde_json::to_vec(&ty).expect("serialize");
    let restored: EntryType = serde_json::from_slice(&json).expect("deserialize");
    assert!(matches!(
        restored,
        EntryType::TrustIssuerRegistration { trust_scope, .. } if trust_scope == "cross-gate"
    ));
}

#[test]
fn entry_type_serde_roundtrip_token_verification_cross_gate() {
    let ty = EntryType::TokenVerificationCrossGate {
        issuer_gate: Did::new("did:key:z6MkIssuerGate"),
        verifier_gate: Did::new("did:key:z6MkVerifierGate"),
        token_hash: [0xcd; 32],
        verified: true,
        failure_reason: None,
    };
    let json = serde_json::to_vec(&ty).expect("serialize");
    let restored: EntryType = serde_json::from_slice(&json).expect("deserialize");
    assert!(matches!(
        restored,
        EntryType::TokenVerificationCrossGate { verified: true, .. }
    ));
}

#[test]
fn entry_type_serde_roundtrip_token_verification_failed() {
    let ty = EntryType::TokenVerificationCrossGate {
        issuer_gate: Did::new("did:key:z6MkIssuerGate"),
        verifier_gate: Did::new("did:key:z6MkVerifierGate"),
        token_hash: [0xef; 32],
        verified: false,
        failure_reason: Some("signature mismatch".into()),
    };
    let json = serde_json::to_vec(&ty).expect("serialize");
    let restored: EntryType = serde_json::from_slice(&json).expect("deserialize");
    assert!(matches!(
        &restored,
        EntryType::TokenVerificationCrossGate { verified: false, failure_reason: Some(reason), .. }
            if reason == "signature mismatch"
    ));
}

#[test]
fn entry_type_serde_key_exchange_no_family() {
    let ty = EntryType::KeyExchange {
        local_gate: Did::new("did:key:z6MkLocal"),
        remote_gate: Did::new("did:key:z6MkRemote"),
        public_key_hash: [0xff; 32],
        direction: "accepted".into(),
        family_id: None,
    };
    let json = serde_json::to_string(&ty).expect("serialize");
    assert!(!json.contains("family_id"));
    let restored: EntryType = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(
        restored,
        EntryType::KeyExchange { family_id: None, direction, .. } if direction == "accepted"
    ));
}

#[test]
fn entry_type_serde_trust_issuer_no_expiry() {
    let ty = EntryType::TrustIssuerRegistration {
        issuer_did: Did::new("did:key:z6MkIssuer"),
        registering_gate: Did::new("did:key:z6MkGate"),
        trust_scope: "global".into(),
        capabilities: vec!["signing".into()],
        expires_at: None,
    };
    let json = serde_json::to_string(&ty).expect("serialize");
    assert!(!json.contains("expires_at"));
    let restored: EntryType = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(
        restored,
        EntryType::TrustIssuerRegistration { expires_at: None, .. }
    ));
}

#[test]
fn cross_gate_entry_full_roundtrip() {
    let did = Did::new("did:key:z6MkCommitter");
    let entry = Entry::new(
        5,
        Some([0xaa; 32]),
        did,
        EntryType::KeyExchange {
            local_gate: Did::new("did:key:z6MkStrandGate"),
            remote_gate: Did::new("did:key:z6MkBiomeGate"),
            public_key_hash: [0xbb; 32],
            direction: "initiated".into(),
            family_id: Some("family-alpha".into()),
        },
    )
    .with_spine_id(SpineId::now_v7())
    .with_metadata("wave", "76");

    let bytes = entry.to_canonical_bytes().expect("canonical");
    assert!(!bytes.is_empty());

    let json = serde_json::to_vec(&entry).expect("serialize");
    let restored: Entry = serde_json::from_slice(&json).expect("deserialize");
    assert_eq!(restored.index, 5);
    assert_eq!(restored.domain(), "trust");
    assert_eq!(restored.metadata.get("wave"), Some(&"76".to_string()));
}
