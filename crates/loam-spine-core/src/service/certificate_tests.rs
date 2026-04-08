// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::certificate::CertificateType;

#[tokio::test]
async fn test_mint_certificate() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert_type = CertificateType::DigitalGame {
        platform: "steam".into(),
        game_id: "test".into(),
        edition: None,
    };

    let result = service
        .mint_certificate(spine_id, cert_type, owner.clone(), None)
        .await;
    assert!(result.is_ok());

    let (cert_id, _hash) = result.unwrap_or_else(|_| unreachable!());

    let cert = service.get_certificate(cert_id).await;
    assert!(cert.is_some());

    let certs = service
        .list_certificates()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(!certs.is_empty());

    assert!(service.certificate_count().await >= 1);
}

#[tokio::test]
async fn test_certificate_transfer() {
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

    let result = service
        .transfer_certificate(cert_id, owner.clone(), buyer.clone())
        .await;
    assert!(result.is_ok());

    let cert = service.get_certificate(cert_id).await;
    assert!(cert.is_some());
    assert_eq!(cert.unwrap_or_else(|| unreachable!()).owner, buyer);
}

#[tokio::test]
async fn test_certificate_loan_and_return() {
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
        .with_auto_return(false);
    let result = service
        .loan_certificate(cert_id, owner.clone(), borrower.clone(), terms)
        .await;
    assert!(result.is_ok());

    let cert = service.get_certificate(cert_id).await;
    assert!(cert.is_some());
    assert!(cert.unwrap_or_else(|| unreachable!()).is_loaned());

    let result = service.return_certificate(cert_id, borrower.clone()).await;
    assert!(result.is_ok());

    let cert = service.get_certificate(cert_id).await;
    assert!(cert.is_some());
    assert!(!cert.unwrap_or_else(|| unreachable!()).is_loaned());
}

#[tokio::test]
async fn test_verify_certificate() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

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

    let verification = service
        .verify_certificate(cert_id)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(verification.exists());
    assert!(verification.is_valid());
}

#[tokio::test]
async fn test_verify_nonexistent_certificate() {
    let service = LoamSpineService::new();
    let fake_id = uuid::Uuid::now_v7();

    let verification = service
        .verify_certificate(fake_id)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(!verification.exists());
    assert!(!verification.is_valid());
}

#[tokio::test]
async fn test_get_certificate_history() {
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

    service
        .transfer_certificate(cert_id, owner.clone(), buyer.clone())
        .await
        .unwrap_or_else(|_| unreachable!());

    let history = service
        .certificate_lifecycle(cert_id)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_eq!(history.len(), 2);
}

#[tokio::test]
async fn test_get_certificate_history_with_loan() {
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
        .with_auto_return(false);
    service
        .loan_certificate(cert_id, owner.clone(), borrower.clone(), terms)
        .await
        .unwrap_or_else(|_| unreachable!());

    service
        .return_certificate(cert_id, borrower.clone())
        .await
        .unwrap_or_else(|_| unreachable!());

    let history = service
        .certificate_lifecycle(cert_id)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_eq!(history.len(), 3);
}

#[tokio::test]
async fn test_sublend_certificate() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");
    let borrower_a = Did::new("did:key:z6MkA");
    let borrower_b = Did::new("did:key:z6MkB");

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
        .with_auto_return(false)
        .with_sublend(true, Some(2));

    service
        .loan_certificate(cert_id, owner.clone(), borrower_a.clone(), terms)
        .await
        .unwrap_or_else(|_| unreachable!());

    let result = service
        .sublend_certificate(cert_id, borrower_a, borrower_b.clone())
        .await;
    assert!(result.is_ok());

    let cert = service.get_certificate(cert_id).await;
    assert!(cert.is_some());
    assert_eq!(
        cert.as_ref().map(|c| c.holder.as_ref()),
        Some(Some(&borrower_b))
    );
    assert!(
        cert.as_ref()
            .is_some_and(|c| c.active_loan.as_ref().is_some_and(|l| l
                .relending_chain
                .as_ref()
                .is_some_and(|ch| ch.links.len() == 2)))
    );
}

#[tokio::test]
async fn test_sublend_forbidden_by_terms() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");
    let borrower_a = Did::new("did:key:z6MkA");
    let borrower_b = Did::new("did:key:z6MkB");

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
        .with_sublend(false, None);

    service
        .loan_certificate(cert_id, owner.clone(), borrower_a.clone(), terms)
        .await
        .unwrap_or_else(|_| unreachable!());

    let result = service
        .sublend_certificate(cert_id, borrower_a, borrower_b)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_return_certificate_at_unwind() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");
    let borrower_a = Did::new("did:key:z6MkA");
    let borrower_b = Did::new("did:key:z6MkB");

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

    let terms = crate::certificate::LoanTerms::new().with_sublend(true, Some(2));

    service
        .loan_certificate(cert_id, owner.clone(), borrower_a.clone(), terms)
        .await
        .unwrap_or_else(|_| unreachable!());

    service
        .sublend_certificate(cert_id, borrower_a.clone(), borrower_b.clone())
        .await
        .unwrap_or_else(|_| unreachable!());

    let result = service.return_certificate_at(cert_id, borrower_b).await;
    assert!(result.is_ok());

    let cert = service.get_certificate(cert_id).await;
    assert!(cert.is_some());
    assert_eq!(
        cert.as_ref().map(|c| c.holder.as_ref()),
        Some(Some(&borrower_a))
    );
}

#[tokio::test]
async fn test_return_certificate_expired_fails_when_not_expired() {
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

    let result = service.return_certificate_expired(cert_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_generate_provenance_proof_mint_only() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

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

    let proof = service
        .generate_provenance_proof(cert_id)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_eq!(proof.certificate_id, cert_id);
    assert_eq!(proof.chain_length, 1);
    assert_eq!(proof.entries.len(), 1);
    assert!(proof.verify().unwrap_or_else(|_| unreachable!()));
}

#[tokio::test]
async fn test_generate_provenance_proof_with_transfers() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");
    let buyer_a = Did::new("did:key:z6MkBuyerA");
    let buyer_b = Did::new("did:key:z6MkBuyerB");

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

    service
        .transfer_certificate(cert_id, owner.clone(), buyer_a.clone())
        .await
        .unwrap_or_else(|_| unreachable!());

    service
        .transfer_certificate(cert_id, buyer_a.clone(), buyer_b.clone())
        .await
        .unwrap_or_else(|_| unreachable!());

    let proof = service
        .generate_provenance_proof(cert_id)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_eq!(proof.certificate_id, cert_id);
    assert_eq!(proof.chain_length, 3);
    assert_eq!(proof.entries.len(), 3);
    assert!(proof.verify().unwrap_or_else(|_| unreachable!()));
}

#[tokio::test]
async fn test_generate_provenance_proof_excludes_loans() {
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
        .with_auto_return(false);
    service
        .loan_certificate(cert_id, owner.clone(), borrower.clone(), terms)
        .await
        .unwrap_or_else(|_| unreachable!());

    service
        .return_certificate(cert_id, borrower.clone())
        .await
        .unwrap_or_else(|_| unreachable!());

    let proof = service
        .generate_provenance_proof(cert_id)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_eq!(proof.chain_length, 1);
    assert!(proof.verify().unwrap_or_else(|_| unreachable!()));
}

#[tokio::test]
async fn test_generate_provenance_proof_not_found() {
    let service = LoamSpineService::new();
    let fake_id = uuid::Uuid::now_v7();

    let result = service.generate_provenance_proof(fake_id).await;
    assert!(result.is_err());
}
