// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::entry::SpineConfig;
use crate::types::{Did, Signature};

fn create_test_entry() -> Entry {
    let owner = Did::new("did:key:z6MkOwner");
    Entry::genesis(owner, SpineId::now_v7(), SpineConfig::default())
}

#[test]
fn inclusion_proof_creation() {
    let entry = create_test_entry();
    let spine_id = SpineId::now_v7();
    let tip = entry.compute_hash().expect("compute_hash");

    let proof = InclusionProof::new(entry, spine_id, tip).expect("new proof");

    assert_eq!(proof.spine_id, spine_id);
    assert_eq!(proof.tip, tip);
    assert_eq!(proof.depth(), 0);
}

#[test]
fn inclusion_proof_verify_tip() {
    let entry = create_test_entry();
    let spine_id = SpineId::now_v7();
    let tip = entry.compute_hash().expect("compute_hash");

    let proof = InclusionProof::new(entry, spine_id, tip).expect("new proof");

    assert!(proof.verify().expect("verify"));
}

#[test]
fn inclusion_proof_with_path() {
    let entry = create_test_entry();
    let spine_id = SpineId::now_v7();
    let tip = [1u8; 32];
    let path = vec![[2u8; 32], [3u8; 32], tip];

    let proof = InclusionProof::new(entry, spine_id, tip)
        .expect("new proof")
        .with_path(path);

    assert_eq!(proof.depth(), 3);
    assert!(proof.verify().expect("verify"));
}

#[test]
fn inclusion_proof_with_attestation() {
    let entry = create_test_entry();
    let spine_id = SpineId::now_v7();
    let tip = entry.compute_hash().expect("compute_hash");

    let signature = Signature::from_vec(vec![1, 2, 3, 4, 5]);
    let proof = InclusionProof::new(entry, spine_id, tip)
        .expect("new proof")
        .with_attestation(signature);

    assert!(proof.owner_attestation.is_some());
    assert_eq!(proof.owner_attestation.as_ref().map(|s| s.0.len()), Some(5));
    assert!(proof.verify().expect("verify"));
}

#[test]
fn inclusion_proof_verify_fails_with_wrong_tip() {
    let entry = create_test_entry();
    let spine_id = SpineId::now_v7();
    let wrong_tip = [99u8; 32];

    let proof = InclusionProof::new(entry, spine_id, wrong_tip).expect("new proof");

    assert!(!proof.verify().expect("verify"));
}

#[test]
fn inclusion_proof_verify_fails_with_wrong_path() {
    let entry = create_test_entry();
    let spine_id = SpineId::now_v7();
    let tip = [1u8; 32];
    let bad_path = vec![[2u8; 32], [3u8; 32], [99u8; 32]];

    let proof = InclusionProof::new(entry, spine_id, tip)
        .expect("new proof")
        .with_path(bad_path);

    assert!(!proof.verify().expect("verify"));
}

#[test]
fn certificate_proof_creation() {
    let entry = create_test_entry();
    let spine_id = SpineId::now_v7();
    let tip = entry.compute_hash().expect("compute_hash");

    let mint_proof = InclusionProof::new(entry.clone(), spine_id, tip).expect("new proof");
    let current_proof = InclusionProof::new(entry, spine_id, tip).expect("new proof");

    let cert_id = CertificateId::now_v7();
    let owner = Did::new("did:key:z6MkOwner");

    let proof = CertificateProof::new(cert_id, owner, mint_proof, current_proof);

    assert!(proof.verify().expect("verify"));
    assert!(proof.transfer_proofs.is_empty());
}

#[test]
fn certificate_proof_with_transfers() {
    let entry = create_test_entry();
    let spine_id = SpineId::now_v7();
    let tip = entry.compute_hash().expect("compute_hash");

    let mint_proof = InclusionProof::new(entry.clone(), spine_id, tip).expect("new proof");
    let current_proof = InclusionProof::new(entry.clone(), spine_id, tip).expect("new proof");
    let transfer1 = InclusionProof::new(entry.clone(), spine_id, tip).expect("new proof");
    let transfer2 = InclusionProof::new(entry, spine_id, tip).expect("new proof");

    let cert_id = CertificateId::now_v7();
    let owner = Did::new("did:key:z6MkOwner");

    let proof = CertificateProof::new(cert_id, owner, mint_proof, current_proof)
        .with_transfers(vec![transfer1, transfer2]);

    assert!(proof.verify().expect("verify"));
    assert_eq!(proof.transfer_proofs.len(), 2);
    assert_eq!(proof.history_summary.transfer_count, 2);
}

