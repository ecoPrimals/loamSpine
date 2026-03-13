// SPDX-License-Identifier: AGPL-3.0-only

//! Certificate and Spine management traits.
//!
//! These traits define the operations for managing spines and certificates.

use std::collections::HashMap;

use crate::certificate::{
    Certificate, CertificateLocation, CertificateMetadata, CertificateState, CertificateType,
    LoanInfo, LoanTerms, MintInfo,
};
use crate::entry::EntryType;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::spine::Spine;
use crate::types::{CertificateId, Did, EntryHash, Timestamp};

/// Certificate manager for a spine.
///
/// Provides operations for minting, transferring, loaning, and returning certificates.
pub struct CertificateManager {
    /// The underlying spine.
    spine: Spine,
    /// Certificate registry (id -> certificate).
    certificates: HashMap<CertificateId, Certificate>,
}

impl CertificateManager {
    /// Create a new certificate manager for a spine.
    #[must_use]
    pub fn new(spine: Spine) -> Self {
        Self {
            spine,
            certificates: HashMap::new(),
        }
    }

    /// Get the underlying spine.
    #[must_use]
    pub const fn spine(&self) -> &Spine {
        &self.spine
    }

    /// Get a mutable reference to the underlying spine.
    pub const fn spine_mut(&mut self) -> &mut Spine {
        &mut self.spine
    }

    /// Get a certificate by ID.
    #[must_use]
    pub fn get_certificate(&self, id: &CertificateId) -> Option<&Certificate> {
        self.certificates.get(id)
    }

    /// List all certificates.
    #[must_use]
    pub fn list_certificates(&self) -> Vec<&Certificate> {
        self.certificates.values().collect()
    }

    /// Mint a new certificate.
    ///
    /// # Errors
    ///
    /// Returns an error if the spine is sealed or the certificate already exists.
    pub fn mint(
        &mut self,
        cert_type: CertificateType,
        initial_owner: &Did,
        metadata: CertificateMetadata,
    ) -> LoamSpineResult<(Certificate, EntryHash)> {
        // Generate certificate ID
        let cert_id = CertificateId::now_v7();

        // Check if certificate already exists
        if self.certificates.contains_key(&cert_id) {
            return Err(LoamSpineError::CertificateExists(cert_id));
        }

        // Create mint entry
        let entry = self.spine.create_entry(EntryType::CertificateMint {
            cert_id,
            cert_type: cert_type.category().to_string(),
            initial_owner: initial_owner.clone(),
        });

        // Append to spine
        let entry_hash = self.spine.append(entry)?;

        // Create mint info
        let mint_info = MintInfo::new(initial_owner.clone(), self.spine.id, entry_hash);

        // Create certificate
        let cert =
            Certificate::new(cert_id, cert_type, initial_owner, &mint_info).with_metadata(metadata);

        // Register certificate
        self.certificates.insert(cert_id, cert.clone());

        Ok((cert, entry_hash))
    }

    /// Transfer a certificate to a new owner.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The certificate doesn't exist
    /// - The caller is not the owner
    /// - The certificate is loaned
    pub fn transfer(
        &mut self,
        cert_id: CertificateId,
        caller: &Did,
        to: &Did,
    ) -> LoamSpineResult<EntryHash> {
        // Get certificate
        let cert = self
            .certificates
            .get(&cert_id)
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        // Check ownership
        if &cert.owner != caller {
            return Err(LoamSpineError::NotCertificateOwner);
        }

        // Check not loaned
        if cert.is_loaned() {
            return Err(LoamSpineError::CertificateLoaned(cert_id));
        }

        // Create transfer entry
        let entry = self.spine.create_entry(EntryType::CertificateTransfer {
            cert_id,
            from: caller.clone(),
            to: to.clone(),
        });

        // Append to spine
        let entry_hash = self.spine.append(entry)?;

        // Update certificate
        if let Some(cert) = self.certificates.get_mut(&cert_id) {
            cert.owner = to.clone();
            cert.transfer_count += 1;
            cert.updated_at = Timestamp::now();
            cert.current_location = CertificateLocation {
                spine: self.spine.id,
                entry: entry_hash,
                index: self.spine.height - 1,
            };
        }

        Ok(entry_hash)
    }

