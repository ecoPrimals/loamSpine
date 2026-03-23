// SPDX-License-Identifier: AGPL-3.0-or-later

//! Certificate types for LoamSpine.
//!
//! Certificates are memory-bound objects with verifiable ownership,
//! transferability, and complete provenance history. Unlike blockchain NFTs,
//! Loam Certificates are sovereign, lendable, and have complete history.

mod escrow;
mod lifecycle;
mod metadata;
mod provenance;
mod types;
mod usage;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "proptests use expect for concise assertions"
)]
mod proptests {
    use proptest::prelude::*;

    use super::*;
    use crate::types::{CertificateId, Did, SpineId, Timestamp};

    fn arb_did() -> impl Strategy<Value = Did> {
        "[a-z]{4,12}".prop_map(|s| Did::new(format!("did:key:z6Mk{s}")))
    }

    fn arb_entry_hash() -> impl Strategy<Value = [u8; 32]> {
        proptest::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&v);
            arr
        })
    }

    fn arb_mint_info() -> impl Strategy<Value = MintInfo> {
        (arb_did(), arb_entry_hash())
            .prop_map(|(minter, entry)| MintInfo::new(minter, SpineId::now_v7(), entry))
    }

    fn arb_cert_type() -> impl Strategy<Value = CertificateType> {
        prop_oneof![
            ("[a-z]{3,8}", "[a-z0-9]{3,12}").prop_map(|(platform, game_id)| {
                CertificateType::DigitalGame {
                    platform,
                    game_id,
                    edition: None,
                }
            }),
            ("[a-z]{3,8}", "[a-z]{3,8}").prop_map(|(software_id, license_type)| {
                CertificateType::SoftwareLicense {
                    software_id,
                    license_type,
                    seats: Some(1),
                    expires: None,
                }
            }),
            Just(CertificateType::scyborg_license()),
        ]
    }

    proptest! {
        #[test]
        fn new_certificate_is_always_active(
            owner in arb_did(),
            mint_info in arb_mint_info(),
            cert_type in arb_cert_type(),
        ) {
            let cert = Certificate::new(
                CertificateId::now_v7(),
                cert_type,
                &owner,
                &mint_info,
            );
            prop_assert!(cert.is_active());
            prop_assert!(!cert.is_loaned());
            prop_assert!(!cert.is_revoked());
            prop_assert_eq!(cert.transfer_count, 0);
        }

        #[test]
        fn effective_holder_is_owner_when_no_loan(
            owner in arb_did(),
            mint_info in arb_mint_info(),
        ) {
            let cert = Certificate::new(
                CertificateId::now_v7(),
                CertificateType::scyborg_license(),
                &owner,
                &mint_info,
            );
            prop_assert_eq!(cert.effective_holder(), &owner);
        }

        #[test]
        fn effective_holder_is_borrower_when_loaned(
            owner in arb_did(),
            borrower in arb_did(),
            mint_info in arb_mint_info(),
        ) {
            let mut cert = Certificate::new(
                CertificateId::now_v7(),
                CertificateType::scyborg_license(),
                &owner,
                &mint_info,
            );
            cert.holder = Some(borrower.clone());
            prop_assert_eq!(cert.effective_holder(), &borrower);
        }

        #[test]
        fn loaned_state_is_loaned(
            entry_hash in arb_entry_hash(),
        ) {
            let state = CertificateState::Loaned { loan_entry: entry_hash };
            let is_loaned = matches!(state, CertificateState::Loaned { .. });
            prop_assert!(is_loaned);
        }

        #[test]
        fn revoked_state_is_revoked(
            entry_hash in arb_entry_hash(),
        ) {
            let state = CertificateState::Revoked {
                revoke_entry: entry_hash,
                reason: RevocationReason::OwnerRequest,
            };
            let is_revoked = matches!(state, CertificateState::Revoked { .. });
            prop_assert!(is_revoked);
        }

        #[test]
        fn scyborg_license_roundtrip(
            owner in arb_did(),
            mint_info in arb_mint_info(),
        ) {
            let cert = Certificate::new(
                CertificateId::now_v7(),
                CertificateType::scyborg_license(),
                &owner,
                &mint_info,
            ).with_metadata(
                CertificateMetadata::new()
                    .with_name("Test License")
                    .with_scyborg_license("AGPL-3.0-or-later", "code", "Test", true),
            );

            prop_assert!(cert.cert_type.is_scyborg_license());
            prop_assert!(cert.metadata.is_scyborg_license());
            prop_assert_eq!(cert.metadata.scyborg_spdx(), Some("AGPL-3.0-or-later"));

            let json = serde_json::to_string(&cert).expect("serialize");
            let back: Certificate = serde_json::from_str(&json).expect("deserialize");
            prop_assert_eq!(back.id, cert.id);
            prop_assert_eq!(&back.owner, &cert.owner);
            prop_assert!(back.cert_type.is_scyborg_license());
        }

        #[test]
        fn certificate_timestamps_are_non_zero(
            owner in arb_did(),
            mint_info in arb_mint_info(),
        ) {
            let cert = Certificate::new(
                CertificateId::now_v7(),
                CertificateType::scyborg_license(),
                &owner,
                &mint_info,
            );
            prop_assert!(cert.created_at > Timestamp::from_nanos(0));
            prop_assert!(cert.updated_at >= cert.created_at);
        }

        #[test]
        fn category_is_never_empty(
            cert_type in arb_cert_type(),
        ) {
            prop_assert!(!cert_type.category().is_empty());
        }

        #[test]
        fn loan_terms_builder_preserves_values(
            duration_secs in 1u64..=86400 * 365,
            auto_return in any::<bool>(),
            allow_sublend in any::<bool>(),
        ) {
            let terms = LoanTerms::new()
                .with_duration(duration_secs)
                .with_auto_return(auto_return)
                .with_sublend(allow_sublend, None);
            prop_assert_eq!(terms.duration_secs, Some(duration_secs));
            prop_assert_eq!(terms.auto_return, auto_return);
            prop_assert_eq!(terms.allow_sublend, allow_sublend);
        }
    }
}

