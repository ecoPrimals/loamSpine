// SPDX-License-Identifier: AGPL-3.0-only

//! Expiry sweeper for auto-returning expired certificate loans.
//!
//! Runs periodically, checks all active loans for expiration, and
//! auto-returns them when `LoanTerms::auto_return` is true.

use std::time::Duration;

use tokio::sync::watch;
use tokio::time::interval;

use crate::error::LoamSpineResult;

use super::LoamSpineService;

/// Default sweep interval (60 seconds).
pub const DEFAULT_SWEEP_INTERVAL_SECS: u64 = 60;

/// Configuration for the expiry sweeper.
#[derive(Clone, Debug)]
pub struct ExpirySweeperConfig {
    /// Sweep interval in seconds.
    pub sweep_interval_secs: u64,
}

impl Default for ExpirySweeperConfig {
    fn default() -> Self {
        Self {
            sweep_interval_secs: DEFAULT_SWEEP_INTERVAL_SECS,
        }
    }
}

impl ExpirySweeperConfig {
    /// Create config with custom sweep interval.
    #[must_use]
    pub const fn with_interval(mut self, secs: u64) -> Self {
        self.sweep_interval_secs = secs;
        self
    }
}

/// Background task that periodically sweeps for expired loans and auto-returns them.
///
/// Spawn with `ExpirySweeper::spawn`. Use the returned handle to shut down.
pub struct ExpirySweeper {
    service: LoamSpineService,
    config: ExpirySweeperConfig,
}

impl ExpirySweeper {
    /// Create a new expiry sweeper.
    #[must_use]
    pub fn new(service: LoamSpineService, config: ExpirySweeperConfig) -> Self {
        Self { service, config }
    }

    /// Spawn the sweeper as a background task.
    ///
    /// Returns a handle that can be used to stop the sweeper.
    #[must_use]
    pub fn spawn(self) -> ExpirySweeperHandle {
        let (tx, rx) = watch::channel(());
        let handle = ExpirySweeperHandle(tx);

        tokio::spawn(async move {
            self.run(rx).await;
        });

        handle
    }

    /// Run one sweep cycle.
    ///
    /// Checks all certificates for expired loans and auto-returns them.
    ///
    /// # Errors
    ///
    /// Returns error if listing certificates or returning an expired loan fails.
    pub async fn sweep_once(&self) -> LoamSpineResult<usize> {
        let certs = self.service.list_certificates().await?;
        let mut returned = 0;

        for cert in certs {
            let Some(loan) = cert.active_loan.as_ref() else {
                continue;
            };

            if !loan.terms.auto_return {
                continue;
            }

            let Some(expires_at) = loan.expires_at else {
                continue;
            };

            if crate::types::Timestamp::now() <= expires_at {
                continue;
            }

            match self.service.return_certificate_expired(cert.id).await {
                Ok(_) => {
                    tracing::info!(
                        cert_id = %cert.id,
                        borrower = %loan.borrower,
                        "auto-returned expired loan"
                    );
                    returned += 1;
                }
                Err(e) => {
                    tracing::warn!(
                        cert_id = %cert.id,
                        error = %e,
                        "failed to auto-return expired loan"
                    );
                }
            }
        }

        Ok(returned)
    }

    async fn run(self, mut shutdown: watch::Receiver<()>) {
        let mut ticker = interval(Duration::from_secs(self.config.sweep_interval_secs));
        ticker.tick().await;

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if let Ok(count) = self.sweep_once().await
                        && count > 0 {
                            tracing::debug!(
                                count,
                                "expiry sweep completed"
                            );
                        }
                }
                _ = shutdown.changed() => {
                    tracing::debug!("expiry sweeper shutting down");
                    break;
                }
            }
        }
    }
}

/// Handle to stop the expiry sweeper.
#[derive(Clone)]
pub struct ExpirySweeperHandle(watch::Sender<()>);