#[test]
fn certificate_proof_fails_with_invalid_mint() {
    let entry = create_test_entry();
    let spine_id = SpineId::now_v7();
    let tip = entry.compute_hash().expect("compute_hash");
    let wrong_tip = [99u8; 32];

    let mint_proof = InclusionProof::new(entry.clone(), spine_id, wrong_tip).expect("new proof");
    let current_proof = InclusionProof::new(entry, spine_id, tip).expect("new proof");

    let cert_id = CertificateId::now_v7();
    let owner = Did::new("did:key:z6MkOwner");

    let proof = CertificateProof::new(cert_id, owner, mint_proof, current_proof);

    assert!(!proof.verify().expect("verify"));
}

#[test]
fn certificate_proof_fails_with_invalid_transfer() {
    let entry = create_test_entry();
    let spine_id = SpineId::now_v7();
    let tip = entry.compute_hash().expect("compute_hash");
    let wrong_tip = [99u8; 32];

    let mint_proof = InclusionProof::new(entry.clone(), spine_id, tip).expect("new proof");
    let current_proof = InclusionProof::new(entry.clone(), spine_id, tip).expect("new proof");
    let bad_transfer = InclusionProof::new(entry, spine_id, wrong_tip).expect("new proof");

    let cert_id = CertificateId::now_v7();
    let owner = Did::new("did:key:z6MkOwner");

    let proof = CertificateProof::new(cert_id, owner, mint_proof, current_proof)
        .with_transfers(vec![bad_transfer]);

    assert!(!proof.verify().expect("verify"));
}

#[test]
fn certificate_proof_fails_with_invalid_current() {
    let entry = create_test_entry();
    let spine_id = SpineId::now_v7();
    let tip = entry.compute_hash().expect("compute_hash");
    let wrong_tip = [99u8; 32];

    let mint_proof = InclusionProof::new(entry.clone(), spine_id, tip).expect("new proof");
    let current_proof = InclusionProof::new(entry, spine_id, wrong_tip).expect("new proof");

    let cert_id = CertificateId::now_v7();
    let owner = Did::new("did:key:z6MkOwner");

    let proof = CertificateProof::new(cert_id, owner, mint_proof, current_proof);

    assert!(!proof.verify().expect("verify"));
}

#[test]
fn provenance_proof_creation() {
    let entry = create_test_entry();
    let spine_id = SpineId::now_v7();
    let tip = entry.compute_hash().expect("compute_hash");
    let data_hash = [42u8; 32];

    let anchor_proof = InclusionProof::new(entry, spine_id, tip).expect("new proof");
    let proof = ProvenanceProof::new(data_hash, anchor_proof);

    assert_eq!(proof.data_hash, data_hash);
    assert!(proof.custody_chain.is_empty());
    assert!(proof.verify().expect("verify"));
}

#[test]
fn provenance_proof_with_custody() {
    let entry = create_test_entry();
    let spine_id = SpineId::now_v7();
    let tip = entry.compute_hash().expect("compute_hash");
    let data_hash = [42u8; 32];

    let anchor_proof = InclusionProof::new(entry.clone(), spine_id, tip).expect("new proof");
    let custody1 = InclusionProof::new(entry.clone(), spine_id, tip).expect("new proof");
    let custody2 = InclusionProof::new(entry, spine_id, tip).expect("new proof");

    let proof =
        ProvenanceProof::new(data_hash, anchor_proof).with_custody(vec![custody1, custody2]);

    assert_eq!(proof.custody_chain.len(), 2);
    assert!(proof.verify().expect("verify"));
}

#[test]
fn provenance_proof_fails_with_invalid_anchor() {
    let entry = create_test_entry();
    let spine_id = SpineId::now_v7();
    let wrong_tip = [99u8; 32];
    let data_hash = [42u8; 32];

    let anchor_proof = InclusionProof::new(entry, spine_id, wrong_tip).expect("new proof");
    let proof = ProvenanceProof::new(data_hash, anchor_proof);

    assert!(!proof.verify().expect("verify"));
}

