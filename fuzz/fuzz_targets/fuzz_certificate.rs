// SPDX-License-Identifier: AGPL-3.0-or-later

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use loam_spine_core::certificate::{Certificate, CertificateType, MintInfo};
use loam_spine_core::types::{CertificateId, Did, EntryHash, SpineId, Timestamp};

#[derive(Debug, Arbitrary)]
struct CertificateInput {
    owner_suffix: String,
    cert_type_variant: u8,
}

fuzz_target!(|input: CertificateInput| {
    let owner = Did::new(&format!("did:key:z6Mk{}", &input.owner_suffix));
    let cert_id = CertificateId::now_v7();
    let spine_id = SpineId::now_v7();
    let mint_entry: EntryHash = [0u8; 32];

    let cert_type = match input.cert_type_variant % 4 {
        0 => CertificateType::DigitalGame {
            platform: "test".into(),
            game_id: "game-001".into(),
            edition: None,
        },
        1 => CertificateType::DigitalCollectible {
            collection_id: "collection".into(),
            item_number: Some(1),
            total_supply: None,
            rarity: None,
        },
        2 => CertificateType::SoftwareLicense {
            software_id: "software".into(),
            license_type: "perpetual".into(),
            seats: None,
            expires: None,
        },
        _ => CertificateType::Custom {
            type_uri: "custom:test".into(),
            schema_version: 1,
        },
    };

    let mint_info = MintInfo {
        minter: owner.clone(),
        spine: spine_id,
        entry: mint_entry,
        timestamp: Timestamp::now(),
        authority: None,
    };

    let cert = Certificate::new(cert_id, cert_type, &owner, &mint_info);

    let _ = cert.is_loaned();
    let _ = cert.is_active();
    let _ = cert.effective_holder();
    let _ = cert.cert_type.category();

    if let Ok(serialized) = serde_json::to_string(&cert) {
        let _ = serde_json::from_str::<Certificate>(&serialized);
    }
});
