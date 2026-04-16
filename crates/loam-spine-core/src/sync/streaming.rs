// SPDX-License-Identifier: AGPL-3.0-or-later

//! Streaming sync helpers for NDJSON progress reporting.
//!
//! Wraps the core `SyncEngine` push/pull operations with bounded `mpsc`
//! channel progress, enabling ecosystem pipeline coordination graphs to
//! wire springs together with observable data flow.

use tracing::{debug, warn};

use crate::entry::Entry;
use crate::error::LoamSpineResult;
use crate::streaming::StreamItem;
use crate::traits::SyncResult;
use crate::types::SpineId;

use super::SyncEngine;

impl SyncEngine {
    /// Push entries to peers with NDJSON progress streaming.
    ///
    /// Emits [`Progress`](crate::streaming::StreamItem::Progress) during
    /// the push and [`End`](crate::streaming::StreamItem::End) on completion.
    ///
    /// # Errors
    ///
    /// Returns an error if no peers are registered.
    pub async fn push_entries_streaming(
        &self,
        spine_id: SpineId,
        entries: Vec<Entry>,
        progress: &tokio::sync::mpsc::Sender<StreamItem>,
    ) -> LoamSpineResult<SyncResult> {
        let endpoint = self.best_peer_endpoint().await?;
        let entry_count = u64::try_from(entries.len()).unwrap_or(u64::MAX);

        let _ = progress
            .send(StreamItem::progress(0, Some(entry_count)))
            .await;

        let push_result = self.push_to_peer(&endpoint, spine_id, &entries).await;

        self.spine_states
            .write()
            .await
            .entry(spine_id)
            .or_default()
            .pending_push
            .extend(entries);

        match push_result {
            Ok(result) => {
                debug!(
                    "Pushed {} entries to peer {} for spine {}",
                    result.accepted, endpoint, spine_id
                );
                let _ = progress
                    .send(StreamItem::progress(result.accepted, Some(entry_count)))
                    .await;
                let _ = progress.send(StreamItem::end()).await;
                Ok(result)
            }
            Err(e) => {
                warn!(
                    "Streaming push to {} failed (entries queued locally): {e}",
                    endpoint
                );
                let _ = progress
                    .send(StreamItem::error(format!("push to {endpoint}: {e}")))
                    .await;
                let _ = progress.send(StreamItem::end()).await;
                Ok(SyncResult {
                    accepted: entry_count,
                    rejected: 0,
                    rejection_reasons: Vec::new(),
                })
            }
        }
    }

    /// Pull entries from peers with NDJSON progress streaming.
    ///
    /// # Errors
    ///
    /// Returns an error if no peers are registered.
    pub async fn pull_entries_streaming(
        &self,
        spine_id: SpineId,
        from_index: u64,
        limit: u64,
        progress: &tokio::sync::mpsc::Sender<StreamItem>,
    ) -> LoamSpineResult<Vec<Entry>> {
        let endpoint = self.best_peer_endpoint().await?;
        let _ = progress.send(StreamItem::progress(0, None)).await;

        match self
            .pull_from_peer(&endpoint, spine_id, from_index, limit)
            .await
        {
            Ok(entries) => {
                let count = u64::try_from(entries.len()).unwrap_or(u64::MAX);
                debug!(
                    "Pulled {} entries from peer {} for spine {}",
                    entries.len(),
                    endpoint,
                    spine_id
                );
                let _ = progress
                    .send(StreamItem::progress(count, Some(count)))
                    .await;
                let _ = progress.send(StreamItem::end()).await;
                Ok(entries)
            }
            Err(e) => {
                warn!(
                    "Streaming pull from {} failed, returning local queue: {e}",
                    endpoint
                );
                let _ = progress
                    .send(StreamItem::error(format!("pull from {endpoint}: {e}")))
                    .await;
                let _ = progress.send(StreamItem::end()).await;
                let limit_usize = usize::try_from(limit).unwrap_or(usize::MAX);
                let states = self.spine_states.read().await;
                Ok(states.get(&spine_id).map_or_else(Vec::new, |state| {
                    state
                        .pending_push
                        .iter()
                        .filter(|e| e.index >= from_index)
                        .take(limit_usize)
                        .cloned()
                        .collect()
                }))
            }
        }
    }
}