#[test]
fn provenance_proof_fails_with_invalid_custody() {
    let entry = create_test_entry();
    let spine_id = SpineId::now_v7();
    let tip = entry.compute_hash().expect("compute_hash");
    let wrong_tip = [99u8; 32];
    let data_hash = [42u8; 32];

    let anchor_proof = InclusionProof::new(entry.clone(), spine_id, tip).expect("new proof");
    let bad_custody = InclusionProof::new(entry, spine_id, wrong_tip).expect("new proof");

    let proof = ProvenanceProof::new(data_hash, anchor_proof).with_custody(vec![bad_custody]);

    assert!(!proof.verify().expect("verify"));
}

#[test]
fn history_summary_default() {
    let summary = HistorySummary::default();
    assert_eq!(summary.transfer_count, 0);
    assert_eq!(summary.loan_count, 0);
    assert_eq!(summary.age_nanos, 0);
}

#[test]
fn verification_result() {
    let success = VerificationResult::success();
    assert!(success.valid);
    assert!(success.errors.is_empty());

    let failure = VerificationResult::failure(vec![VerificationError::ChainBroken { at_index: 5 }]);
    assert!(!failure.valid);
    assert_eq!(failure.errors.len(), 1);
}

#[test]
fn verification_error_variants() {
    let err1 = VerificationError::HashMismatch {
        expected: [1u8; 32],
        actual: [2u8; 32],
    };
    assert!(matches!(err1, VerificationError::HashMismatch { .. }));

    let err2 = VerificationError::InvalidSignature {
        signer: Did::new("did:key:z6MkBad"),
    };
    assert!(matches!(err2, VerificationError::InvalidSignature { .. }));

    let err3 = VerificationError::ChainBroken { at_index: 42 };
    assert!(matches!(
        err3,
        VerificationError::ChainBroken { at_index: 42 }
    ));

    let err4 = VerificationError::EntryNotFound { hash: [3u8; 32] };
    assert!(matches!(err4, VerificationError::EntryNotFound { .. }));
}

#[test]
fn certificate_ownership_proof_verify() {
    let cert_id = uuid::Uuid::now_v7();
    let entries = vec![[1u8; 32], [2u8; 32], [3u8; 32]];
    let chain_root = super::compute_merkle_root(&entries);

    let proof = super::CertificateOwnershipProof {
        certificate_id: cert_id,
        chain_root,
        chain_length: 3,
        entries,
        created_at: crate::types::Timestamp::now(),
    };

    assert!(proof.verify().expect("verify"));
}

#[test]
fn certificate_ownership_proof_verify_fails_tampered() {
    let cert_id = uuid::Uuid::now_v7();
    let entries = vec![[1u8; 32], [2u8; 32]];
    let _chain_root = super::compute_merkle_root(&entries);

    let mut proof = super::CertificateOwnershipProof {
        certificate_id: cert_id,
        chain_root: [99_u8; 32],
        chain_length: 2,
        entries,
        created_at: crate::types::Timestamp::now(),
    };

    assert!(!proof.verify().expect("verify"));

    proof.chain_root = super::compute_merkle_root(&proof.entries);
    assert!(proof.verify().expect("verify"));
}

#[test]
fn certificate_ownership_proof_empty_chain() {
    let cert_id = uuid::Uuid::now_v7();
    let entries: Vec<crate::types::EntryHash> = vec![];
    let chain_root = super::compute_merkle_root(&entries);

    let proof = super::CertificateOwnershipProof {
        certificate_id: cert_id,
        chain_root,
        chain_length: 0,
        entries: vec![],
        created_at: crate::types::Timestamp::now(),
    };

    assert!(proof.verify().expect("verify"));
}

#[test]
fn verification_result_multiple_errors() {
    let errors = vec![
        VerificationError::ChainBroken { at_index: 5 },
        VerificationError::InvalidSignature {
            signer: Did::new("did:key:z6MkBad"),
        },
        VerificationError::HashMismatch {
            expected: [0u8; 32],
            actual: [1u8; 32],
        },
    ];
    let failure = VerificationResult::failure(errors);
    assert!(!failure.valid);
    assert_eq!(failure.errors.len(), 3);
}
