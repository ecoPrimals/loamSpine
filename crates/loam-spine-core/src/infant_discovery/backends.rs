// SPDX-License-Identifier: AGPL-3.0-or-later

//! Discovery backend implementations: mDNS, DNS SRV name mapping.
//!
//! Extracted from `mod.rs` to keep the main discovery orchestration
//! module focused on the infant-learning lifecycle. Backend-specific
//! parsing and protocol details live here.

#[cfg(feature = "mdns")]
use std::collections::HashMap;
#[cfg(feature = "mdns")]
use std::time::{Duration, SystemTime};

#[cfg(any(feature = "dns-srv", feature = "mdns", test))]
use crate::capabilities::identifiers::external;
#[cfg(feature = "mdns")]
use crate::constants::HTTPS_DEFAULT_PORT;

#[cfg(feature = "mdns")]
use crate::capabilities::{DiscoveredService, ServiceHealth};
#[cfg(feature = "mdns")]
use tracing::{debug, info, warn};

/// Real mDNS discovery implementation (runs in `spawn_blocking`).
///
/// Uses the `mdns` crate to query for DNS-SD services, parses SRV records,
/// and converts results to `DiscoveredService`. All errors are handled
/// gracefully (returns empty vec, logs warnings).
#[cfg(feature = "mdns")]
pub(super) fn mdns_discover_impl(
    service_type: &str,
    capability: &str,
    cache_ttl_secs: u64,
) -> Vec<DiscoveredService> {
    use std::time::Instant;

    let discovery = match mdns::discover::all(service_type, Duration::from_secs(2)) {
        Ok(d) => d,
        Err(e) => {
            warn!("mDNS discovery failed for {service_type}: {e}");
            return vec![];
        }
    };

    let stream = discovery.listen();

    let services = async_std::task::block_on(async move {
        use futures_util::{pin_mut, stream::StreamExt};

        let mut services = Vec::new();
        pin_mut!(stream);

        let deadline = Instant::now() + Duration::from_secs(2);

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }

            let next = stream.next();
            match async_std::future::timeout(remaining, next).await {
                Ok(Some(Ok(response))) => {
                    if let Some(service) =
                        parse_mdns_response(&response, capability, cache_ttl_secs)
                    {
                        services.push(service);
                    }
                }
                Ok(Some(Err(e))) => {
                    warn!("mDNS response error: {e}");
                }
                Ok(None) | Err(_) => break,
            }
        }

        services
    });

    if services.is_empty() {
        debug!("No mDNS services found for {service_type}");
    } else {
        info!(
            "Found {} services via mDNS for '{capability}'",
            services.len()
        );
    }

    services
}

/// Parse a single mDNS response into a `DiscoveredService`.
///
/// Extracts SRV records for host/port and A/AAAA for address resolution.
/// Returns `None` if the response cannot be parsed into a valid service.
#[cfg(feature = "mdns")]
fn parse_mdns_response(
    response: &mdns::Response,
    capability: &str,
    cache_ttl_secs: u64,
) -> Option<DiscoveredService> {
    use mdns::RecordKind;

    let port = response.port()?;
    let endpoint = if let Some(addr) = response.ip_addr() {
        if port == HTTPS_DEFAULT_PORT {
            format!("https://{addr}")
        } else {
            format!("https://{addr}:{port}")
        }
    } else {
        let target = response.records().find_map(|r| match &r.kind {
            RecordKind::SRV { target, .. } => Some(target.clone()),
            _ => None,
        })?;
        if port == HTTPS_DEFAULT_PORT {
            format!("https://{target}")
        } else {
            format!("https://{target}:{port}")
        }
    };

    let id = response.ip_addr().map_or_else(
        || format!("mdns-{capability}-{port}"),
        |a| format!("mdns-{a}:{port}"),
    );

    Some(DiscoveredService {
        id,
        capability: capability.to_string(),
        endpoint,
        discovered_via: "mdns".to_string(),
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