pub use escrow::{EscrowCondition, EscrowId, TransferConditions};
pub use lifecycle::{CertificateState, LoanInfo, LoanTerms, RevocationReason};
pub use metadata::{
    CertificateMetadata, SCYBORG_LICENSE_SCHEMA_VERSION, SCYBORG_LICENSE_TYPE_URI,
    SCYBORG_META_CATEGORY, SCYBORG_META_COPYRIGHT, SCYBORG_META_SHARE_ALIKE, SCYBORG_META_SPDX,
    SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE, SECONDS_PER_WEEK, SECONDS_PER_YEAR,
};
pub use provenance::{
    AcquisitionType, CertificateLocation, LoanRecord, MintInfo, MintingAuthority, OwnershipRecord,
};
pub use types::{CertificateType, MediaType, Rarity};
pub use usage::UsageSummary;

use serde::{Deserialize, Serialize};

use crate::types::{CertificateId, Did, Timestamp};

/// A Loam Certificate (memory-bound object).
///
/// Certificates represent digital ownership with full provenance tracking.
/// They can be minted, transferred, loaned, and returned.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Certificate {
    /// Unique certificate ID.
    pub id: CertificateId,

    /// Certificate type.
    pub cert_type: CertificateType,

    /// Certificate version (for schema evolution).
    pub version: u32,

    /// Current owner.
    pub owner: Did,

    /// Current holder (if loaned, different from owner).
    pub holder: Option<Did>,

    /// Minting information.
    pub mint_info: MintInfo,

    /// Current location.
    pub current_location: CertificateLocation,

    /// Certificate state.
    pub state: CertificateState,

    /// Transfer count.
    pub transfer_count: u64,

    /// Active loan (if any).
    pub active_loan: Option<LoanInfo>,

    /// Certificate metadata.
    pub metadata: CertificateMetadata,

    /// Creation timestamp.
    pub created_at: Timestamp,

    /// Last update timestamp.
    pub updated_at: Timestamp,
}

impl Certificate {
    /// Create a new certificate.
    #[must_use]
    pub fn new(
        id: CertificateId,
        cert_type: CertificateType,
        owner: &Did,
        mint_info: &MintInfo,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id,
            cert_type,
            version: 1,
            owner: owner.clone(),
            holder: None,
            mint_info: mint_info.clone(),
            current_location: CertificateLocation {
                spine: mint_info.spine,
                entry: mint_info.entry,
                index: 0,
            },
            state: CertificateState::Active,
            transfer_count: 0,
            active_loan: None,
            metadata: CertificateMetadata::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if the certificate is active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self.state, CertificateState::Active)
    }

    /// Check if the certificate is loaned.
    #[must_use]
    pub const fn is_loaned(&self) -> bool {
        matches!(self.state, CertificateState::Loaned { .. })
    }

    /// Check if the certificate is revoked.
    #[must_use]
    pub const fn is_revoked(&self) -> bool {
        matches!(self.state, CertificateState::Revoked { .. })
    }

    /// Get the effective holder (holder if loaned, owner otherwise).
    #[must_use]
    pub fn effective_holder(&self) -> &Did {
        self.holder.as_ref().unwrap_or(&self.owner)
    }

    /// Set metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: CertificateMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Full certificate history.
///
/// Contains the certificate and its complete ownership and loan records.
#[derive(Clone, Debug)]
pub struct CertificateHistory {
    /// The certificate.
    pub certificate: Certificate,
    /// Ownership records.
    pub ownership_records: Vec<OwnershipRecord>,
    /// Loan records.
    pub loan_records: Vec<LoanRecord>,
}
