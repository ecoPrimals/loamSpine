// SPDX-License-Identifier: AGPL-3.0-or-later

//! Discovery cache with TTL-based freshness.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use tokio::sync::RwLock;
use tracing::debug;

use crate::capabilities::DiscoveredService;

/// Cache for discovered services, keyed by capability name.
///
/// Services are evicted when their individual TTL expires. The cache is
/// safe for concurrent access via `RwLock`.
pub(crate) struct DiscoveryCache {
    entries: RwLock<HashMap<String, Vec<DiscoveredService>>>,
}

impl DiscoveryCache {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// Return fresh (within TTL) cached services for a capability, if any.
    pub async fn get_fresh(&self, capability: &str) -> Option<Vec<DiscoveredService>> {
        let discovered = self.entries.read().await;
        let services = discovered.get(capability)?;
        if services.is_empty() {
            return None;
        }
        let fresh: Vec<_> = services
            .iter()
            .filter(|s| Self::is_fresh(s))
            .cloned()
            .collect();
        if fresh.is_empty() {
            return None;
        }
        debug!("Found {} cached services for '{}'", fresh.len(), capability);
        Some(fresh)
    }

    /// Insert or replace cached services for a capability.
    pub async fn insert(&self, capability: impl Into<String>, services: Vec<DiscoveredService>) {
        self.entries
            .write()
            .await
            .insert(capability.into(), services);
    }

    /// Clear all cached entries (force rediscovery).
    pub async fn clear(&self) {
        self.entries.write().await.clear();
    }

    /// Return a snapshot of all cached services.
    pub async fn all(&self) -> HashMap<String, Vec<DiscoveredService>> {
        self.entries.read().await.clone()
    }

    /// Check if a service is still fresh (within TTL).
    pub(crate) fn is_fresh(service: &DiscoveredService) -> bool {
        let age = SystemTime::now()
            .duration_since(service.discovered_at)
            .unwrap_or(Duration::from_secs(u64::MAX));
        age.as_secs() < service.ttl_secs
    }
}
