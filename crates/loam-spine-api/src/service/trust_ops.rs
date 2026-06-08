// SPDX-License-Identifier: AGPL-3.0-or-later

//! Trust ledger RPC operations — bridge from API types to core service.

use crate::error::ApiResult;
use crate::types::{
    TrustAnchorRequest, TrustAnchorResponse, TrustEventCountRequest, TrustEventCountResponse,
    TrustQueryRequest, TrustQueryResponse,
};

use super::LoamSpineRpcService;

impl LoamSpineRpcService {
    /// Anchor a cross-gate trust event as a permanent ledger entry.
    ///
    /// # Errors
    ///
    /// Returns error if the entry type is not a trust-domain variant
    /// or if the core trust anchor operation fails.
    pub async fn trust_anchor(
        &self,
        request: TrustAnchorRequest,
    ) -> ApiResult<TrustAnchorResponse> {
        let core = self.core().await;
        let (entry_hash, index) = core.trust_anchor_event(request.entry_type).await?;

        Ok(TrustAnchorResponse { entry_hash, index })
    }

    /// Query trust events involving a specific gate DID.
    ///
    /// # Errors
    ///
    /// Returns error on internal failure.
    pub async fn trust_query(&self, request: TrustQueryRequest) -> ApiResult<TrustQueryResponse> {
        let core = self.core().await;
        let events = core.trust_query_by_gate(&request.gate_did).await;

        Ok(TrustQueryResponse { events })
    }

    /// Return the number of trust events anchored in the ledger.
    ///
    /// # Errors
    ///
    /// Returns error on internal failure.
    pub async fn trust_event_count(
        &self,
        _request: TrustEventCountRequest,
    ) -> ApiResult<TrustEventCountResponse> {
        let core = self.core().await;
        let count = core.trust_event_count().await;

        Ok(TrustEventCountResponse { count })
    }
}
