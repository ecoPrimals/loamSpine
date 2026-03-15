// SPDX-License-Identifier: AGPL-3.0-only

//! Certificate type taxonomy and related enums.
//!
//! Defines the classification of certificates (DigitalGame, SoftwareLicense,
//! etc.) and supporting types like Rarity and MediaType.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::Timestamp;

use super::metadata::{SCYBORG_LICENSE_SCHEMA_VERSION, SCYBORG_LICENSE_TYPE_URI};

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

    /// Create a scyborg license certificate type.
    ///
    /// The scyborg licensing model uses Loam Certificates to immutably record
    /// license terms. This encodes the tri-license structure:
    /// - **Code**: AGPL-3.0-or-later
    /// - **Creative**: CC-BY-SA-4.0
    /// - **Mechanics**: ORC (reserved material)
    ///
    /// License-specific metadata (SPDX, copyright holder, share-alike) is stored
    /// on `CertificateMetadata` via `with_scyborg_license()`.
    #[must_use]
    pub fn scyborg_license() -> Self {
        Self::Custom {
            type_uri: SCYBORG_LICENSE_TYPE_URI.to_string(),
            schema_version: SCYBORG_LICENSE_SCHEMA_VERSION,
        }
    }

    /// Check if this is a scyborg license certificate.
    #[must_use]
    pub fn is_scyborg_license(&self) -> bool {
        matches!(self, Self::Custom { type_uri, .. } if type_uri == SCYBORG_LICENSE_TYPE_URI)
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
