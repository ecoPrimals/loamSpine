// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP (Bonded Tunnel Secure Protocol) handshake integration.
//!
//! Implements the **consumer side** of BTSP Phase 2 for LoamSpine's UDS listener.
//! LoamSpine does NOT implement cryptographic operations directly — all crypto
//! is delegated to the BTSP provider via JSON-RPC ("handshake-as-a-service").
//!
//! ## Architecture
//!
//! ```text
//! Client ──connect──▶ LoamSpine UDS
//!                        │
//!                        ├─ Read ClientHello (length-prefixed frame)
//!                        ├─ Call BTSP provider btsp.session.create → get server keys
//!                        ├─ Send ServerHello to client
//!                        ├─ Read ChallengeResponse from client
//!                        ├─ Call BTSP provider btsp.session.verify → verify HMAC
//!                        ├─ Call BTSP provider btsp.negotiate → cipher suite
//!                        ├─ Send HandshakeComplete / HandshakeError
//!                        └─ Return BtspSession on success
//! ```
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
pub mod wire;

mod provider_client;

pub use config::{
    BtspHandshakeConfig, is_btsp_required, is_btsp_required_with, resolve_provider_socket,
    resolve_provider_socket_with,
};
pub use frame::{read_frame, write_frame};
pub use handshake::perform_server_handshake;
pub use wire::{
    BtspSession, ChallengeResponse, ClientHello, HandshakeComplete, HandshakeError, ServerHello,
};

#[cfg(test)]
#[path = "../btsp_tests.rs"]
mod tests;
