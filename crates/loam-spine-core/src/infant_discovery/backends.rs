// SPDX-License-Identifier: AGPL-3.0-or-later

//! Discovery backend implementations: mDNS-SD, DNS SRV name mapping.
//!
//! Extracted from `mod.rs` to keep the main discovery orchestration
//! module focused on the infant-learning lifecycle. Backend-specific
//! parsing and protocol details live here.

#[cfg(feature = "mdns")]
use std::collections::HashMap;
#[cfg(feature = "mdns")]
use std::time::{Duration, SystemTime};

/// How long to wait for mDNS responses before giving up.
#[cfg(feature = "mdns")]
const MDNS_TIMEOUT: Duration = Duration::from_secs(2);

#[cfg(any(feature = "dns-srv", feature = "mdns", test))]
use crate::capabilities::identifiers::external;
#[cfg(feature = "mdns")]
use crate::constants::HTTPS_DEFAULT_PORT;

#[cfg(feature = "mdns")]
use crate::capabilities::{DiscoveredService, ServiceHealth};
#[cfg(feature = "mdns")]
use tracing::{debug, info, warn};

/// mDNS-SD discovery implementation (async, tokio-compatible).
///
/// Uses `mdns-sd` which manages its own daemon thread internally.
/// Results arrive via `recv_async()` on a flume channel — no second
/// async runtime, no thread isolation needed.
#[cfg(feature = "mdns")]
pub(super) async fn mdns_discover_impl(
    service_type: &str,
    capability: &str,
    cache_ttl_secs: u64,
) -> Vec<DiscoveredService> {
    let daemon = match mdns_sd::ServiceDaemon::new() {
        Ok(d) => d,
        Err(e) => {
            warn!("mDNS-SD daemon creation failed: {e}");
            return vec![];
        }
    };

    let receiver = match daemon.browse(service_type) {
        Ok(r) => r,
        Err(e) => {
            warn!("mDNS-SD browse failed for {service_type}: {e}");
            let _ = daemon.shutdown();
            return vec![];
        }
    };

    let mut services = Vec::new();

    let collect = async {
        loop {
            match receiver.recv_async().await {
                Ok(mdns_sd::ServiceEvent::ServiceResolved(info)) => {
                    if let Some(svc) = resolved_to_discovered(&info, capability, cache_ttl_secs) {
                        services.push(svc);
                    }
                }
                Ok(mdns_sd::ServiceEvent::SearchStopped(_)) | Err(_) => break,
                Ok(_) => {}
            }
        }
    };

    let _ = tokio::time::timeout(MDNS_TIMEOUT, collect).await;
    let _ = daemon.shutdown();

    if services.is_empty() {
        debug!("No mDNS-SD services found for {service_type}");
    } else {
        info!(
            "Found {} services via mDNS-SD for '{capability}'",
            services.len()
        );
    }

    services
}

/// Convert a resolved `mdns-sd` service into a `DiscoveredService`.
#[cfg(feature = "mdns")]
fn resolved_to_discovered(
    info: &mdns_sd::ServiceInfo,
    capability: &str,
    cache_ttl_secs: u64,
) -> Option<DiscoveredService> {
    let port = info.get_port();
    let addr = info.get_addresses().iter().next()?;

    let endpoint = if port == HTTPS_DEFAULT_PORT {
        format!("https://{addr}")
    } else {
        format!("https://{addr}:{port}")
    };

    let id = format!("mdns-{addr}:{port}");

    Some(DiscoveredService {
        id,
        capability: capability.to_string(),
        endpoint,
        discovered_via: crate::constants::discovery_method::MDNS.to_string(),
        metadata: HashMap::new(),
        health: ServiceHealth::Unknown,
        discovered_at: SystemTime::now(),
        ttl_secs: cache_ttl_secs,
    })
}

/// Convert capability to DNS SRV service name (RFC 2782).
///
/// Maps capability identifiers from [`crate::capabilities::identifiers::external`]
/// to their corresponding SRV record names.
///
/// Examples:
/// - `"cryptographic-signing"` -> `"_signing._tcp.local"`
/// - `"content-storage"` -> `"_storage._tcp.local"`
/// - `"service-discovery"` -> `"_discovery._tcp.local"`
#[cfg(any(feature = "dns-srv", feature = "mdns", test))]
pub(super) fn capability_to_srv_name(capability: &str) -> String {
    let service_part = match capability {
        external::SIGNING => "signing",
        external::STORAGE => "storage",
        external::DISCOVERY => "discovery",
        external::SESSION_MANAGEMENT => "session",
        external::COMPUTE => "compute",
        external::ATTESTATION => "attestation",
        other => other.split('-').next_back().unwrap_or("service"),
    };

    format!("_{service_part}._tcp.local")
}