impl ExpirySweeperHandle {
    /// Stop the sweeper.
    pub fn stop(&self) {
        let _ = self.0.send(());
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::certificate::{CertificateType, LoanTerms};
    use crate::types::Did;

    #[tokio::test]
    async fn sweep_once_no_loans() {
        let service = LoamSpineService::new();
        let sweeper = ExpirySweeper::new(service, ExpirySweeperConfig::default());
        let count = sweeper.sweep_once().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn sweep_once_returns_expired() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");
        let borrower = Did::new("did:key:z6MkBorrower");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap();

        let cert_type = CertificateType::DigitalGame {
            platform: "steam".into(),
            game_id: "test".into(),
            edition: None,
        };

        let (cert_id, _hash) = service
            .mint_certificate(spine_id, cert_type, owner.clone(), None)
            .await
            .unwrap();

        let terms = LoanTerms::new().with_duration(1).with_auto_return(true);

        service
            .loan_certificate(cert_id, owner.clone(), borrower.clone(), terms)
            .await
            .unwrap();

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let sweeper = ExpirySweeper::new(service.clone(), ExpirySweeperConfig::default());
        let count = sweeper.sweep_once().await.unwrap();
        assert_eq!(count, 1);

        let cert = service.get_certificate(cert_id).await;
        assert!(cert.is_some());
        assert!(!cert.unwrap().is_loaned());
    }

    #[test]
    fn expiry_sweeper_config_default() {
        let config = ExpirySweeperConfig::default();
        assert_eq!(config.sweep_interval_secs, DEFAULT_SWEEP_INTERVAL_SECS);
    }

    #[test]
    fn expiry_sweeper_config_with_interval() {
        let config = ExpirySweeperConfig::default().with_interval(120);
        assert_eq!(config.sweep_interval_secs, 120);
    }

    #[tokio::test]
    async fn sweep_once_skips_loan_without_auto_return() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");
        let borrower = Did::new("did:key:z6MkBorrower");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap();

        let cert_type = CertificateType::DigitalGame {
            platform: "steam".into(),
            game_id: "test".into(),
            edition: None,
        };

        let (cert_id, _hash) = service
            .mint_certificate(spine_id, cert_type, owner.clone(), None)
            .await
            .unwrap();

        let terms = LoanTerms::new().with_duration(1).with_auto_return(false);

        service
            .loan_certificate(cert_id, owner.clone(), borrower.clone(), terms)
            .await
            .unwrap();

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let sweeper = ExpirySweeper::new(service.clone(), ExpirySweeperConfig::default());
        let count = sweeper.sweep_once().await.unwrap();
        assert_eq!(count, 0, "loan without auto_return should not be swept");
    }

    #[tokio::test]
    async fn sweep_once_skips_loan_without_expires_at() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");
        let borrower = Did::new("did:key:z6MkBorrower");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap();

        let cert_type = CertificateType::DigitalGame {
            platform: "steam".into(),
            game_id: "test".into(),
            edition: None,
        };

        let (cert_id, _hash) = service
            .mint_certificate(spine_id, cert_type, owner.clone(), None)
            .await
            .unwrap();

        let terms = LoanTerms::new().with_auto_return(true);

        service
            .loan_certificate(cert_id, owner.clone(), borrower.clone(), terms)
            .await
            .unwrap();

        let sweeper = ExpirySweeper::new(service.clone(), ExpirySweeperConfig::default());
        let count = sweeper.sweep_once().await.unwrap();
        assert_eq!(count, 0, "loan without expires_at should not be swept");
    }

    #[tokio::test]
    async fn sweep_once_multiple_certs_mixed_expiry() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");
        let borrower = Did::new("did:key:z6MkBorrower");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap();

        let cert_type = CertificateType::DigitalGame {
            platform: "steam".into(),
            game_id: "test".into(),
            edition: None,
        };

        let (cert1, _) = service
            .mint_certificate(spine_id, cert_type.clone(), owner.clone(), None)
            .await
            .unwrap();
        let (cert2, _) = service
            .mint_certificate(spine_id, cert_type.clone(), owner.clone(), None)
            .await
            .unwrap();
        let (cert3, _) = service
            .mint_certificate(spine_id, cert_type, owner.clone(), None)
            .await
            .unwrap();

        let terms = LoanTerms::new().with_duration(1).with_auto_return(true);

        service
            .loan_certificate(cert1, owner.clone(), borrower.clone(), terms.clone())
            .await
            .unwrap();
        service
            .loan_certificate(cert2, owner.clone(), borrower.clone(), terms.clone())
            .await
            .unwrap();
        service
            .loan_certificate(cert3, owner.clone(), borrower.clone(), terms)
            .await
            .unwrap();

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let sweeper = ExpirySweeper::new(service.clone(), ExpirySweeperConfig::default());
        let count = sweeper.sweep_once().await.unwrap();
        assert_eq!(count, 3);

        assert!(!service.get_certificate(cert1).await.unwrap().is_loaned());
        assert!(!service.get_certificate(cert2).await.unwrap().is_loaned());
        assert!(!service.get_certificate(cert3).await.unwrap().is_loaned());
    }

    #[tokio::test]
    async fn expiry_sweeper_spawn_and_shutdown() {
        let service = LoamSpineService::new();
        let config = ExpirySweeperConfig::default().with_interval(60);
        let sweeper = ExpirySweeper::new(service, config);

        let handle = sweeper.spawn();
        handle.stop();

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
