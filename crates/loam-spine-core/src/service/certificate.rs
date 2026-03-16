// SPDX-License-Identifier: AGPL-3.0-or-later

//! Certificate lifecycle operations.
//!
//! This module provides certificate management functionality:
//! - **Mint**: Create new certificates
//! - **Transfer**: Permanent ownership change
//! - **Loan**: Temporary access grant
//! - **Return**: Loan completion
//!
//! ## Ownership Model
//!
//! Certificates follow a clear ownership model where only the current owner
//! can transfer or loan. Loans must be returned by the borrower.

use crate::certificate::{
    Certificate, CertificateLocation, CertificateMetadata, CertificateType, MintInfo,
};
use crate::entry::EntryType;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::proof::CertificateOwnershipProof;
use crate::storage::{CertificateStorage, EntryStorage, SpineStorage};
use crate::types::{CertificateId, Did, EntryHash, SpineId, Timestamp};

use super::LoamSpineService;

/// Result of certificate integrity verification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertificateVerification {
    /// Certificate ID that was verified.
    pub cert_id: CertificateId,
    /// Which checks passed (empty set = not found).
    pub passed: Vec<VerificationCheck>,
}

impl CertificateVerification {
    /// Whether the certificate exists at all.
    #[must_use]
    pub fn exists(&self) -> bool {
        self.passed.contains(&VerificationCheck::Exists)
    }

    /// Whether all checks passed (full chain integrity).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.passed.contains(&VerificationCheck::ChainValid)
    }
}

/// Individual verification checks for a certificate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerificationCheck {
    /// Certificate record exists in storage.
    Exists,
    /// Associated spine exists.
    SpineExists,
    /// Mint entry present in entry storage.
    MintEntryExists,
    /// Full chain (spine + mint + current location) is intact.
    ChainValid,
}

/// Check if an entry type references a specific certificate ID.
fn entry_references_certificate(
    entry_type: &crate::entry::EntryType,
    cert_id: CertificateId,
) -> bool {
    match entry_type {
        crate::entry::EntryType::CertificateMint { cert_id: id, .. }
        | crate::entry::EntryType::CertificateTransfer { cert_id: id, .. }
        | crate::entry::EntryType::CertificateLoan { cert_id: id, .. }
        | crate::entry::EntryType::CertificateReturn { cert_id: id, .. } => *id == cert_id,
        _ => false,
    }
}

impl LoamSpineService {
    // ========================================================================
    // Certificate Operations
    // ========================================================================

    /// Mint a new certificate.
    ///
    /// Creates a new certificate and records it in the spine.
    ///
    /// # Errors
    ///
    /// Returns error if spine not found or certificate creation fails.
    pub async fn mint_certificate(
        &self,
        spine_id: SpineId,
        cert_type: CertificateType,
        owner: Did,
        metadata: Option<CertificateMetadata>,
    ) -> LoamSpineResult<(CertificateId, EntryHash)> {
        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let cert_id = uuid::Uuid::now_v7();

        let cert_type_str = format!("{cert_type:?}");
        let entry = spine.create_entry(EntryType::CertificateMint {
            cert_id,
            cert_type: cert_type_str,
            initial_owner: owner.clone(),
        });

        let entry_hash = spine.append(entry)?;
        let appended = spine
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;

        let mint_info = MintInfo {
            minter: owner.clone(),
            spine: spine_id,
            entry: entry_hash,
            timestamp: Timestamp::now(),
            authority: None,
        };

        let mut cert = Certificate::new(cert_id, cert_type, &owner, &mint_info);
        cert.current_location = CertificateLocation {
            spine: spine_id,
            entry: entry_hash,
            index: spine.height - 1,
        };

        if let Some(meta) = metadata {
            cert.metadata = meta;
        }

        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&spine).await?;
        self.certificate_storage
            .save_certificate(&cert, spine_id)
            .await?;

