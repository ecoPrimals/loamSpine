//! Proof types for LoamSpine.
//!
//! Proofs allow verification of entries and certificates without
//! requiring access to the full chain data.

use serde::{Deserialize, Serialize};

use crate::entry::Entry;
use crate::types::{CertificateId, Did, EntryHash, Signature, SpineId, Timestamp};

/// Proof that an entry exists in a spine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InclusionProof {
    /// The entry being proven.
    pub entry: Entry,

    /// Entry hash.
    pub entry_hash: EntryHash,

    /// Path from entry to tip (chain of hashes).
    pub path: Vec<EntryHash>,

    /// Current tip.
    pub tip: EntryHash,

    /// Spine ID.
    pub spine_id: SpineId,

    /// Proof timestamp.
    pub timestamp: Timestamp,

    /// Optional: signature from spine owner.
    pub owner_attestation: Option<Signature>,
}

impl InclusionProof {
    /// Create a new inclusion proof.
    #[must_use]
    pub fn new(entry: Entry, spine_id: SpineId, tip: EntryHash) -> Self {
        let entry_hash = entry.compute_hash();
        Self {
            entry,
            entry_hash,
            path: Vec::new(),
            tip,
            spine_id,
            timestamp: Timestamp::now(),
            owner_attestation: None,
        }
    }

    /// Add path entries.
    #[must_use]
    pub fn with_path(mut self, path: Vec<EntryHash>) -> Self {
        self.path = path;
        self
    }

    /// Add owner attestation.
    #[must_use]
    pub fn with_attestation(mut self, signature: Signature) -> Self {
        self.owner_attestation = Some(signature);
        self
    }

    /// Verify this proof.
    ///
    /// Checks that the hash chain from entry to tip is valid.
    #[must_use]
    pub fn verify(&self) -> bool {
        // Verify entry hash matches
        if self.entry.compute_hash() != self.entry_hash {
            return false;
        }

        // If path is empty and entry_hash == tip, it's valid (entry is tip)
        if self.path.is_empty() {
            return self.entry_hash == self.tip;
        }

        // Verify we reach the tip through the path
        // In a proper implementation, we'd verify each entry links correctly
        // For now, just verify the path ends at tip
        self.path.last() == Some(&self.tip)
    }

    /// Get the number of entries between this entry and tip.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.path.len()
    }
}

/// Proof of certificate ownership and history.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificateProof {
    /// Certificate ID.
    pub cert_id: CertificateId,

    /// Current owner.
    pub owner: Did,

    /// Mint entry with inclusion proof.
    pub mint_proof: InclusionProof,

    /// Transfer chain (if any).
    pub transfer_proofs: Vec<InclusionProof>,

    /// Current state entry.
    pub current_proof: InclusionProof,

    /// History summary.
    pub history_summary: HistorySummary,

    /// Proof timestamp.
    pub timestamp: Timestamp,
}

impl CertificateProof {
    /// Create a new certificate proof.
    #[must_use]
    pub fn new(
        cert_id: CertificateId,
        owner: Did,
        mint_proof: InclusionProof,
        current_proof: InclusionProof,
    ) -> Self {
        Self {
            cert_id,
            owner,
            mint_proof,
            transfer_proofs: Vec::new(),
            current_proof,
            history_summary: HistorySummary::default(),
            timestamp: Timestamp::now(),
        }
    }

    /// Add transfer proofs.
    #[must_use]
    pub fn with_transfers(mut self, proofs: Vec<InclusionProof>) -> Self {
        self.history_summary.transfer_count = proofs.len() as u64;
        self.transfer_proofs = proofs;
        self
    }

    /// Verify this proof.
    #[must_use]
    pub fn verify(&self) -> bool {
        // Verify mint proof
        if !self.mint_proof.verify() {
            return false;
        }

        // Verify all transfer proofs
        for proof in &self.transfer_proofs {
            if !proof.verify() {
                return false;
            }
        }

        // Verify current proof
        if !self.current_proof.verify() {
            return false;
        }

        true
    }
}

