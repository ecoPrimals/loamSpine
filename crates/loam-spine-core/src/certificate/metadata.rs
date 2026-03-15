// SPDX-License-Identifier: AGPL-3.0-only

//! Certificate metadata and constants.
//!
//! Defines metadata structures and scyborg license constants used by
//! Loam Certificates.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::PayloadRef;

// ============================================================================
// Scyborg License Constants
// ============================================================================

/// Type URI for scyborg license certificates.
pub const SCYBORG_LICENSE_TYPE_URI: &str = "scyborg:license";

/// Current schema version for scyborg license certificates.
pub const SCYBORG_LICENSE_SCHEMA_VERSION: u32 = 1;

/// Metadata key for SPDX license expression.
pub const SCYBORG_META_SPDX: &str = "scyborg:spdx_expression";

/// Metadata key for content category (code, creative, mechanics).
pub const SCYBORG_META_CATEGORY: &str = "scyborg:content_category";

/// Metadata key for copyright holder.
pub const SCYBORG_META_COPYRIGHT: &str = "scyborg:copyright_holder";

/// Metadata key for share-alike requirement.
pub const SCYBORG_META_SHARE_ALIKE: &str = "scyborg:share_alike";

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

/// Certificate metadata.
///
/// Stores display information and custom attributes for a certificate.
/// Supports scyborg license metadata for tri-license compliance.
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

    /// Populate metadata for a scyborg license certificate.
    ///
    /// Sets the canonical scyborg metadata fields used by sweetGrass (attribution)
    /// and rhizoCrypt (derivation chains) to enforce tri-license compliance.
    #[must_use]
    pub fn with_scyborg_license(
        self,
        spdx_expression: impl Into<String>,
        content_category: impl Into<String>,
        copyright_holder: impl Into<String>,
        share_alike: bool,
    ) -> Self {
        self.with_attribute(SCYBORG_META_SPDX, spdx_expression)
            .with_attribute(SCYBORG_META_CATEGORY, content_category)
            .with_attribute(SCYBORG_META_COPYRIGHT, copyright_holder)
            .with_attribute(SCYBORG_META_SHARE_ALIKE, share_alike.to_string())
    }

    /// Check if this metadata contains scyborg license fields.
    #[must_use]
    pub fn is_scyborg_license(&self) -> bool {
        self.attributes.contains_key(SCYBORG_META_SPDX)
    }

    /// Get the SPDX expression from scyborg metadata.
    #[must_use]
    pub fn scyborg_spdx(&self) -> Option<&str> {
        self.attributes.get(SCYBORG_META_SPDX).map(String::as_str)
    }

    /// Get the content category from scyborg metadata.
    #[must_use]
    pub fn scyborg_category(&self) -> Option<&str> {
        self.attributes
            .get(SCYBORG_META_CATEGORY)
            .map(String::as_str)
    }
}