    /// Loan a certificate to a borrower.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The certificate doesn't exist
    /// - The caller is not the owner
    /// - The certificate is already loaned
    pub fn loan(
        &mut self,
        cert_id: CertificateId,
        caller: &Did,
        borrower: &Did,
        terms: LoanTerms,
    ) -> LoamSpineResult<EntryHash> {
        // Get certificate
        let cert = self
            .certificates
            .get(&cert_id)
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        // Check ownership
        if &cert.owner != caller {
            return Err(LoamSpineError::NotCertificateOwner);
        }

        // Check not already loaned
        if cert.is_loaned() {
            return Err(LoamSpineError::CertificateLoaned(cert_id));
        }

        // Create loan entry
        let entry = self.spine.create_entry(EntryType::CertificateLoan {
            cert_id,
            lender: caller.clone(),
            borrower: borrower.clone(),
            duration_secs: terms.duration_secs,
            auto_return: terms.auto_return,
        });

        // Append to spine
        let entry_hash = self.spine.append(entry)?;

        // Calculate expiration
        let expires_at = terms.duration_secs.map(|secs| {
            let now = Timestamp::now();
            Timestamp::from_nanos(now.as_nanos() + secs * 1_000_000_000)
        });

        // Update certificate
        if let Some(cert) = self.certificates.get_mut(&cert_id) {
            cert.holder = Some(borrower.clone());
            cert.state = CertificateState::Loaned {
                loan_entry: entry_hash,
            };
            cert.active_loan = Some(LoanInfo {
                loan_entry: entry_hash,
                borrower: borrower.clone(),
                terms,
                started_at: Timestamp::now(),
                expires_at,
                waypoint: None,
                waypoint_anchor: None,
            });
            cert.updated_at = Timestamp::now();
            cert.current_location = CertificateLocation {
                spine: self.spine.id,
                entry: entry_hash,
                index: self.spine.height - 1,
            };
        }

        Ok(entry_hash)
    }

    /// Return a loaned certificate.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The certificate doesn't exist
    /// - The certificate is not loaned
    /// - The caller is not the borrower
    pub fn return_loan(
        &mut self,
        cert_id: CertificateId,
        caller: &Did,
    ) -> LoamSpineResult<EntryHash> {
        // Get certificate
        let cert = self
            .certificates
            .get(&cert_id)
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        // Check is loaned
        let loan_entry = match &cert.state {
            CertificateState::Loaned { loan_entry } => *loan_entry,
            _ => {
                return Err(LoamSpineError::LoanTermsViolation(
                    "certificate is not loaned".into(),
                ))
            }
        };

        // Check caller is borrower
        if cert.holder.as_ref() != Some(caller) {
            return Err(LoamSpineError::LoanTermsViolation(
                "caller is not the borrower".into(),
            ));
        }

        // Create return entry
        let entry = self.spine.create_entry(EntryType::CertificateReturn {
            cert_id,
            loan_entry,
        });

        // Append to spine
        let entry_hash = self.spine.append(entry)?;

        // Update certificate
        if let Some(cert) = self.certificates.get_mut(&cert_id) {
            cert.holder = None;
            cert.state = CertificateState::Active;
            cert.active_loan = None;
            cert.updated_at = Timestamp::now();
            cert.current_location = CertificateLocation {
                spine: self.spine.id,
                entry: entry_hash,
                index: self.spine.height - 1,
            };
        }

        Ok(entry_hash)
    }

