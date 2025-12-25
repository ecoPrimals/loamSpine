//! Certificate Lifecycle Example
//!
//! Demonstrates the full lifecycle of a Loam Certificate:
//! 1. Mint a new certificate
//! 2. Transfer ownership
//! 3. Loan to temporary holder
//! 4. Return from loan
//! 5. Final transfer

// Examples allow patterns for demonstration purposes
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::clone_on_copy)]

use loam_spine_core::{
    certificate::SECONDS_PER_YEAR,
    entry::{Entry, EntryType, SpineConfig},
    types::CertificateId,
    Did as DidType, LoamSpineResult, Spine,
};

// Allow long function for comprehensive demonstration example
#[allow(clippy::too_many_lines)]
fn main() -> LoamSpineResult<()> {
    println!("🦴 LoamSpine Certificate Lifecycle Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Create spine for certificate history
    let issuer_did = DidType::new("did:example:issuer");
    let config = SpineConfig::default();

    let mut spine = Spine::new(
        issuer_did.clone(),
        Some("Certificate Lifecycle Demo".into()),
        config,
    )?;

    println!("📋 Created certificate spine:");
    println!("   Spine ID: {}", spine.id);
    println!("   Owner: {}", spine.owner);
    println!();

    // Step 1: Mint Certificate
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎫 Step 1: Minting Certificate\n");

    let alice_did = DidType::new("did:example:alice");
    let cert_id = CertificateId::now_v7();

    let mint_entry = Entry::new(
        spine.height,
        Some(spine.tip),
        spine.owner.clone(),
        EntryType::CertificateMint {
            cert_id,
            cert_type: "AccessCredential".to_string(),
            initial_owner: alice_did.clone(),
        },
    );

    spine.append(mint_entry)?;

    println!("✅ Certificate minted:");
    println!("   Certificate ID: {}", cert_id);
    println!("   Type: AccessCredential");
    println!("   Initial Owner: {}", alice_did);
    println!("   Spine Height: {}", spine.height);
    println!();

    // Step 2: Transfer Certificate
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔄 Step 2: Transferring to Bob\n");

    let bob_did = DidType::new("did:example:bob");

    let transfer_entry = Entry::new(
        spine.height,
        Some(spine.tip),
        alice_did.clone(),
        EntryType::CertificateTransfer {
            cert_id,
            from: alice_did.clone(),
            to: bob_did.clone(),
        },
    );

    spine.append(transfer_entry)?;

    println!("✅ Certificate transferred:");
    println!("   From: {}", alice_did);
    println!("   To: {}", bob_did);
    println!("   Spine Height: {}", spine.height);
    println!();

    // Step 3: Loan Certificate
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🤝 Step 3: Loaning to Charlie (temporary)\n");

    let charlie_did = DidType::new("did:example:charlie");
    let loan_duration = SECONDS_PER_YEAR; // 1 year

    let loan_entry = Entry::new(
        spine.height,
        Some(spine.tip),
        bob_did.clone(),
        EntryType::CertificateLoan {
            cert_id,
            lender: bob_did.clone(),
            borrower: charlie_did.clone(),
            duration_secs: Some(loan_duration),
            auto_return: false,
        },
    );

    spine.append(loan_entry)?;

    println!("✅ Certificate loaned:");
    println!("   Owner: {} (unchanged)", bob_did);
    println!("   Temporary Holder: {}", charlie_did);
    println!("   Duration: {} seconds (1 year)", loan_duration);
    println!("   Spine Height: {}", spine.height);
    println!();

    // Step 4: Return from Loan
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("↩️  Step 4: Returning from Loan\n");

    // Get the hash of the loan entry to reference it
    let loan_entry_hash = spine.tip;

    let return_entry = Entry::new(
        spine.height,
        Some(spine.tip),
        charlie_did.clone(),
        EntryType::CertificateReturn {
            cert_id,
            loan_entry: loan_entry_hash,
        },
    );

    spine.append(return_entry)?;

    println!("✅ Certificate returned:");
    println!("   From: {}", charlie_did);
    println!("   Back to Owner: {}", bob_did);
    println!("   Spine Height: {}", spine.height);
    println!();

    // Step 5: Final Transfer
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎁 Step 5: Final Transfer to Dave\n");

    let dave_did = DidType::new("did:example:dave");

    let final_transfer_entry = Entry::new(
        spine.height,
        Some(spine.tip),
        bob_did.clone(),
        EntryType::CertificateTransfer {
            cert_id,
            from: bob_did.clone(),
            to: dave_did.clone(),
        },
    );

    spine.append(final_transfer_entry)?;

    println!("✅ Final transfer complete:");
    println!("   From: {}", bob_did);
    println!("   To: {}", dave_did);
    println!("   Spine Height: {}", spine.height);
    println!();

    // Summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Certificate Lifecycle Summary\n");
    println!("Certificate ID: {}", cert_id);
    println!("Total Operations: {}", spine.height);
    println!();
    println!("Ownership History:");
    println!("  1. Minted → Alice");
    println!("  2. Alice → Bob");
    println!("  3. Bob → Charlie (loan)");
    println!("  4. Charlie → Bob (return)");
    println!("  5. Bob → Dave");
    println!();
    println!("Current State:");
    println!("  Owner: {}", dave_did);
    println!("  Holder: {} (same as owner)", dave_did);
    println!("  Spine Height: {}", spine.height);
    println!("  Spine Tip: {:?}", spine.tip);
    println!();
    println!("✅ Certificate lifecycle complete!");
    println!();
    println!("💡 Key Features:");
    println!("   • Complete provenance history");
    println!("   • Tamper-proof ownership trail");
    println!("   • Loan/return capability");
    println!("   • All changes in spine");
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}
