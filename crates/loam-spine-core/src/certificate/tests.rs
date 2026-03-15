// SPDX-License-Identifier: AGPL-3.0-only

//! Certificate module tests.

use super::*;
use crate::types::{CertificateId, Did, SpineId, Timestamp};

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

#[test]
fn scyborg_license_certificate_type() {
    let cert_type = CertificateType::scyborg_license();
    assert!(cert_type.is_scyborg_license());
    assert_eq!(cert_type.category(), "custom");
}

#[test]
fn scyborg_license_metadata() {
    let metadata = CertificateMetadata::new()
        .with_name("ecoPrimals Code License")
        .with_scyborg_license("AGPL-3.0-or-later", "code", "ecoPrimals Contributors", true);

    assert!(metadata.is_scyborg_license());
    assert_eq!(metadata.scyborg_spdx(), Some("AGPL-3.0-or-later"));
    assert_eq!(metadata.scyborg_category(), Some("code"));
    assert_eq!(
        metadata.attributes.get(SCYBORG_META_COPYRIGHT),
        Some(&"ecoPrimals Contributors".to_string())
    );
    assert_eq!(
        metadata.attributes.get(SCYBORG_META_SHARE_ALIKE),
        Some(&"true".to_string())
    );
}

#[test]
fn scyborg_license_full_certificate() {
    let owner = Did::new("did:key:z6MkLicensor");
    let mint_info = MintInfo {
        minter: owner.clone(),
        spine: SpineId::now_v7(),
        entry: [1u8; 32],
        timestamp: Timestamp::now(),
        authority: None,
    };

    let cert = Certificate::new(
        CertificateId::now_v7(),
        CertificateType::scyborg_license(),
        &owner,
        &mint_info,
    )
    .with_metadata(
        CertificateMetadata::new()
            .with_name("LoamSpine AGPL License")
            .with_description("Immutable proof that AGPL-3.0-or-later applies")
            .with_scyborg_license("AGPL-3.0-or-later", "code", "ecoPrimals Contributors", true),
    );

    assert!(cert.cert_type.is_scyborg_license());
    assert!(cert.is_active());
    assert!(cert.metadata.is_scyborg_license());
    assert_eq!(cert.metadata.scyborg_spdx(), Some("AGPL-3.0-or-later"));
}

#[test]
fn scyborg_creative_commons_license() {
    let metadata = CertificateMetadata::new().with_scyborg_license(
        "CC-BY-SA-4.0",
        "creative",
        "ecoPrimals Authors",
        true,
    );

    assert!(metadata.is_scyborg_license());
    assert_eq!(metadata.scyborg_spdx(), Some("CC-BY-SA-4.0"));
    assert_eq!(metadata.scyborg_category(), Some("creative"));
}

#[test]
fn non_scyborg_certificate_is_not_scyborg() {
    let cert_type = CertificateType::Custom {
        type_uri: "other:custom".to_string(),
        schema_version: 1,
    };
    assert!(!cert_type.is_scyborg_license());

    let metadata = CertificateMetadata::new().with_name("Not a license");
    assert!(!metadata.is_scyborg_license());
}