    /// Check for expired loans and auto-return them.
    ///
    /// Returns the number of certificates auto-returned.
    pub fn process_expired_loans(&mut self) -> usize {
        let now = Timestamp::now();
        let mut expired: Vec<(CertificateId, Did)> = Vec::new();

        // Find expired loans
        for cert in self.certificates.values() {
            if let Some(loan) = &cert.active_loan {
                if loan.terms.auto_return {
                    if let Some(expires_at) = loan.expires_at {
                        if now.as_nanos() > expires_at.as_nanos() {
                            expired.push((cert.id, loan.borrower.clone()));
                        }
                    }
                }
            }
        }

        // Auto-return expired loans
        let mut count = 0;
        for (cert_id, borrower) in expired {
            if self.return_loan(cert_id, &borrower).is_ok() {
                count += 1;
            }
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::SpineConfig;

    fn create_test_manager() -> CertificateManager {
        let owner = Did::new("did:key:z6MkOwner");
        let spine = Spine::new(owner, Some("Test".into()), SpineConfig::default())
            .unwrap_or_else(|_| unreachable!());
        CertificateManager::new(spine)
    }

    #[test]
    fn mint_certificate() {
        let mut manager = create_test_manager();
        let owner = Did::new("did:key:z6MkOwner");

        let (cert, _hash) = manager
            .mint(
                CertificateType::DigitalGame {
                    platform: "steam".into(),
                    game_id: "hl3".into(),
                    edition: None,
                },
                &owner,
                CertificateMetadata::new().with_name("Half-Life 3"),
            )
            .unwrap_or_else(|_| unreachable!());

        assert_eq!(cert.owner, owner);
        assert!(cert.is_active());
        assert_eq!(manager.spine().height, 2); // genesis + mint
        assert!(manager.get_certificate(&cert.id).is_some());
    }

    #[test]
    fn transfer_certificate() {
        let mut manager = create_test_manager();
        let owner = Did::new("did:key:z6MkOwner");
        let buyer = Did::new("did:key:z6MkBuyer");

        // Mint
        let (cert, _) = manager
            .mint(
                CertificateType::DigitalGame {
                    platform: "steam".into(),
                    game_id: "hl3".into(),
                    edition: None,
                },
                &owner,
                CertificateMetadata::new(),
            )
            .unwrap_or_else(|_| unreachable!());

        // Transfer
        let result = manager.transfer(cert.id, &owner, &buyer);
        assert!(result.is_ok());

        // Verify new owner
        let updated = manager
            .get_certificate(&cert.id)
            .unwrap_or_else(|| unreachable!());
        assert_eq!(updated.owner, buyer);
        assert_eq!(updated.transfer_count, 1);
    }

    #[test]
    fn transfer_not_owner() {
        let mut manager = create_test_manager();
        let owner = Did::new("did:key:z6MkOwner");
        let attacker = Did::new("did:key:z6MkAttacker");
        let buyer = Did::new("did:key:z6MkBuyer");

        // Mint
        let (cert, _) = manager
            .mint(
                CertificateType::DigitalGame {
                    platform: "steam".into(),
                    game_id: "hl3".into(),
                    edition: None,
                },
                &owner,
                CertificateMetadata::new(),
            )
            .unwrap_or_else(|_| unreachable!());

        // Try to transfer as non-owner
        let result = manager.transfer(cert.id, &attacker, &buyer);
        assert!(matches!(result, Err(LoamSpineError::NotCertificateOwner)));
    }

    #[test]
    fn loan_and_return() {
        let mut manager = create_test_manager();
        let owner = Did::new("did:key:z6MkOwner");
        let borrower = Did::new("did:key:z6MkBorrower");

        // Mint
        let (cert, _) = manager
            .mint(
                CertificateType::DigitalGame {
                    platform: "steam".into(),
                    game_id: "hl3".into(),
                    edition: None,
                },
                &owner,
                CertificateMetadata::new(),
            )
            .unwrap_or_else(|_| unreachable!());

        // Loan
        let terms = LoanTerms::new()
            .with_duration(crate::SECONDS_PER_DAY)
            .with_auto_return(true);
        let result = manager.loan(cert.id, &owner, &borrower, terms);
        assert!(result.is_ok());

        // Verify loaned state
        let loaned = manager
            .get_certificate(&cert.id)
            .unwrap_or_else(|| unreachable!());
        assert!(loaned.is_loaned());
        assert_eq!(loaned.holder, Some(borrower.clone()));

        // Return
        let result = manager.return_loan(cert.id, &borrower);
        assert!(result.is_ok());

        // Verify returned
        let returned = manager
            .get_certificate(&cert.id)
            .unwrap_or_else(|| unreachable!());
        assert!(returned.is_active());
        assert!(returned.holder.is_none());
    }

    #[test]
    fn cannot_transfer_loaned() {
        let mut manager = create_test_manager();
        let owner = Did::new("did:key:z6MkOwner");
        let borrower = Did::new("did:key:z6MkBorrower");
        let buyer = Did::new("did:key:z6MkBuyer");

        // Mint
        let (cert, _) = manager
            .mint(
                CertificateType::DigitalGame {
                    platform: "steam".into(),
                    game_id: "hl3".into(),
                    edition: None,
                },
                &owner,
                CertificateMetadata::new(),
            )
            .unwrap_or_else(|_| unreachable!());

        // Loan
        let terms = LoanTerms::new();
        manager
            .loan(cert.id, &owner, &borrower, terms)
            .unwrap_or_else(|_| unreachable!());

        // Try to transfer while loaned
        let result = manager.transfer(cert.id, &owner, &buyer);
        assert!(matches!(result, Err(LoamSpineError::CertificateLoaned(_))));
    }

    #[test]
    fn list_certificates() {
        let mut manager = create_test_manager();
        let owner = Did::new("did:key:z6MkOwner");

        // Mint several certificates
        for i in 0..3 {
            manager
                .mint(
                    CertificateType::DigitalGame {
                        platform: "steam".into(),
                        game_id: format!("game-{i}"),
                        edition: None,
                    },
                    &owner,
                    CertificateMetadata::new(),
                )
                .unwrap_or_else(|_| unreachable!());
        }

        assert_eq!(manager.list_certificates().len(), 3);
    }

    #[test]
    fn spine_accessors() {
        let mut manager = create_test_manager();

        // Test spine()
        let spine = manager.spine();
        assert_eq!(spine.height, 1); // Genesis only

        // Test spine_mut()
        let spine_mut = manager.spine_mut();
        assert_eq!(spine_mut.height, 1);
    }

    #[test]
    fn get_nonexistent_certificate() {
        let manager = create_test_manager();
        let fake_id = CertificateId::now_v7();

        assert!(manager.get_certificate(&fake_id).is_none());
    }

    #[test]
    fn return_by_non_borrower() {
        let mut manager = create_test_manager();
        let owner = Did::new("did:key:z6MkOwner");
        let borrower = Did::new("did:key:z6MkBorrower");
        let attacker = Did::new("did:key:z6MkAttacker");

        // Mint
        let (cert, _) = manager
            .mint(
                CertificateType::DigitalGame {
                    platform: "steam".into(),
                    game_id: "hl3".into(),
                    edition: None,
                },
                &owner,
                CertificateMetadata::new(),
            )
            .unwrap_or_else(|_| unreachable!());

        // Loan
        let terms = LoanTerms::new();
        manager
            .loan(cert.id, &owner, &borrower, terms)
            .unwrap_or_else(|_| unreachable!());

        // Try to return as non-borrower
        let result = manager.return_loan(cert.id, &attacker);
        assert!(matches!(result, Err(LoamSpineError::LoanTermsViolation(_))));
    }

    #[test]
    fn return_non_loaned_certificate() {
        let mut manager = create_test_manager();
        let owner = Did::new("did:key:z6MkOwner");

        // Mint
        let (cert, _) = manager
            .mint(
                CertificateType::DigitalGame {
                    platform: "steam".into(),
                    game_id: "hl3".into(),
                    edition: None,
                },
                &owner,
                CertificateMetadata::new(),
            )
            .unwrap_or_else(|_| unreachable!());

        // Try to return non-loaned certificate
        let result = manager.return_loan(cert.id, &owner);
        assert!(matches!(result, Err(LoamSpineError::LoanTermsViolation(_))));
    }

    #[test]
    fn loan_not_owner() {
        let mut manager = create_test_manager();
        let owner = Did::new("did:key:z6MkOwner");
        let attacker = Did::new("did:key:z6MkAttacker");
        let borrower = Did::new("did:key:z6MkBorrower");

        // Mint
        let (cert, _) = manager
            .mint(
                CertificateType::DigitalGame {
                    platform: "steam".into(),
                    game_id: "hl3".into(),
                    edition: None,
                },
                &owner,
                CertificateMetadata::new(),
            )
            .unwrap_or_else(|_| unreachable!());

        // Try to loan as non-owner
        let terms = LoanTerms::new();
        let result = manager.loan(cert.id, &attacker, &borrower, terms);
        assert!(matches!(result, Err(LoamSpineError::NotCertificateOwner)));
    }

    #[test]
    fn loan_already_loaned() {
        let mut manager = create_test_manager();
        let owner = Did::new("did:key:z6MkOwner");
        let borrower1 = Did::new("did:key:z6MkBorrower1");
        let borrower2 = Did::new("did:key:z6MkBorrower2");

        // Mint
        let (cert, _) = manager
            .mint(
                CertificateType::DigitalGame {
                    platform: "steam".into(),
                    game_id: "hl3".into(),
                    edition: None,
                },
                &owner,
                CertificateMetadata::new(),
            )
            .unwrap_or_else(|_| unreachable!());

        // Loan to borrower1
        let terms = LoanTerms::new();
        manager
            .loan(cert.id, &owner, &borrower1, terms.clone())
            .unwrap_or_else(|_| unreachable!());

        // Try to loan again while already loaned
        let result = manager.loan(cert.id, &owner, &borrower2, terms);
        assert!(matches!(result, Err(LoamSpineError::CertificateLoaned(_))));
    }

    #[test]
    fn transfer_nonexistent_certificate() {
        let mut manager = create_test_manager();
        let owner = Did::new("did:key:z6MkOwner");
        let buyer = Did::new("did:key:z6MkBuyer");
        let fake_id = CertificateId::now_v7();

        let result = manager.transfer(fake_id, &owner, &buyer);
        assert!(matches!(
            result,
            Err(LoamSpineError::CertificateNotFound(_))
        ));
    }

    #[test]
    fn loan_nonexistent_certificate() {
        let mut manager = create_test_manager();
        let owner = Did::new("did:key:z6MkOwner");
        let borrower = Did::new("did:key:z6MkBorrower");
        let fake_id = CertificateId::now_v7();

        let terms = LoanTerms::new();
        let result = manager.loan(fake_id, &owner, &borrower, terms);
        assert!(matches!(
            result,
            Err(LoamSpineError::CertificateNotFound(_))
        ));
    }

    #[test]
    fn return_nonexistent_certificate() {
        let mut manager = create_test_manager();
        let borrower = Did::new("did:key:z6MkBorrower");
        let fake_id = CertificateId::now_v7();

        let result = manager.return_loan(fake_id, &borrower);
        assert!(matches!(
            result,
            Err(LoamSpineError::CertificateNotFound(_))
        ));
    }

    #[test]
    fn process_expired_loans_none() {
        let mut manager = create_test_manager();
        let owner = Did::new("did:key:z6MkOwner");
        let borrower = Did::new("did:key:z6MkBorrower");

        // Mint
        let (cert, _) = manager
            .mint(
                CertificateType::DigitalGame {
                    platform: "steam".into(),
                    game_id: "hl3".into(),
                    edition: None,
                },
                &owner,
                CertificateMetadata::new(),
            )
            .unwrap_or_else(|_| unreachable!());

        // Loan with long duration (not expired)
        let terms = LoanTerms::new()
            .with_duration(crate::SECONDS_PER_YEAR)
            .with_auto_return(true);
        manager
            .loan(cert.id, &owner, &borrower, terms)
            .unwrap_or_else(|_| unreachable!());

        // Process - should return 0 (none expired)
        let count = manager.process_expired_loans();
        assert_eq!(count, 0);
    }

    #[test]
    fn process_expired_loans_no_auto_return() {
        let mut manager = create_test_manager();
        let owner = Did::new("did:key:z6MkOwner");
        let borrower = Did::new("did:key:z6MkBorrower");

        // Mint
        let (cert, _) = manager
            .mint(
                CertificateType::DigitalGame {
                    platform: "steam".into(),
                    game_id: "hl3".into(),
                    edition: None,
                },
                &owner,
                CertificateMetadata::new(),
            )
            .unwrap_or_else(|_| unreachable!());

        // Loan without auto_return
        let terms = LoanTerms::new()
            .with_duration(0) // Already expired
            .with_auto_return(false); // But no auto-return
        manager
            .loan(cert.id, &owner, &borrower, terms)
            .unwrap_or_else(|_| unreachable!());

        // Process - should return 0 (no auto-return)
        let count = manager.process_expired_loans();
        assert_eq!(count, 0);
    }

    #[test]
    fn process_expired_loans_no_duration() {
        let mut manager = create_test_manager();
        let owner = Did::new("did:key:z6MkOwner");
        let borrower = Did::new("did:key:z6MkBorrower");

        // Mint
        let (cert, _) = manager
            .mint(
                CertificateType::DigitalGame {
                    platform: "steam".into(),
                    game_id: "hl3".into(),
                    edition: None,
                },
                &owner,
                CertificateMetadata::new(),
            )
            .unwrap_or_else(|_| unreachable!());

        // Loan without duration (indefinite)
        let terms = LoanTerms::new().with_auto_return(true); // auto-return but no duration
        manager
            .loan(cert.id, &owner, &borrower, terms)
            .unwrap_or_else(|_| unreachable!());

        // Process - should return 0 (no expiry)
        let count = manager.process_expired_loans();
        assert_eq!(count, 0);
    }
}
