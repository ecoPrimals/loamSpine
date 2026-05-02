// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP Phase 3 negotiation handler.
//!
//! Returns `cipher: "null"` (plaintext fallback) until the BTSP provider
//! API supports key material export for server-side AEAD framing.
//! Clients (primalSpring) handle null-cipher gracefully — no breakage.

use super::LoamSpineRpcService;
use crate::error::ApiResult;
use crate::types::{BtspNegotiateRequest, BtspNegotiateResponse};
use tracing::debug;

impl LoamSpineRpcService {
    /// Handle BTSP Phase 3 cipher negotiation.
    ///
    /// Currently returns `cipher: "null"` (authenticated plaintext).
    /// Full Phase 3 encrypted framing requires the BTSP provider to
    /// export session key material, which is not yet available.
    ///
    /// # Errors
    ///
    /// Returns error if the request is malformed.
    pub async fn negotiate_btsp(
        &self,
        request: BtspNegotiateRequest,
    ) -> ApiResult<BtspNegotiateResponse> {
        debug!(
            session_id = %request.session_id,
            preferred_cipher = %request.preferred_cipher,
            "BTSP Phase 3 negotiate: returning null cipher (plaintext fallback)"
        );

        Ok(BtspNegotiateResponse {
            cipher: "null".into(),
            server_nonce: None,
        })
    }
}