        Ok((cert_id, entry_hash))
    }

    /// Get a certificate by ID.
    pub async fn get_certificate(&self, cert_id: CertificateId) -> Option<Certificate> {
        self.certificate_storage
            .get_certificate(cert_id)
            .await
            .ok()
            .flatten()
            .map(|(cert, _)| cert)
    }

    /// Transfer a certificate to a new owner.
    ///
    /// This is a permanent ownership change. The certificate cannot be
    /// transferred if it is currently loaned out.
    ///
    /// # Errors
    ///
    /// Returns error if certificate not found, caller is not owner, or transfer fails.
    pub async fn transfer_certificate(
        &self,
        cert_id: CertificateId,
        from: Did,
        to: Did,
    ) -> LoamSpineResult<EntryHash> {
        let (mut cert, spine_id) = self
            .certificate_storage
            .get_certificate(cert_id)
            .await?
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        if cert.owner != from {
            return Err(LoamSpineError::NotCertificateOwner);
        }

        if cert.is_loaned() {
            return Err(LoamSpineError::CertificateLoaned(cert_id));
        }

        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let entry = spine.create_entry(EntryType::CertificateTransfer {
            cert_id,
            from: from.clone(),
            to: to.clone(),
        });

        let entry_hash = spine.append(entry)?;
        let appended = spine
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;

        cert.owner = to;
        cert.transfer_count += 1;
        cert.current_location = CertificateLocation {
            spine: spine_id,
            entry: entry_hash,
            index: spine.height - 1,
        };
        cert.updated_at = Timestamp::now();

        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&spine).await?;
        self.certificate_storage
            .save_certificate(&cert, spine_id)
            .await?;

        Ok(entry_hash)
    }

    /// Verify a certificate's integrity.
    ///
    /// Checks that:
    /// - The certificate exists in storage
    /// - Its associated spine exists
    /// - The mint entry is present in storage
    /// - Chain integrity (owner matches spine records)
    ///
    /// # Errors
    ///
    /// Returns error if storage lookup fails.
    pub async fn verify_certificate(
        &self,
        cert_id: CertificateId,
    ) -> LoamSpineResult<CertificateVerification> {
        let Some((cert, spine_id)) = self.certificate_storage.get_certificate(cert_id).await?
        else {
            return Ok(CertificateVerification {
                cert_id,
                passed: Vec::new(),
            });
        };

        let mut passed = vec![VerificationCheck::Exists];

        let spine_exists = self.spine_storage.get_spine(spine_id).await?.is_some();
        if spine_exists {
            passed.push(VerificationCheck::SpineExists);
        }

        let mint_entry_exists = self
            .entry_storage
            .get_entry(cert.mint_info.entry)
            .await?
            .is_some();
        if mint_entry_exists {
            passed.push(VerificationCheck::MintEntryExists);
        }

        let location_entry_exists = self
            .entry_storage
            .get_entry(cert.current_location.entry)
            .await?
            .is_some();

        if spine_exists && mint_entry_exists && location_entry_exists {
            passed.push(VerificationCheck::ChainValid);
        }

        Ok(CertificateVerification { cert_id, passed })
    }

    /// Retrieve the lifecycle history of a certificate stored via the
    /// certificate storage trait.
    ///
    /// Returns all entries referencing this certificate ID (mint, transfer,
    /// loan, return) in spine order.  This method requires the certificate
    /// to have been created through `mint_certificate` (i.e. it exists in
    /// `certificate_storage`).  For a trait-based scan of all spines, use
    /// the `ProvenanceSource::get_certificate_history` trait method.
    ///
    /// # Errors
    ///
    /// Returns error if certificate or spine not found, or storage fails.
    pub async fn certificate_lifecycle(
        &self,
        cert_id: CertificateId,
    ) -> LoamSpineResult<Vec<crate::entry::Entry>> {
        let (_cert, spine_id) = self
            .certificate_storage
            .get_certificate(cert_id)
            .await?
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        let spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let entries = self
            .entry_storage
            .get_entries_for_spine(spine_id, 0, spine.height)
            .await?;

        let history: Vec<crate::entry::Entry> = entries
            .into_iter()
            .filter(|e| entry_references_certificate(&e.entry_type, cert_id))
            .collect();

        Ok(history)
    }

    /// Generate a cryptographic proof of a certificate's ownership chain.
    ///
    /// Collects all ownership-establishing entries (mint and transfers),
    /// builds a Merkle tree over their hashes, and returns a
    /// `CertificateOwnershipProof`.
    ///
    /// # Errors
    ///
    /// Returns error if certificate not found or storage fails.
    pub async fn generate_provenance_proof(
        &self,
        cert_id: CertificateId,
    ) -> LoamSpineResult<CertificateOwnershipProof> {
        let (_cert, _spine_id) = self
            .certificate_storage
            .get_certificate(cert_id)
            .await?
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        let lifecycle = self.certificate_lifecycle(cert_id).await?;

        let mut entries: Vec<EntryHash> = Vec::new();

        for entry in &lifecycle {
            let hash = entry.compute_hash()?;
            match &entry.entry_type {
                EntryType::CertificateMint { .. } | EntryType::CertificateTransfer { .. } => {
                    entries.push(hash);
                }
                _ => {}
            }
        }

        let chain_root = crate::proof::compute_merkle_root(&entries);
        let chain_length = entries.len() as u64;

        Ok(CertificateOwnershipProof {
            certificate_id: cert_id,
            chain_root,
            chain_length,
            entries,
            created_at: Timestamp::now(),
        })
    }

    /// List all certificates.
    ///
    /// # Errors
    ///
    /// Returns error if storage fails.
    pub async fn list_certificates(&self) -> LoamSpineResult<Vec<Certificate>> {
        let ids = self.certificate_storage.list_certificates().await?;
        let mut result = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some((cert, _)) = self.certificate_storage.get_certificate(id).await? {
                result.push(cert);
            }
        }
        Ok(result)
    }

    /// Get certificate count.
    pub async fn certificate_count(&self) -> usize {
        self.certificate_storage.certificate_count().await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
#[path = "certificate_tests.rs"]
mod tests;
