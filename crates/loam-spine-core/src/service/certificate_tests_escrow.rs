// SPDX-License-Identifier: AGPL-3.0-or-later

//! Certificate escrow, loan expiry, and return edge-case tests.
//!
//! Split from `certificate_tests.rs` for navigability.
//! Tests here cover: hold/release/cancel escrow, loan expiry auto-return,
//! return error paths (not-loaned, wrong borrower, nonexistent), and
//! sublending chain unwinding on expiry.

use super::*;
use crate::certificate::{Certificate, CertificateType};

#[tokio::test]
async fn test_hold_and_release_certificate() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");
    let buyer = Did::new("did:key:z6MkBuyer");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert_type = CertificateType::DigitalGame {
        platform: "steam".into(),
        game_id: "test".into(),
        edition: None,
    };

    let (cert_id, _hash) = service
        .mint_certificate(spine_id, cert_type, owner.clone(), None)
        .await
        .unwrap_or_else(|_| unreachable!());

    let conditions = vec![crate::certificate::EscrowCondition::PaymentReceived {
        amount: 100,
        currency: "USD".into(),
    }];

    let escrow_id = service
        .hold_certificate(cert_id, buyer.clone(), conditions, None)
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert = service.get_certificate(cert_id).await;
    assert!(cert.is_some());
    assert!(matches!(
        cert.as_ref().map(|c| &c.state),
        Some(crate::certificate::CertificateState::PendingTransfer { .. })
    ));

    let released_id = service
        .release_certificate(escrow_id)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_eq!(released_id, cert_id);

    let cert = service.get_certificate(cert_id).await;
    assert!(cert.is_some());
    assert_eq!(cert.as_ref().map(|c| &c.owner), Some(&buyer));
    assert!(cert.as_ref().is_some_and(Certificate::is_active));
}

#[tokio::test]
async fn test_hold_and_cancel_escrow() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");
    let buyer = Did::new("did:key:z6MkBuyer");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert_type = CertificateType::DigitalGame {
        platform: "steam".into(),
        game_id: "test".into(),
        edition: None,
    };

    let (cert_id, _hash) = service
        .mint_certificate(spine_id, cert_type, owner.clone(), None)
        .await
        .unwrap_or_else(|_| unreachable!());

    let conditions = vec![crate::certificate::EscrowCondition::SignatureRequired {
        signer: buyer.clone(),
    }];

    let escrow_id = service
        .hold_certificate(cert_id, buyer.clone(), conditions, None)
        .await
        .unwrap_or_else(|_| unreachable!());

    let result = service.cancel_escrow(escrow_id).await;
    assert!(result.is_ok());

    let cert = service.get_certificate(cert_id).await;
    assert!(cert.is_some());
    assert_eq!(cert.as_ref().map(|c| &c.owner), Some(&owner));
    assert!(cert.as_ref().is_some_and(Certificate::is_active));
}

