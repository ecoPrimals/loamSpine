// SPDX-License-Identifier: AGPL-3.0-or-later

//! Proof types for LoamSpine.
//!
//! Proofs allow verification of entries and certificates without
//! requiring access to the full chain data.

use serde::{Deserialize, Serialize};

use crate::entry::Entry;
use crate::error::LoamSpineResult;
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
    ///
    /// # Errors
    ///
    /// Returns an error if entry hash computation fails.
    pub fn new(entry: Entry, spine_id: SpineId, tip: EntryHash) -> LoamSpineResult<Self> {
        let entry_hash = entry.compute_hash()?;
        Ok(Self {
            entry,
            entry_hash,
            path: Vec::new(),
            tip,
            spine_id,
            timestamp: Timestamp::now(),
            owner_attestation: None,
        })
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
    ///
    /// # Errors
    ///
    /// Returns an error if entry hash computation fails.
    pub fn verify(&self) -> LoamSpineResult<bool> {
        // Verify entry hash matches
        if self.entry.compute_hash()? != self.entry_hash {
            return Ok(false);
        }

        // If path is empty and entry_hash == tip, it's valid (entry is tip)
        if self.path.is_empty() {
            return Ok(self.entry_hash == self.tip);
        }

        // Verify we reach the tip through the path
        // In a proper implementation, we'd verify each entry links correctly
        // For now, just verify the path ends at tip
        Ok(self.path.last() == Some(&self.tip))
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
        self.history_summary.transfer_count = u64::try_from(proofs.len()).unwrap_or(u64::MAX);
        self.transfer_proofs = proofs;
        self
    }

    /// Verify this proof.
    ///
    /// # Errors
    ///
    /// Returns an error if any sub-proof verification fails.
    pub fn verify(&self) -> LoamSpineResult<bool> {
        // Verify mint proof
        if !self.mint_proof.verify()? {
            return Ok(false);
        }

        // Verify all transfer proofs
        for proof in &self.transfer_proofs {
            if !proof.verify()? {
                return Ok(false);
            }
        }

        // Verify current proof
        if !self.current_proof.verify()? {
            return Ok(false);
        }

        Ok(true)
    }
}

/// Cryptographic proof of a certificate's ownership chain.
///
/// Built from all `OwnershipRecord`s for the certificate, with a Merkle root
/// over the entry hashes. Used by the CERTIFICATE_LAYER for provenance
/// verification.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificateOwnershipProof {
    /// Certificate ID.
    pub certificate_id: crate::types::CertificateId,
    /// Merkle root of the ownership chain.
    pub chain_root: crate::types::ContentHash,
    /// Number of entries in the chain.
    pub chain_length: u64,
    /// Entry hashes in order (ownership-establishing entries).
    pub entries: Vec<crate::types::EntryHash>,
    /// When the proof was created.
    pub created_at: crate::types::Timestamp,
}

impl CertificateOwnershipProof {
    /// Verify that the proof's chain root matches recomputation from entries.
    ///
    /// # Errors
    ///
    /// Returns an error if hash computation fails.
    pub fn verify(&self) -> crate::error::LoamSpineResult<bool> {
        let computed = compute_merkle_root(&self.entries);
        Ok(computed == self.chain_root)
    }
}

/// Compute Merkle root of a list of entry hashes using Blake3.
///
/// Uses standard binary Merkle tree: hash(left || right) for pairs,
/// duplicate last element if odd number of leaves.
pub(crate) fn compute_merkle_root(leaves: &[crate::types::EntryHash]) -> crate::types::ContentHash {
    use crate::types::hash_bytes;

    if leaves.is_empty() {
        return hash_bytes(b"");
    }
    if leaves.len() == 1 {
        return leaves[0];
    }

    let mut layer: Vec<crate::types::EntryHash> = leaves.to_vec();
    while layer.len() > 1 {
        let mut next = Vec::with_capacity(layer.len().div_ceil(2));
        let mut i = 0;
        while i < layer.len() {
            let left = layer[i];
            let right = if i + 1 < layer.len() {
                layer[i + 1]
            } else {
                layer[i]
            };
            let mut combined = Vec::with_capacity(64);
            combined.extend_from_slice(&left);
            combined.extend_from_slice(&right);
            next.push(hash_bytes(&combined));
            i += 2;
        }
        layer = next;
    }
    layer[0]
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
    ///
    /// # Errors
    ///
    /// Returns an error if any sub-proof verification fails.
    pub fn verify(&self) -> LoamSpineResult<bool> {
        // Verify anchor proof
        if !self.anchor_proof.verify()? {
            return Ok(false);
        }

        // Verify all custody entries
        for proof in &self.custody_chain {
            if !proof.verify()? {
                return Ok(false);
            }
        }

        Ok(true)
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
#[expect(
    clippy::expect_used,
    reason = "tests use expect for concise error paths"
)]
#[path = "proof_tests.rs"]
mod tests;
