//! Certificate types for LoamSpine.
//!
//! Certificates are memory-bound objects with verifiable ownership,
//! transferability, and complete provenance history. Unlike blockchain NFTs,
//! Loam Certificates are sovereign, lendable, and have complete history.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{CertificateId, Did, EntryHash, PayloadRef, SpineId, Timestamp};

// ============================================================================
// Time Constants
// ============================================================================

/// Seconds in a minute.
pub const SECONDS_PER_MINUTE: u64 = 60;

/// Seconds in an hour.
pub const SECONDS_PER_HOUR: u64 = 60 * SECONDS_PER_MINUTE;

/// Seconds in a day (24 hours).
pub const SECONDS_PER_DAY: u64 = 24 * SECONDS_PER_HOUR;

/// Seconds in a week (7 days).
pub const SECONDS_PER_WEEK: u64 = 7 * SECONDS_PER_DAY;

/// Seconds in a year (365 days, approximation).
pub const SECONDS_PER_YEAR: u64 = 365 * SECONDS_PER_DAY;

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

/// Certificate type taxonomy.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CertificateType {
    // === Digital Assets ===
    /// Digital game license.
    DigitalGame {
        /// Platform (steam, gog, epic, etc.).
        platform: String,
        /// Game identifier.
        game_id: String,
        /// Edition (standard, deluxe, etc.).
        edition: Option<String>,
    },

    /// In-game item.
    GameItem {
        /// Game identifier.
        game_id: String,
        /// Item type (weapon, armor, cosmetic, etc.).
        item_type: String,
        /// Unique item identifier.
        item_id: String,
        /// Item attributes.
        attributes: HashMap<String, String>,
    },

    /// Digital collectible.
    DigitalCollectible {
        /// Collection identifier.
        collection_id: String,
        /// Item number in collection.
        item_number: Option<u64>,
        /// Total supply (None = unlimited).
        total_supply: Option<u64>,
        /// Rarity tier.
        rarity: Option<Rarity>,
    },

    /// Software license.
    SoftwareLicense {
        /// Software identifier.
        software_id: String,
        /// License type (perpetual, subscription, etc.).
        license_type: String,
        /// Number of seats.
        seats: Option<u32>,
        /// Expiration timestamp.
        expires: Option<Timestamp>,
    },

    /// Digital media (book, music, video).
    DigitalMedia {
        /// Media type.
        media_type: MediaType,
        /// Content identifier.
        content_id: String,
        /// Format (epub, mp3, mp4, etc.).
        format: String,
    },

    // === Credentials ===
    /// Academic credential.
    AcademicCredential {
        /// Institution name.
        institution: String,
        /// Credential type (degree, certificate, etc.).
        credential_type: String,
        /// Field of study.
        field_of_study: String,
        /// Date awarded.
        date_awarded: Timestamp,
    },

    /// Professional license.
    ProfessionalLicense {
        /// Issuing authority.
        issuing_authority: String,
        /// License type.
        license_type: String,
        /// License number.
        license_number: String,
        /// Jurisdiction.
        jurisdiction: String,
        /// Expiration.
        expires: Option<Timestamp>,
    },

    // === Provenance ===
    /// Artwork provenance.
    ArtworkProvenance {
        /// Artist name.
        artist: String,
        /// Artwork title.
        title: String,
        /// Medium (oil, digital, etc.).
        medium: String,
        /// Year created.
        year_created: Option<u32>,
    },

    /// Data provenance.
    DataProvenance {
        /// Data type description.
        data_type: String,
        /// Source identifier.
        source_id: String,
        /// Collection timestamp.
        collected_at: Timestamp,
    },

    // === Custom ===
    /// Custom certificate type.
    Custom {
        /// Type URI for schema.
        type_uri: String,
        /// Schema version.
        schema_version: u32,
    },
}

impl CertificateType {
    /// Get the category of this certificate type.
    #[must_use]
    pub const fn category(&self) -> &'static str {
        match self {
            Self::DigitalGame { .. }
            | Self::GameItem { .. }
            | Self::DigitalCollectible { .. }
            | Self::SoftwareLicense { .. }
            | Self::DigitalMedia { .. } => "digital_asset",
            Self::AcademicCredential { .. } | Self::ProfessionalLicense { .. } => "credential",
            Self::ArtworkProvenance { .. } | Self::DataProvenance { .. } => "provenance",
            Self::Custom { .. } => "custom",
        }
    }

    /// Check if this certificate type can expire.
    #[must_use]
    pub const fn can_expire(&self) -> bool {
        matches!(
            self,
            Self::SoftwareLicense {
                expires: Some(_),
                ..
            } | Self::ProfessionalLicense {
                expires: Some(_),
                ..
            }
        )
    }
}

/// Rarity tier for collectibles.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rarity {
    /// Common item.
    Common,
    /// Uncommon item.
    Uncommon,
    /// Rare item.
    Rare,
    /// Epic item.
    Epic,
    /// Legendary item.
    Legendary,
    /// Unique/one-of-a-kind item.
    Unique,
}

