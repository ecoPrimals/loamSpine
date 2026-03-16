// SPDX-License-Identifier: AGPL-3.0-or-later

//! Certificate escrow operations.
//!
//! Manages conditional transfers through escrow: hold, release, and cancel.
//! Escrow places a certificate into `PendingTransfer` state until conditions
//! are met or the escrow is cancelled.

use crate::certificate::{
    CertificateLocation, CertificateState, EscrowCondition, EscrowId, TransferConditions,
};
use crate::entry::EntryType;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::storage::{CertificateStorage, EntryStorage, SpineStorage};
use crate::types::{CertificateId, Did, Timestamp};

use super::LoamSpineService;

impl LoamSpineService {
    /// Put a certificate in escrow (PendingTransfer state).
    ///
    /// The certificate must be Active. The current owner becomes the seller
    /// (`from`), and `to` is the intended buyer. Conditions must be met
    /// before `release_certificate` completes the transfer.
    ///
    /// # Errors
    ///
    /// Returns error if certificate not found, not owned by seller, or
    /// already loaned.
    pub async fn hold_certificate(
        &self,
        cert_id: CertificateId,
        to: Did,
        conditions: Vec<EscrowCondition>,
        expires_at: Option<Timestamp>,
    ) -> LoamSpineResult<EscrowId> {
        let (mut cert, spine_id) = self
            .certificate_storage
            .get_certificate(cert_id)
            .await?
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        let from = cert.owner.clone();

        if cert.is_loaned() {
            return Err(LoamSpineError::CertificateLoaned(cert_id));
        }

        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let entry = spine.create_entry(EntryType::CertificateTransfer {
            cert_id,
            from: from.clone(),
            to: to.clone(),
        });

        let entry_hash = spine.append(entry)?;
        let appended = spine
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;

        let escrow_id = uuid::Uuid::now_v7();
        let now = Timestamp::now();

        cert.state = CertificateState::PendingTransfer {
            transfer_entry: entry_hash,
            to: to.clone(),
        };
        cert.current_location = CertificateLocation {
            spine: spine_id,
            entry: entry_hash,
            index: spine.height - 1,
        };
        cert.updated_at = now;

        let transfer_conditions = TransferConditions {
            escrow_id,
            cert_id,
            from: from.clone(),
            to: to.clone(),
            conditions,
            expires_at,
            created_at: now,
        };

        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&spine).await?;
        self.certificate_storage
            .save_certificate(&cert, spine_id)
            .await?;

        self.escrows
            .write()
            .await
            .insert(escrow_id, transfer_conditions);

        tracing::info!(escrow_id = %escrow_id, cert_id = %cert_id, "certificate held in escrow");

        Ok(escrow_id)
    }

    /// Release a certificate from escrow, completing the transfer.
    ///
    /// # Errors
    ///
    /// Returns error if escrow not found.
    pub async fn release_certificate(&self, escrow_id: EscrowId) -> LoamSpineResult<CertificateId> {
        let conditions = self
            .escrows
            .write()
            .await
            .remove(&escrow_id)
            .ok_or(LoamSpineError::EscrowNotFound(escrow_id))?;

        let cert_id = conditions.cert_id;

        let (mut cert, spine_id) = self
            .certificate_storage
            .get_certificate(cert_id)
            .await?
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        let to = conditions.to.clone();

        match &cert.state {
            CertificateState::PendingTransfer { .. } => {}
            _ => {
                return Err(LoamSpineError::InvalidEntryType(
                    "certificate not in pending transfer".into(),
                ));
            }
        }

        cert.owner = to;
        cert.transfer_count += 1;
        cert.state = CertificateState::Active;
        cert.updated_at = Timestamp::now();

        self.certificate_storage
            .save_certificate(&cert, spine_id)
            .await?;

        tracing::info!(escrow_id = %escrow_id, cert_id = %cert_id, "certificate released from escrow");

        Ok(cert_id)
    }

    /// Cancel an escrow, returning the certificate to Active state.
    ///
    /// # Errors
    ///
    /// Returns error if escrow not found.
    pub async fn cancel_escrow(&self, escrow_id: EscrowId) -> LoamSpineResult<()> {
        let conditions = self
            .escrows
            .write()
            .await
            .remove(&escrow_id)
            .ok_or(LoamSpineError::EscrowNotFound(escrow_id))?;

        let cert_id = conditions.cert_id;

        let (mut cert, spine_id) = self
            .certificate_storage
            .get_certificate(cert_id)
            .await?
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        match &cert.state {
            CertificateState::PendingTransfer { .. } => {}
            _ => {
                return Err(LoamSpineError::InvalidEntryType(
                    "certificate not in pending transfer".into(),
                ));
            }
        }

        cert.state = CertificateState::Active;
        cert.updated_at = Timestamp::now();

        self.certificate_storage
            .save_certificate(&cert, spine_id)
            .await?;

        tracing::info!(escrow_id = %escrow_id, cert_id = %cert_id, "escrow cancelled");

        Ok(())
    }
}
