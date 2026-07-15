// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for integration trait implementations.
//!
//! Split into focused submodules per domain:
//! - `spine_ops` — core spine operations (slice checkout/resolve, entries, commits, braids)
//! - `slice_mgr` — SliceManager extended operations (mark, clear, status, list)
//! - `provenance` — ProvenanceSource + Nest depth convergence verification

#![expect(
    clippy::expect_used,
    reason = "test assertions use expect for failure clarity"
)]

use super::*;
use crate::types::Timestamp;

#[path = "integration_tests_spine_ops.rs"]
mod spine_ops;

#[path = "integration_tests_slice_mgr.rs"]
mod slice_mgr;

#[path = "integration_tests_provenance.rs"]
mod provenance;
