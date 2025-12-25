#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use loam_spine_core::certificate::{Certificate, CertificateType, LoanTerms, MintInfo};
use loam_spine_core::types::{CertificateId, Did, EntryHash, SpineId, Timestamp};

/// Arbitrary input for certificate fuzzing
#[derive(Debug, Arbitrary)]
struct CertificateInput {
    owner_suffix: String,
    cert_type_variant: u8,
    loan_duration_secs: Option<u64>,
}

fuzz_target!(|input: CertificateInput| {
    // Create owner DID
    let owner = Did::new(&format!("did:key:z6Mk{}", &input.owner_suffix));
    let cert_id = CertificateId::now_v7();
    let spine_id = SpineId::now_v7();
    let mint_entry: EntryHash = [0u8; 32];

    // Select certificate type based on input
    let cert_type = match input.cert_type_variant % 5 {
        0 => CertificateType::DigitalGame {
            platform: "test".into(),
            game_id: "game-001".into(),
        },
        1 => CertificateType::DigitalCollectible {
            collection: "collection".into(),
            item_id: "item-001".into(),
        },
        2 => CertificateType::DigitalLicense {
            software: "software".into(),
            license_type: "perpetual".into(),
        },
        3 => CertificateType::AcademicDegree {
            institution: "MIT".into(),
            degree: "PhD".into(),
        },
        _ => CertificateType::Custom {
            type_uri: "custom:test".into(),
        },
    };

    // Create certificate
    let mint_info = MintInfo {
        minter: owner.clone(),
        minted_at: Timestamp::now(),
        mint_entry,
        spine_id,
    };

    let cert = Certificate::new(cert_id, cert_type, owner, mint_info);

    // Verify certificate methods don't panic
    let _ = cert.is_loaned();
    let _ = cert.can_expire();
    let _ = cert.effective_holder();
    let _ = cert.category();

    // Test loan terms if provided
    if let Some(duration) = input.loan_duration_secs {
        let terms = LoanTerms::default().with_duration(duration);
        let _ = terms.is_expired();
    }

    // Verify serialization roundtrip
    if let Ok(serialized) = serde_json::to_string(&cert) {
        let _ = serde_json::from_str::<Certificate>(&serialized);
    }
});