#[tokio::test]
async fn test_hold_certificate_fails_when_loaned() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");
    let borrower = Did::new("did:key:z6MkBorrower");
    let buyer = Did::new("did:key:z6MkBuyer");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert_type = CertificateType::DigitalGame {
        platform: "steam".into(),
        game_id: "test".into(),
        edition: None,
    };

    let (cert_id, _hash) = service
        .mint_certificate(spine_id, cert_type, owner.clone(), None)
        .await
        .unwrap_or_else(|_| unreachable!());

    let terms = crate::certificate::LoanTerms::new()
        .with_duration(crate::certificate::SECONDS_PER_DAY)
        .with_auto_return(false);
    service
        .loan_certificate(cert_id, owner.clone(), borrower.clone(), terms)
        .await
        .unwrap_or_else(|_| unreachable!());

    let conditions = vec![crate::certificate::EscrowCondition::Custom {
        description: "test".into(),
    }];

    let result = service
        .hold_certificate(cert_id, buyer.clone(), conditions, None)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_release_escrow_not_found() {
    let service = LoamSpineService::new();
    let fake_escrow_id = uuid::Uuid::now_v7();

    let result = service.release_certificate(fake_escrow_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cancel_escrow_not_found() {
    let service = LoamSpineService::new();
    let fake_escrow_id = uuid::Uuid::now_v7();

    let result = service.cancel_escrow(fake_escrow_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_escrow_conditions_variants() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");
    let buyer = Did::new("did:key:z6MkBuyer");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert_type = CertificateType::DigitalGame {
        platform: "steam".into(),
        game_id: "test".into(),
        edition: None,
    };

    let (cert_id, _hash) = service
        .mint_certificate(spine_id, cert_type, owner.clone(), None)
        .await
        .unwrap_or_else(|_| unreachable!());

    let conditions = vec![
        crate::certificate::EscrowCondition::PaymentReceived {
            amount: 50,
            currency: "ETH".into(),
        },
        crate::certificate::EscrowCondition::TimeElapsed {
            after: crate::types::Timestamp::now(),
        },
    ];

    let escrow_id = service
        .hold_certificate(cert_id, buyer.clone(), conditions, None)
        .await
        .unwrap_or_else(|_| unreachable!());

    let released = service.release_certificate(escrow_id).await;
    assert!(released.is_ok());
}

#[tokio::test]
async fn test_loan_expires_at_set() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");
    let borrower = Did::new("did:key:z6MkBorrower");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert_type = CertificateType::DigitalGame {
        platform: "steam".into(),
        game_id: "test".into(),
        edition: None,
    };

    let (cert_id, _hash) = service
        .mint_certificate(spine_id, cert_type, owner.clone(), None)
        .await
        .unwrap_or_else(|_| unreachable!());

    let terms = crate::certificate::LoanTerms::new()
        .with_duration(crate::certificate::SECONDS_PER_DAY)
        .with_auto_return(true);

    service
        .loan_certificate(cert_id, owner.clone(), borrower.clone(), terms)
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert = service.get_certificate(cert_id).await;
    assert!(cert.is_some());
    assert!(
        cert.as_ref()
            .and_then(|c| c.active_loan.as_ref())
            .and_then(|l| l.expires_at)
            .is_some()
    );
}

#[tokio::test]
async fn return_not_loaned_certificate_fails() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwnerReturn");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Return Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert_type = CertificateType::DigitalGame {
        platform: "test".into(),
        game_id: "return-test".into(),
        edition: None,
    };

    let (cert_id, _) = service
        .mint_certificate(spine_id, cert_type, owner.clone(), None)
        .await
        .unwrap_or_else(|_| unreachable!());

    let result = service.return_certificate(cert_id, owner).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn return_certificate_by_wrong_borrower_fails() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwnerWrongReturn");
    let borrower = Did::new("did:key:z6MkBorrowerWrongReturn");
    let wrong_returner = Did::new("did:key:z6MkWrongReturner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Wrong Return Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert_type = CertificateType::DigitalGame {
        platform: "test".into(),
        game_id: "wrong-return".into(),
        edition: None,
    };

    let (cert_id, _) = service
        .mint_certificate(spine_id, cert_type, owner.clone(), None)
        .await
        .unwrap_or_else(|_| unreachable!());

    let terms = crate::certificate::LoanTerms::default();
    service
        .loan_certificate(cert_id, owner, borrower, terms)
        .await
        .unwrap_or_else(|_| unreachable!());

    let result = service.return_certificate(cert_id, wrong_returner).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn transfer_nonexistent_certificate_fails() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwnerTransferFail");
    let recipient = Did::new("did:key:z6MkRecipient");

    let fake_cert_id = crate::types::CertificateId::now_v7();
    let result = service
        .transfer_certificate(fake_cert_id, owner, recipient)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn loan_nonexistent_certificate_fails() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwnerLoanFail");
    let borrower = Did::new("did:key:z6MkBorrowerLoanFail");

    let fake_cert_id = crate::types::CertificateId::now_v7();
    let terms = crate::certificate::LoanTerms::default();
    let result = service
        .loan_certificate(fake_cert_id, owner, borrower, terms)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn verify_nonexistent_certificate() {
    let service = LoamSpineService::new();

    let fake_cert_id = crate::types::CertificateId::now_v7();
    let result = service.verify_certificate(fake_cert_id).await;
    assert!(result.is_ok());
    let verification = result.unwrap_or_else(|_| unreachable!());
    assert!(!verification.is_valid());
}

#[tokio::test]
async fn test_return_certificate_expired_auto_return_disabled() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwnerAutoOff");
    let borrower = Did::new("did:key:z6MkBorrowerAutoOff");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("AutoOff".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert_type = CertificateType::DigitalGame {
        platform: "test".into(),
        game_id: "auto-off".into(),
        edition: None,
    };

    let (cert_id, _) = service
        .mint_certificate(spine_id, cert_type, owner.clone(), None)
        .await
        .unwrap_or_else(|_| unreachable!());

    let terms = crate::certificate::LoanTerms::new()
        .with_duration(crate::certificate::SECONDS_PER_DAY)
        .with_auto_return(false);
    service
        .loan_certificate(cert_id, owner, borrower, terms)
        .await
        .unwrap_or_else(|_| unreachable!());

    let result = service.return_certificate_expired(cert_id).await;
    let Err(e) = result else {
        unreachable!("expected auto_return error");
    };
    let msg = e.to_string();
    assert!(
        msg.contains("auto_return") || msg.contains("not enabled"),
        "expected auto_return error, got: {msg}"
    );
}

#[tokio::test]
async fn test_return_certificate_expired_no_expiry() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwnerNoExp");
    let borrower = Did::new("did:key:z6MkBorrowerNoExp");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("NoExp".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert_type = CertificateType::DigitalGame {
        platform: "test".into(),
        game_id: "no-expiry".into(),
        edition: None,
    };

    let (cert_id, _) = service
        .mint_certificate(spine_id, cert_type, owner.clone(), None)
        .await
        .unwrap_or_else(|_| unreachable!());

    let mut terms = crate::certificate::LoanTerms::new().with_auto_return(true);
    terms.duration_secs = None;
    service
        .loan_certificate(cert_id, owner, borrower, terms)
        .await
        .unwrap_or_else(|_| unreachable!());

    let result = service.return_certificate_expired(cert_id).await;
    let Err(e) = result else {
        unreachable!("expected no-expiry error");
    };
    let msg = e.to_string();
    assert!(
        msg.contains("no expiry"),
        "expected no-expiry error, got: {msg}"
    );
}

#[tokio::test]
async fn test_return_certificate_expired_success() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwnerExpOk");
    let borrower = Did::new("did:key:z6MkBorrowerExpOk");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("ExpOk".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert_type = CertificateType::DigitalGame {
        platform: "test".into(),
        game_id: "expire-ok".into(),
        edition: None,
    };

    let (cert_id, _) = service
        .mint_certificate(spine_id, cert_type, owner.clone(), None)
        .await
        .unwrap_or_else(|_| unreachable!());

    let terms = crate::certificate::LoanTerms::new()
        .with_duration(0)
        .with_auto_return(true);
    service
        .loan_certificate(cert_id, owner, borrower, terms)
        .await
        .unwrap_or_else(|_| unreachable!());

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let result = service.return_certificate_expired(cert_id).await;
    assert!(
        result.is_ok(),
        "expired loan should auto-return: {:?}",
        result.err()
    );

    let cert = service
        .get_certificate(cert_id)
        .await
        .unwrap_or_else(|| unreachable!("cert should exist"));
    assert!(
        !cert.is_loaned(),
        "cert should no longer be loaned after expiry return"
    );
}

#[tokio::test]
async fn test_return_certificate_expired_with_sublending_chain() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwnerChain");
    let borrower_a = Did::new("did:key:z6MkBorrowerA");
    let borrower_b = Did::new("did:key:z6MkBorrowerB");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Chain".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert_type = CertificateType::DigitalGame {
        platform: "test".into(),
        game_id: "chain-expire".into(),
        edition: None,
    };

    let (cert_id, _) = service
        .mint_certificate(spine_id, cert_type, owner.clone(), None)
        .await
        .unwrap_or_else(|_| unreachable!());

    let terms = crate::certificate::LoanTerms::new()
        .with_duration(0)
        .with_auto_return(true)
        .with_sublend(true, Some(3));

    service
        .loan_certificate(cert_id, owner, borrower_a.clone(), terms)
        .await
        .unwrap_or_else(|_| unreachable!());

    service
        .sublend_certificate(cert_id, borrower_a, borrower_b)
        .await
        .unwrap_or_else(|_| unreachable!());

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let result = service.return_certificate_expired(cert_id).await;
    assert!(
        result.is_ok(),
        "expired chain should fully unwind: {:?}",
        result.err()
    );

    let cert = service
        .get_certificate(cert_id)
        .await
        .unwrap_or_else(|| unreachable!("cert should exist"));
    assert!(
        !cert.is_loaned(),
        "cert should be fully returned after chain unwind"
    );
}

#[tokio::test]
async fn test_return_certificate_expired_nonexistent_cert() {
    let service = LoamSpineService::new();
    let fake_id = crate::types::CertificateId::now_v7();
    let result = service.return_certificate_expired(fake_id).await;
    assert!(result.is_err());
}
