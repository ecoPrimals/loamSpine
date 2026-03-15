// SPDX-License-Identifier: AGPL-3.0-only

//! Certificate escrow and transfer conditions.
//!
//! Escrow allows a certificate transfer to be put "on hold" until conditions
//! are met (e.g. payment received, signature obtained).

use serde::{Deserialize, Serialize};

use crate::types::{CertificateId, Did, Timestamp};

/// Escrow identifier (UUID).
pub type EscrowId = uuid::Uuid;

/// Conditions for releasing a certificate from escrow.
///
/// A transfer can be held until one or more conditions are satisfied.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransferConditions {
    /// Unique escrow identifier.
    pub escrow_id: EscrowId,
    /// Certificate in escrow.
    pub cert_id: CertificateId,
    /// Seller (current owner).
    pub from: Did,
    /// Buyer (intended recipient).
    pub to: Did,
    /// Conditions that must be met for release.
    pub conditions: Vec<EscrowCondition>,
    /// Optional expiration time.
    pub expires_at: Option<Timestamp>,
    /// When the escrow was created.
    pub created_at: Timestamp,
}

/// A single condition that must be met to release escrow.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EscrowCondition {
    /// Payment must be received.
    PaymentReceived {
        /// Amount required.
        amount: u64,
        /// Currency (e.g. "USD", "ETH").
        currency: String,
    },
    /// Signature required from a specific party.
    SignatureRequired {
        /// DID of the required signer.
        signer: Did,
    },
    /// Time must have elapsed.
    TimeElapsed {
        /// Release only after this timestamp.
        after: Timestamp,
    },
    /// Custom condition (human-readable description).
    Custom {
        /// Description of the condition.
        description: String,
    },
}
