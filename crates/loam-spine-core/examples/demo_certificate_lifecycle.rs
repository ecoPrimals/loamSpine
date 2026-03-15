// SPDX-License-Identifier: AGPL-3.0-only

//! # 🦴 Demo: Certificate Lifecycle
//!
//! Full ownership model: mint → transfer → loan → return.
//!
//! This demo shows:
//! - Minting certificates for digital ownership
//! - Transferring ownership permanently
//! - Loaning with terms (duration, auto-return)
//! - Returning from loan
//!
//! ## Run
//! ```bash
//! cargo run -p loam-spine-core --example demo_certificate_lifecycle
//! ```

// Examples allow patterns for demonstration purposes
use loam_spine_core::{
    Did, LoamSpineResult, SpineId,
    certificate::{CertificateType, LoanTerms, SECONDS_PER_DAY, SECONDS_PER_HOUR},
    service::LoamSpineService,
    types::CertificateId,
};

/// Print certificate state from service.
async fn print_certificate_state(service: &LoamSpineService, cert_id: CertificateId) {
    if let Some(cert) = service.get_certificate(cert_id).await {
        println!("  Owner: {}", cert.owner);
        println!("  State: {:?}", cert.state);
        if cert.holder.is_some() {
            println!("  Holder: {:?}", cert.holder);
        }
        if let Some(loan) = &cert.active_loan {
            println!(
                "  Duration: {} hours",
                loan.terms.duration_secs.unwrap_or(0) / SECONDS_PER_HOUR
            );
            println!("  Auto-return: {}", loan.terms.auto_return);
        }
        if cert.transfer_count > 0 {
            println!("  Transfer count: {}", cert.transfer_count);
        }
    }
}

/// Mint a new game certificate.
async fn mint_certificate(
    service: &LoamSpineService,
    spine_id: SpineId,
    publisher: &Did,
) -> LoamSpineResult<CertificateId> {
    println!("1. MINT - Publisher Creates Game Key");
    println!("-------------------------------------");
    let (cert_id, entry_hash) = service
        .mint_certificate(
            spine_id,
            CertificateType::DigitalGame {
                game_id: "eco-adventure-2025".to_string(),
                platform: "universal".to_string(),
                edition: Some("collector".to_string()),
            },
            publisher.clone(),
            None,
        )
        .await?;
    println!("✓ Certificate minted!");
    println!("  ID: {cert_id}");
    println!("  Entry: {entry_hash:?}");
    print_certificate_state(service, cert_id).await;
    println!();
    Ok(cert_id)
}

/// Transfer certificate between owners.
async fn transfer_certificate(
    service: &LoamSpineService,
    cert_id: CertificateId,
    from: &Did,
    to: &Did,
    step: u8,
    from_name: &str,
    to_name: &str,
) -> LoamSpineResult<()> {
    println!("{step}. TRANSFER - {from_name} Sells to {to_name}");
    println!("{}", "-".repeat(40));
    service
        .transfer_certificate(cert_id, from.clone(), to.clone())
        .await?;
    println!("✓ Transferred to {to_name}!");
    print_certificate_state(service, cert_id).await;
    println!();
    Ok(())
}

/// Loan certificate to another user.
async fn loan_certificate(
    service: &LoamSpineService,
    cert_id: CertificateId,
    owner: &Did,
    borrower: &Did,
) -> LoamSpineResult<()> {
    println!("4. LOAN - Player Lends to Friend (48h)");
    println!("--------------------------------------");
    let terms = LoanTerms::new()
        .with_duration(2 * SECONDS_PER_DAY)
        .with_auto_return(true);
    service
        .loan_certificate(cert_id, owner.clone(), borrower.clone(), terms)
        .await?;
    println!("✓ Loaned to friend!");
    print_certificate_state(service, cert_id).await;
    println!();
    Ok(())
}

/// Return certificate from loan.
async fn return_certificate(
    service: &LoamSpineService,
    cert_id: CertificateId,
    borrower: &Did,
) -> LoamSpineResult<()> {
    println!("5. RETURN - Friend Returns Early");
    println!("--------------------------------");
    service
        .return_certificate(cert_id, borrower.clone())
        .await?;
    println!("✓ Returned to owner!");
    print_certificate_state(service, cert_id).await;
    println!();
    Ok(())
}

/// Print final summary.
async fn print_summary(service: &LoamSpineService, cert_id: CertificateId) {
    println!("6. CERTIFICATE HISTORY");
    println!("----------------------");
    if let Some(cert) = service.get_certificate(cert_id).await {
        println!("Final certificate state:");
        println!("  ID: {}", cert.id);
        println!("  Owner: {}", cert.owner);
        println!("  Transfers: {}", cert.transfer_count);
        println!("  State: {:?}", cert.state);
    }
    println!();

    println!("🎉 Success!");
    println!("===========");
    println!("You've completed a full certificate lifecycle:");
    println!();
    println!("  Publisher → Retailer → Player → (Friend loan) → Player");
    println!();
    println!("Key concepts:");
    println!("  • Mint: Create new certificate");
    println!("  • Transfer: Permanent ownership change");
    println!("  • Loan: Temporary access with terms");
    println!("  • Return: End loan, restore owner access");
    println!();
    println!("Next: cargo run -p loam-spine-core --example demo_backup_restore");
}

#[tokio::main]
async fn main() -> LoamSpineResult<()> {
    println!("🦴 Demo: Certificate Lifecycle");
    println!("==============================\n");

    // Create service and users
    let service = LoamSpineService::new();

    let publisher = Did::new("did:key:z6MkPublisher");
    let retailer = Did::new("did:key:z6MkRetailer");
    let player = Did::new("did:key:z6MkPlayer");
    let friend = Did::new("did:key:z6MkFriend");

    println!("Users:");
    println!("  Publisher: {publisher}");
    println!("  Retailer:  {retailer}");
    println!("  Player:    {player}");
    println!("  Friend:    {friend}");
    println!();

    // Create spine for the publisher
    let spine_id = service
        .ensure_spine(publisher.clone(), Some("Game Certificates".into()))
        .await?;
    println!("Created spine: {spine_id}\n");

    // Execute lifecycle steps
    let cert_id = mint_certificate(&service, spine_id, &publisher).await?;
    transfer_certificate(
        &service,
        cert_id,
        &publisher,
        &retailer,
        2,
        "Publisher",
        "Retailer",
    )
    .await?;
    transfer_certificate(
        &service, cert_id, &retailer, &player, 3, "Retailer", "Player",
    )
    .await?;
    loan_certificate(&service, cert_id, &player, &friend).await?;
    return_certificate(&service, cert_id, &friend).await?;
    print_summary(&service, cert_id).await;

    Ok(())
}