/// Media type for digital media certificates.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    /// E-book.
    Book,
    /// Music track or album.
    Music,
    /// Video content.
    Video,
    /// Podcast episode.
    Podcast,
    /// Other media type.
    Other(String),
}

/// Minting information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MintInfo {
    /// Who minted the certificate.
    pub minter: Did,
    /// Spine where minted.
    pub spine: SpineId,
    /// Mint entry hash.
    pub entry: EntryHash,
    /// Mint timestamp.
    pub timestamp: Timestamp,
    /// Minting authority (if delegated).
    pub authority: Option<MintingAuthority>,
}

impl MintInfo {
    /// Create new mint info.
    #[must_use]
    pub fn new(minter: Did, spine: SpineId, entry: EntryHash) -> Self {
        Self {
            minter,
            spine,
            entry,
            timestamp: Timestamp::now(),
            authority: None,
        }
    }
}

/// Minting authority (for delegated minting).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MintingAuthority {
    /// Authority DID.
    pub authority: Did,
    /// Authorization entry.
    pub authorization_entry: EntryHash,
}

/// Current certificate location.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificateLocation {
    /// Current spine.
    pub spine: SpineId,
    /// Latest state entry.
    pub entry: EntryHash,
    /// Entry index in spine.
    pub index: u64,
}

/// Certificate state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateState {
    /// Active and owned.
    Active,

    /// Currently loaned out.
    Loaned {
        /// Loan entry hash.
        loan_entry: EntryHash,
    },

    /// Pending transfer (escrow).
    PendingTransfer {
        /// Transfer entry hash.
        transfer_entry: EntryHash,
        /// Recipient.
        to: Did,
    },

    /// Revoked (no longer valid).
    Revoked {
        /// Revoke entry hash.
        revoke_entry: EntryHash,
        /// Revocation reason.
        reason: RevocationReason,
    },

    /// Expired (time-limited certificate).
    Expired {
        /// Expiration timestamp.
        expired_at: Timestamp,
    },
}

/// Revocation reason.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevocationReason {
    /// Fraudulent certificate.
    Fraud,
    /// Terms violated.
    TermsViolation,
    /// Replaced by new certificate.
    Superseded {
        /// Replacement certificate.
        replacement: CertificateId,
    },
    /// Administrative revocation.
    Administrative {
        /// Reason text.
        reason: String,
    },
    /// Owner requested revocation.
    OwnerRequest,
}

/// Loan information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoanInfo {
    /// Loan entry hash.
    pub loan_entry: EntryHash,
    /// Borrower DID.
    pub borrower: Did,
    /// Loan terms.
    pub terms: LoanTerms,
    /// Start time.
    pub started_at: Timestamp,
    /// Expected end time.
    pub expires_at: Option<Timestamp>,
    /// Waypoint spine (if anchored).
    pub waypoint: Option<SpineId>,
    /// Waypoint anchor entry.
    pub waypoint_anchor: Option<EntryHash>,
}

/// Loan terms.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LoanTerms {
    /// Loan duration in seconds.
    pub duration_secs: Option<u64>,
    /// Grace period in seconds.
    pub grace_period_secs: Option<u64>,
    /// Automatic return on expiry.
    pub auto_return: bool,
    /// Allowed operations during loan.
    pub allowed_operations: Vec<String>,
    /// Forbidden operations during loan.
    pub forbidden_operations: Vec<String>,
    /// Allow sub-lending.
    pub allow_sublend: bool,
    /// Maximum sublend depth.
    pub max_sublend_depth: Option<u32>,
}

impl LoanTerms {
    /// Create default loan terms.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set duration.
    #[must_use]
    pub const fn with_duration(mut self, secs: u64) -> Self {
        self.duration_secs = Some(secs);
        self
    }

    /// Set auto-return.
    #[must_use]
    pub const fn with_auto_return(mut self, auto: bool) -> Self {
        self.auto_return = auto;
        self
    }

    /// Allow sublending.
    #[must_use]
    pub const fn with_sublend(mut self, allow: bool, max_depth: Option<u32>) -> Self {
        self.allow_sublend = allow;
        self.max_sublend_depth = max_depth;
        self
    }
}

/// Certificate metadata.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CertificateMetadata {
    /// Display name.
    pub name: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Image reference.
    pub image: Option<PayloadRef>,
    /// External URL.
    pub external_url: Option<String>,
    /// Custom attributes.
    pub attributes: HashMap<String, String>,
}

impl CertificateMetadata {
    /// Create empty metadata.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the display name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add an attribute.
    #[must_use]
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

/// Certificate ownership record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OwnershipRecord {
    /// Owner at this point.
    pub owner: Did,
    /// Entry that established ownership.
    pub entry: EntryHash,
    /// Spine where entry exists.
    pub spine: SpineId,
    /// Ownership start time.
    pub from: Timestamp,
    /// Ownership end time (None if current).
    pub until: Option<Timestamp>,
    /// How ownership was acquired.
    pub acquisition: AcquisitionType,
}

