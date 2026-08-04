// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP (Bonded Tunnel Secure Protocol) handshake integration.
//!
//! Implements both **server-side** and **client-side** BTSP Phase 2 handshakes.
//! Server-side delegates crypto to the BTSP provider via JSON-RPC. Client-side
//! (in [`super::btsp_client`]) computes HMAC-SHA256 locally for outbound
//! connections to the custodian/Tower provider.
//!
//! ## Architecture
//!
//! ### Server-side (incoming connections)
//!
//! ```text
//! Client ──connect──▶ LoamSpine UDS
//!                        │
//!                        ├─ Read ClientHello
//!                        ├─ Call BTSP provider btsp.session.create
//!                        ├─ Send ServerHello to client
//!                        ├─ Read ChallengeResponse → btsp.session.verify
//!                        ├─ Call btsp.negotiate → HandshakeComplete
//!                        └─ Return BtspSession
//! ```
//!
//! ### Client-side (outbound to custodian/Tower provider)
//!
//! See [`super::btsp_client`] — wired into `crypto_provider_call` and
//! `ProviderConn::connect`.
//!
//! ## Module Structure
//!
//! | Module | Responsibility |
//! |--------|---------------|
//! | [`wire`] | Serializable handshake message types |
//! | [`config`] | Environment-driven BTSP configuration |
//! | [`frame`] | Length-prefixed frame I/O |
//! | `provider_client` | JSON-RPC delegation to BTSP provider (internal) |
//! | [`handshake`] | Server-side handshake protocol |

pub mod config;
pub mod frame;
pub mod handshake;
pub mod phase3;
pub mod wire;

mod provider_client;

pub use config::{
    BtspHandshakeConfig, is_btsp_required, is_btsp_required_with, resolve_provider_socket,
    resolve_provider_socket_with,
};
pub use frame::{read_frame, write_frame};
pub use handshake::{perform_ndjson_server_handshake, perform_server_handshake};
pub use phase3::{
    CIPHER_CHACHA20_POLY1305, CIPHER_NULL, SessionKeys, generate_nonce, read_encrypted_frame,
    write_encrypted_frame,
};
pub use wire::{
    BtspSession, ChallengeResponse, ClientHello, HandshakeComplete, HandshakeError,
    NdjsonClientHello, NdjsonServerHello, ServerHello,
};

#[cfg(test)]
#[path = "../btsp_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../btsp_tests_integration.rs"]
mod integration_tests;
