// SPDX-License-Identifier: AGPL-3.0-or-later

//! NDJSON streaming types for pipeline coordination.
//!
//! Used by biomeOS Pipeline coordination graphs to wire bounded `mpsc`
//! channels between springs. Items flow through as each node produces them.
//!
//! Aligns with the ecosystem `StreamItem` and NDJSON pipeline
//! protocol for cross-primal streaming interoperability.

use serde::{Deserialize, Serialize};

/// Protocol version for NDJSON stream framing.
///
/// Included in capability advertisements so peers can negotiate
/// compatible framing. Bump on backward-incompatible changes to
/// the `StreamItem` schema.
pub const NDJSON_PROTOCOL_VERSION: &str = "1.0";

/// A single item in an NDJSON stream.
///
/// Each variant is tagged by `"type"` in the JSON representation,
/// enabling incremental parsing of newline-delimited streams.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum StreamItem {
    /// A data payload in the stream.
    Data {
        /// The payload content (schema depends on the method).
        payload: serde_json::Value,
    },
    /// Stream progress indicator.
    Progress {
        /// Items processed so far.
        processed: u64,
        /// Total items (if known).
        total: Option<u64>,
    },
    /// End of stream marker.
    End,
    /// Stream-level error (non-fatal; stream may continue).
    Error {
        /// Error message.
        message: String,
        /// Whether the stream can continue after this error.
        recoverable: bool,
    },
}

impl StreamItem {
    /// Create a data item with the given payload.
    #[must_use]
    pub const fn data(payload: serde_json::Value) -> Self {
        Self::Data { payload }
    }

    /// Create an end-of-stream marker.
    #[must_use]
    pub const fn end() -> Self {
        Self::End
    }

    /// Create a progress indicator.
    #[must_use]
    pub const fn progress(processed: u64, total: Option<u64>) -> Self {
        Self::Progress { processed, total }
    }

    /// Create a recoverable stream error.
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            recoverable: true,
        }
    }

    /// Create a fatal (non-recoverable) stream error.
    #[must_use]
    pub fn fatal(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            recoverable: false,
        }
    }

    /// Serialize this item as a single NDJSON line (with trailing newline).
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails.
    pub fn to_ndjson_line(&self) -> Result<String, serde_json::Error> {
        let mut line = serde_json::to_string(self)?;
        line.push('\n');
        Ok(line)
    }

    /// Parse a single NDJSON line into a `StreamItem`.
    ///
    /// # Errors
    ///
    /// Returns an error if the line is not valid JSON or doesn't match
    /// the `StreamItem` schema.
    pub fn parse_ndjson_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line.trim())
    }

    /// Whether this item signals end of stream.
    #[must_use]
    pub const fn is_end(&self) -> bool {
        matches!(self, Self::End)
    }

    /// Whether this item is a fatal error.
    #[must_use]
    pub const fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::Error {
                recoverable: false,
                ..
            }
        )
    }
}

/// Maximum items to accumulate before applying backpressure.
///
/// Prevents unbounded memory growth when reading large streams.
/// Callers needing larger streams should use
/// [`read_ndjson_stream_bounded`] with an explicit limit.
pub const DEFAULT_NDJSON_MAX_ITEMS: usize = 10_000;

/// Read `StreamItem`s from an async buffered reader (NDJSON framing).
///
/// Reads lines until EOF, a fatal/end item, or the default item limit
/// ([`DEFAULT_NDJSON_MAX_ITEMS`]) is reached. Blank lines are skipped.
/// Parse errors emit a recoverable `StreamItem::Error`.
///
/// # Errors
///
/// Returns an I/O error only if the underlying reader fails.
pub async fn read_ndjson_stream<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
) -> std::io::Result<Vec<StreamItem>> {
    read_ndjson_stream_bounded(reader, DEFAULT_NDJSON_MAX_ITEMS).await
}

/// Read `StreamItem`s with an explicit item limit for backpressure.
///
/// When `max_items` items have been accumulated the stream is
/// terminated with a `StreamItem::Error` indicating the limit was hit.
///
/// # Errors
///
/// Returns an I/O error only if the underlying reader fails.
pub async fn read_ndjson_stream_bounded<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
    max_items: usize,
) -> std::io::Result<Vec<StreamItem>> {
    use tokio::io::AsyncBufReadExt;

    let mut items = Vec::new();
    let mut line = String::new();

    loop {
        if items.len() >= max_items {
            items.push(StreamItem::fatal(format!(
                "NDJSON backpressure: item limit ({max_items}) reached"
            )));
            break;
        }

        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match StreamItem::parse_ndjson_line(trimmed) {
            Ok(item) => {
                let terminal = item.is_end() || item.is_fatal();
                items.push(item);
                if terminal {
                    break;
                }
            }
            Err(e) => {
                items.push(StreamItem::error(format!("NDJSON parse error: {e}")));
            }
        }
    }

    Ok(items)
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
#[path = "streaming_tests.rs"]
mod tests;
