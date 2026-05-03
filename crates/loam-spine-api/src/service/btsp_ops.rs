// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP Phase 3 negotiation handler.
//!
//! Derives session keys from the Tower-provided handshake key (Pattern B)
//! and returns `cipher: "chacha20-poly1305"` with a server nonce for HKDF.
//! Falls back to `cipher: "null"` when no handshake key is available
//! (e.g. covalent same-family bonds where BTSP was not authenticated).

use super::LoamSpineRpcService;
use crate::error::ApiResult;
use crate::types::{BtspNegotiateRequest, BtspNegotiateResponse};
use loam_spine_core::btsp::{CIPHER_CHACHA20_POLY1305, CIPHER_NULL};
use tracing::debug;

impl LoamSpineRpcService {
    /// Handle BTSP Phase 3 cipher negotiation.
    ///
    /// If a handshake key is registered for the session (from `BearDog`'s
    /// `btsp.session.verify` response), derives session keys and returns
    /// `cipher: "chacha20-poly1305"` with a base64-encoded server nonce.
    ///
    /// Falls back to `cipher: "null"` when no handshake key is available.
    ///
    /// # Errors
    ///
    /// Returns error if nonce generation fails.
    pub async fn negotiate_btsp(
        &self,
        request: BtspNegotiateRequest,
    ) -> ApiResult<BtspNegotiateResponse> {
        if self
            .get_btsp_handshake_key(&request.session_id)
            .await
            .is_some()
        {
            use base64::Engine;

            let server_nonce =
                loam_spine_core::btsp::generate_nonce().map_err(crate::error::ApiError::from)?;

            let server_nonce_b64 = base64::engine::general_purpose::STANDARD.encode(server_nonce);

            debug!(
                session_id = %request.session_id,
                cipher = CIPHER_CHACHA20_POLY1305,
                "BTSP Phase 3 negotiate: encrypted channel (Tower-provided key)"
            );

            Ok(BtspNegotiateResponse {
                cipher: CIPHER_CHACHA20_POLY1305.into(),
                server_nonce: Some(server_nonce_b64),
            })
        } else {
            debug!(
                session_id = %request.session_id,
                cipher = CIPHER_NULL,
                "BTSP Phase 3 negotiate: null cipher fallback (no handshake key)"
            );

            Ok(BtspNegotiateResponse {
                cipher: CIPHER_NULL.into(),
                server_nonce: None,
            })
        }
    }
}