/// How ownership was acquired.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AcquisitionType {
    /// Original mint.
    Mint,
    /// Transfer from previous owner.
    Transfer {
        /// Previous owner.
        from: Did,
    },
    /// Inherited (original owner dissolved).
    Inherited {
        /// Previous owner.
        from: Did,
    },
    /// Returned from loan.
    LoanReturn {
        /// Borrower who returned.
        borrower: Did,
    },
}

/// Full certificate history.
#[derive(Clone, Debug)]
pub struct CertificateHistory {
    /// The certificate.
    pub certificate: Certificate,
    /// Ownership records.
    pub ownership_records: Vec<OwnershipRecord>,
    /// Loan records.
    pub loan_records: Vec<LoanRecord>,
}

/// Loan record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoanRecord {
    /// Loan entry hash.
    pub loan_entry: EntryHash,
    /// Lender.
    pub lender: Did,
    /// Borrower.
    pub borrower: Did,
    /// Loan terms.
    pub terms: LoanTerms,
    /// Start time.
    pub started_at: Timestamp,
    /// End time.
    pub ended_at: Option<Timestamp>,
    /// Return entry hash.
    pub return_entry: Option<EntryHash>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn certificate_creation() {
        let id = CertificateId::now_v7();
        let owner = Did::new("did:key:z6MkOwner");
        let spine_id = SpineId::now_v7();
        let mint_info = MintInfo::new(owner.clone(), spine_id, [0u8; 32]);

        let cert = Certificate::new(
            id,
            CertificateType::DigitalGame {
                platform: "steam".into(),
                game_id: "half-life-3".into(),
                edition: None,
            },
            &owner,
            &mint_info,
        );

        assert_eq!(cert.owner, owner);
        assert!(cert.is_active());
        assert!(!cert.is_loaned());
    }

    #[test]
    fn certificate_metadata() {
        let metadata = CertificateMetadata::new()
            .with_name("Half-Life 3")
            .with_description("The legendary sequel")
            .with_attribute("platform", "steam");

        assert_eq!(metadata.name, Some("Half-Life 3".to_string()));
        assert_eq!(
            metadata.attributes.get("platform"),
            Some(&"steam".to_string())
        );
    }

    #[test]
    fn loan_terms_builder() {
        let terms = LoanTerms::new()
            .with_duration(SECONDS_PER_DAY)
            .with_auto_return(true)
            .with_sublend(false, None);

        assert_eq!(terms.duration_secs, Some(SECONDS_PER_DAY));
        assert!(terms.auto_return);
        assert!(!terms.allow_sublend);
    }

    #[test]
    fn certificate_type_category() {
        assert_eq!(
            CertificateType::DigitalGame {
                platform: "steam".into(),
                game_id: "test".into(),
                edition: None,
            }
            .category(),
            "digital_asset"
        );

        assert_eq!(
            CertificateType::AcademicCredential {
                institution: "MIT".into(),
                credential_type: "degree".into(),
                field_of_study: "CS".into(),
                date_awarded: Timestamp::now(),
            }
            .category(),
            "credential"
        );
    }

    #[test]
    fn certificate_can_expire() {
        let non_expiring = CertificateType::DigitalGame {
            platform: "steam".into(),
            game_id: "test".into(),
            edition: None,
        };
        assert!(!non_expiring.can_expire());

        let expiring = CertificateType::SoftwareLicense {
            software_id: "test".into(),
            license_type: "subscription".into(),
            seats: Some(1),
            expires: Some(Timestamp::now()),
        };
        assert!(expiring.can_expire());
    }

    #[test]
    fn certificate_state_checks() {
        let active = CertificateState::Active;
        assert!(matches!(active, CertificateState::Active));

        let loaned = CertificateState::Loaned {
            loan_entry: [0u8; 32],
        };
        assert!(matches!(loaned, CertificateState::Loaned { .. }));
    }

    #[test]
    fn effective_holder() {
        let id = CertificateId::now_v7();
        let owner = Did::new("did:key:z6MkOwner");
        let spine_id = SpineId::now_v7();
        let mint_info = MintInfo::new(owner.clone(), spine_id, [0u8; 32]);

        let mut cert = Certificate::new(
            id,
            CertificateType::DigitalGame {
                platform: "steam".into(),
                game_id: "test".into(),
                edition: None,
            },
            &owner,
            &mint_info,
        );

        // Without loan, effective holder is owner
        assert_eq!(cert.effective_holder(), &owner);

        // With loan, effective holder is borrower
        let borrower = Did::new("did:key:z6MkBorrower");
        cert.holder = Some(borrower.clone());
        assert_eq!(cert.effective_holder(), &borrower);
    }
}
