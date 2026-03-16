// SPDX-License-Identifier: AGPL-3.0-or-later

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

    let result = manager.transfer(cert.id, &owner, &buyer);
    assert!(result.is_ok());

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

    let result = manager.transfer(cert.id, &attacker, &buyer);
    assert!(matches!(result, Err(LoamSpineError::NotCertificateOwner)));
}

#[test]
fn loan_and_return() {
    let mut manager = create_test_manager();
    let owner = Did::new("did:key:z6MkOwner");
    let borrower = Did::new("did:key:z6MkBorrower");

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

    let terms = LoanTerms::new()
        .with_duration(crate::SECONDS_PER_DAY)
        .with_auto_return(true);
    let result = manager.loan(cert.id, &owner, &borrower, terms);
    assert!(result.is_ok());

    let loaned = manager
        .get_certificate(&cert.id)
        .unwrap_or_else(|| unreachable!());
    assert!(loaned.is_loaned());
    assert_eq!(loaned.holder, Some(borrower.clone()));

    let result = manager.return_loan(cert.id, &borrower);
    assert!(result.is_ok());

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

    let terms = LoanTerms::new();
    manager
        .loan(cert.id, &owner, &borrower, terms)
        .unwrap_or_else(|_| unreachable!());

    let result = manager.transfer(cert.id, &owner, &buyer);
    assert!(matches!(result, Err(LoamSpineError::CertificateLoaned(_))));
}

#[test]
fn list_certificates() {
    let mut manager = create_test_manager();
    let owner = Did::new("did:key:z6MkOwner");

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

    let spine = manager.spine();
    assert_eq!(spine.height, 1);

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

    let terms = LoanTerms::new();
    manager
        .loan(cert.id, &owner, &borrower, terms)
        .unwrap_or_else(|_| unreachable!());

    let result = manager.return_loan(cert.id, &attacker);
    assert!(matches!(result, Err(LoamSpineError::LoanTermsViolation(_))));
}

#[test]
fn return_non_loaned_certificate() {
    let mut manager = create_test_manager();
    let owner = Did::new("did:key:z6MkOwner");

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

    let result = manager.return_loan(cert.id, &owner);
    assert!(matches!(result, Err(LoamSpineError::LoanTermsViolation(_))));
}

#[test]
fn loan_not_owner() {
    let mut manager = create_test_manager();
    let owner = Did::new("did:key:z6MkOwner");
    let attacker = Did::new("did:key:z6MkAttacker");
    let borrower = Did::new("did:key:z6MkBorrower");

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

    let terms = LoanTerms::new();
    manager
        .loan(cert.id, &owner, &borrower1, terms.clone())
        .unwrap_or_else(|_| unreachable!());

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

    let terms = LoanTerms::new()
        .with_duration(crate::SECONDS_PER_YEAR)
        .with_auto_return(true);
    manager
        .loan(cert.id, &owner, &borrower, terms)
        .unwrap_or_else(|_| unreachable!());

    let count = manager.process_expired_loans();
    assert_eq!(count, 0);
}

#[test]
fn process_expired_loans_no_auto_return() {
    let mut manager = create_test_manager();
    let owner = Did::new("did:key:z6MkOwner");
    let borrower = Did::new("did:key:z6MkBorrower");

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

    let terms = LoanTerms::new().with_duration(0).with_auto_return(false);
    manager
        .loan(cert.id, &owner, &borrower, terms)
        .unwrap_or_else(|_| unreachable!());

    let count = manager.process_expired_loans();
    assert_eq!(count, 0);
}

#[test]
fn process_expired_loans_no_duration() {
    let mut manager = create_test_manager();
    let owner = Did::new("did:key:z6MkOwner");
    let borrower = Did::new("did:key:z6MkBorrower");

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

    let terms = LoanTerms::new().with_auto_return(true);
    manager
        .loan(cert.id, &owner, &borrower, terms)
        .unwrap_or_else(|_| unreachable!());

    let count = manager.process_expired_loans();
    assert_eq!(count, 0);
}