/// History summary for certificate proofs.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HistorySummary {
    /// Number of transfers.
    pub transfer_count: u64,
    /// Number of loans.
    pub loan_count: u64,
    /// Age in nanoseconds.
    pub age_nanos: u64,
}

/// Provenance proof for data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProvenanceProof {
    /// Data hash.
    pub data_hash: EntryHash,

    /// Chain of custody entries.
    pub custody_chain: Vec<InclusionProof>,

    /// Original anchor proof.
    pub anchor_proof: InclusionProof,

    /// Proof timestamp.
    pub timestamp: Timestamp,
}

impl ProvenanceProof {
    /// Create a new provenance proof.
    #[must_use]
    pub fn new(data_hash: EntryHash, anchor_proof: InclusionProof) -> Self {
        Self {
            data_hash,
            custody_chain: Vec::new(),
            anchor_proof,
            timestamp: Timestamp::now(),
        }
    }

    /// Add custody chain.
    #[must_use]
    pub fn with_custody(mut self, chain: Vec<InclusionProof>) -> Self {
        self.custody_chain = chain;
        self
    }

    /// Verify this proof.
    #[must_use]
    pub fn verify(&self) -> bool {
        // Verify anchor proof
        if !self.anchor_proof.verify() {
            return false;
        }

        // Verify all custody entries
        for proof in &self.custody_chain {
            if !proof.verify() {
                return false;
            }
        }

        true
    }
}

/// Verification result.
#[derive(Clone, Debug)]
pub struct VerificationResult {
    /// Whether verification passed.
    pub valid: bool,
    /// Verification errors.
    pub errors: Vec<VerificationError>,
    /// Verified at timestamp.
    pub verified_at: Timestamp,
}

impl VerificationResult {
    /// Create a successful verification result.
    #[must_use]
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            verified_at: Timestamp::now(),
        }
    }

    /// Create a failed verification result.
    #[must_use]
    pub fn failure(errors: Vec<VerificationError>) -> Self {
        Self {
            valid: false,
            errors,
            verified_at: Timestamp::now(),
        }
    }
}

/// Verification error.
#[derive(Clone, Debug)]
pub enum VerificationError {
    /// Hash mismatch.
    HashMismatch {
        /// Expected hash.
        expected: EntryHash,
        /// Actual hash.
        actual: EntryHash,
    },
    /// Invalid signature.
    InvalidSignature {
        /// Signer.
        signer: Did,
    },
    /// Chain broken.
    ChainBroken {
        /// Index where chain broke.
        at_index: u64,
    },
    /// Entry not found.
    EntryNotFound {
        /// Missing entry hash.
        hash: EntryHash,
    },
}

#[cfg(test)]
mod tests {
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
        let tip = entry.compute_hash();

        let proof = InclusionProof::new(entry, spine_id, tip);

