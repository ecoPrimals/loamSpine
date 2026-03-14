// SPDX-License-Identifier: AGPL-3.0-only

//! # `LoamSpine` Pure Rust RPC API
//!
//! **Pure Rust, No Vendor Lock-in**
//!
//! This crate provides the RPC interface for `LoamSpine` using:
//! - **tarpc**: High-performance binary RPC for primal-to-primal communication
//! - **JSON-RPC 2.0**: Universal, language-agnostic RPC for external clients
//!
//! ## Why Not gRPC?
//!
//! ecoPrimals uses pure Rust RPC instead of gRPC because:
//! - ❌ gRPC requires `protoc` (C++ compiler)
//! - ❌ gRPC requires protobuf (Google tooling)
//! - ❌ Non-Rust code generation
//! - ❌ Vendor lock-in
//!
//! ## Our Approach
//!
//! - ✅ Pure Rust (no C/C++ dependencies)
//! - ✅ Native serde serialization
//! - ✅ Rust macros (tarpc procedural generation)
//! - ✅ No external tooling required
//! - ✅ Full Rust compiler type checking
//! - ✅ Community-driven development
//!
//! ## Protocol Stack
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │         LoamSpine Service Layer          │
//! ├─────────────────────────────────────────┤
//! │                                          │
//! │  tarpc (Binary)    JSON-RPC (Universal) │
//! │  ↓                 ↓                     │
//! │  Primal ←→ Primal  External Clients     │
//! │  • Ephemeral       • Python              │
//! │  • Attribution     • JavaScript          │
//! │  • Signing         • curl/httpie         │
//! │                                          │
//! └─────────────────────────────────────────┘
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod error;
pub mod health;
pub mod jsonrpc;
pub mod rpc;
pub mod service;
pub mod tarpc_server;
pub mod types;

pub use error::{ApiError, ApiResult, ServerError};
pub use jsonrpc::{run_jsonrpc_server, LoamSpineJsonRpc};
pub use rpc::LoamSpineRpc;
pub use service::LoamSpineRpcService;
pub use tarpc_server::{run_tarpc_server, LoamSpineTarpcServer};
pub use types::*;