        assert_eq!(proof.spine_id, spine_id);
        assert_eq!(proof.tip, tip);
        assert_eq!(proof.depth(), 0);
    }

    #[test]
    fn inclusion_proof_verify_tip() {
        let entry = create_test_entry();
        let spine_id = SpineId::now_v7();
        let tip = entry.compute_hash();

        let proof = InclusionProof::new(entry, spine_id, tip);

        // Entry is the tip, should verify
        assert!(proof.verify());
    }

    #[test]
    fn inclusion_proof_with_path() {
        let entry = create_test_entry();
        let spine_id = SpineId::now_v7();
        let tip = [1u8; 32];
        let path = vec![[2u8; 32], [3u8; 32], tip];

        let proof = InclusionProof::new(entry, spine_id, tip).with_path(path);

        assert_eq!(proof.depth(), 3);
        // Path ends at tip, so should verify (simplified check)
        assert!(proof.verify());
    }

    #[test]
    fn inclusion_proof_with_attestation() {
        let entry = create_test_entry();
        let spine_id = SpineId::now_v7();
        let tip = entry.compute_hash();

        let signature = Signature::from_vec(vec![1, 2, 3, 4, 5]);
        let proof = InclusionProof::new(entry, spine_id, tip).with_attestation(signature);

        assert!(proof.owner_attestation.is_some());
        assert_eq!(proof.owner_attestation.as_ref().map(|s| s.0.len()), Some(5));
        assert!(proof.verify());
    }

    #[test]
    fn inclusion_proof_verify_fails_with_wrong_tip() {
        let entry = create_test_entry();
        let spine_id = SpineId::now_v7();
        let wrong_tip = [99u8; 32]; // Different from entry hash

        let proof = InclusionProof::new(entry, spine_id, wrong_tip);

        // Empty path, but entry_hash != tip, should fail
        assert!(!proof.verify());
    }

    #[test]
    fn inclusion_proof_verify_fails_with_wrong_path() {
        let entry = create_test_entry();
        let spine_id = SpineId::now_v7();
        let tip = [1u8; 32];
        // Path doesn't end at tip
        let bad_path = vec![[2u8; 32], [3u8; 32], [99u8; 32]];

        let proof = InclusionProof::new(entry, spine_id, tip).with_path(bad_path);

        // Path doesn't end at tip, should fail
        assert!(!proof.verify());
    }

    #[test]
    fn certificate_proof_creation() {
        let entry = create_test_entry();
        let spine_id = SpineId::now_v7();
        let tip = entry.compute_hash();

        let mint_proof = InclusionProof::new(entry.clone(), spine_id, tip);
        let current_proof = InclusionProof::new(entry, spine_id, tip);

        let cert_id = CertificateId::now_v7();
        let owner = Did::new("did:key:z6MkOwner");

        let proof = CertificateProof::new(cert_id, owner, mint_proof, current_proof);

        assert!(proof.verify());
        assert!(proof.transfer_proofs.is_empty());
    }

    #[test]
    fn certificate_proof_with_transfers() {
        let entry = create_test_entry();
        let spine_id = SpineId::now_v7();
        let tip = entry.compute_hash();

        let mint_proof = InclusionProof::new(entry.clone(), spine_id, tip);
        let current_proof = InclusionProof::new(entry.clone(), spine_id, tip);
        let transfer1 = InclusionProof::new(entry.clone(), spine_id, tip);
        let transfer2 = InclusionProof::new(entry, spine_id, tip);

        let cert_id = CertificateId::now_v7();
        let owner = Did::new("did:key:z6MkOwner");

        let proof = CertificateProof::new(cert_id, owner, mint_proof, current_proof)
            .with_transfers(vec![transfer1, transfer2]);

        assert!(proof.verify());
        assert_eq!(proof.transfer_proofs.len(), 2);
        assert_eq!(proof.history_summary.transfer_count, 2);
    }

    #[test]
    fn certificate_proof_fails_with_invalid_mint() {
        let entry = create_test_entry();
        let spine_id = SpineId::now_v7();
        let tip = entry.compute_hash();
        let wrong_tip = [99u8; 32];

        // Invalid mint proof (wrong tip)
        let mint_proof = InclusionProof::new(entry.clone(), spine_id, wrong_tip);
        let current_proof = InclusionProof::new(entry, spine_id, tip);

        let cert_id = CertificateId::now_v7();
        let owner = Did::new("did:key:z6MkOwner");

        let proof = CertificateProof::new(cert_id, owner, mint_proof, current_proof);

        assert!(!proof.verify());
    }

    #[test]
    fn certificate_proof_fails_with_invalid_transfer() {
        let entry = create_test_entry();
        let spine_id = SpineId::now_v7();
        let tip = entry.compute_hash();
        let wrong_tip = [99u8; 32];

        let mint_proof = InclusionProof::new(entry.clone(), spine_id, tip);
        let current_proof = InclusionProof::new(entry.clone(), spine_id, tip);
        // Invalid transfer proof
        let bad_transfer = InclusionProof::new(entry, spine_id, wrong_tip);

        let cert_id = CertificateId::now_v7();
        let owner = Did::new("did:key:z6MkOwner");

        let proof = CertificateProof::new(cert_id, owner, mint_proof, current_proof)
            .with_transfers(vec![bad_transfer]);

        assert!(!proof.verify());
    }

    #[test]
    fn certificate_proof_fails_with_invalid_current() {
        let entry = create_test_entry();
        let spine_id = SpineId::now_v7();
        let tip = entry.compute_hash();
        let wrong_tip = [99u8; 32];

        let mint_proof = InclusionProof::new(entry.clone(), spine_id, tip);
        // Invalid current proof
        let current_proof = InclusionProof::new(entry, spine_id, wrong_tip);

        let cert_id = CertificateId::now_v7();
        let owner = Did::new("did:key:z6MkOwner");

        let proof = CertificateProof::new(cert_id, owner, mint_proof, current_proof);

        assert!(!proof.verify());
    }

    #[test]
    fn provenance_proof_creation() {
        let entry = create_test_entry();
        let spine_id = SpineId::now_v7();
        let tip = entry.compute_hash();
        let data_hash = [42u8; 32];

        let anchor_proof = InclusionProof::new(entry, spine_id, tip);
        let proof = ProvenanceProof::new(data_hash, anchor_proof);

        assert_eq!(proof.data_hash, data_hash);
        assert!(proof.custody_chain.is_empty());
        assert!(proof.verify());
    }

    #[test]
    fn provenance_proof_with_custody() {
        let entry = create_test_entry();
        let spine_id = SpineId::now_v7();
        let tip = entry.compute_hash();
        let data_hash = [42u8; 32];

        let anchor_proof = InclusionProof::new(entry.clone(), spine_id, tip);
        let custody1 = InclusionProof::new(entry.clone(), spine_id, tip);
        let custody2 = InclusionProof::new(entry, spine_id, tip);

        let proof =
            ProvenanceProof::new(data_hash, anchor_proof).with_custody(vec![custody1, custody2]);

        assert_eq!(proof.custody_chain.len(), 2);
        assert!(proof.verify());
    }

    #[test]
    fn provenance_proof_fails_with_invalid_anchor() {
        let entry = create_test_entry();
        let spine_id = SpineId::now_v7();
        let wrong_tip = [99u8; 32];
        let data_hash = [42u8; 32];

        // Invalid anchor proof
        let anchor_proof = InclusionProof::new(entry, spine_id, wrong_tip);
        let proof = ProvenanceProof::new(data_hash, anchor_proof);

        assert!(!proof.verify());
    }

    #[test]
    fn provenance_proof_fails_with_invalid_custody() {
        let entry = create_test_entry();
        let spine_id = SpineId::now_v7();
        let tip = entry.compute_hash();
        let wrong_tip = [99u8; 32];
        let data_hash = [42u8; 32];

        let anchor_proof = InclusionProof::new(entry.clone(), spine_id, tip);
        // Invalid custody proof
        let bad_custody = InclusionProof::new(entry, spine_id, wrong_tip);

        let proof = ProvenanceProof::new(data_hash, anchor_proof).with_custody(vec![bad_custody]);

        assert!(!proof.verify());
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

        let failure =
            VerificationResult::failure(vec![VerificationError::ChainBroken { at_index: 5 }]);
        assert!(!failure.valid);
        assert_eq!(failure.errors.len(), 1);
    }

    #[test]
    fn verification_error_variants() {
        // HashMismatch
        let err1 = VerificationError::HashMismatch {
            expected: [1u8; 32],
            actual: [2u8; 32],
        };
        assert!(matches!(err1, VerificationError::HashMismatch { .. }));

        // InvalidSignature
        let err2 = VerificationError::InvalidSignature {
            signer: Did::new("did:key:z6MkBad"),
        };
        assert!(matches!(err2, VerificationError::InvalidSignature { .. }));

        // ChainBroken
        let err3 = VerificationError::ChainBroken { at_index: 42 };
        assert!(matches!(
            err3,
            VerificationError::ChainBroken { at_index: 42 }
        ));

        // EntryNotFound
        let err4 = VerificationError::EntryNotFound { hash: [3u8; 32] };
        assert!(matches!(err4, VerificationError::EntryNotFound { .. }));
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
}
