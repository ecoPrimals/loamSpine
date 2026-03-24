// SPDX-License-Identifier: AGPL-3.0-or-later

//! NDJSON streaming types for pipeline coordination.
//!
//! Used by biomeOS Pipeline coordination graphs to wire bounded `mpsc`
//! channels between springs. Items flow through as each node produces them.
//!
//! Aligns with rhizoCrypt's `StreamItem` and sweetGrass's NDJSON pipeline
//! protocol for ecosystem-wide streaming interoperability.

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
    pub fn data(payload: serde_json::Value) -> Self {
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

/// Read `StreamItem`s from an async buffered reader (NDJSON framing).
///
/// Reads lines until EOF or a fatal/end item is encountered. Blank lines
/// are skipped. Parse errors emit a recoverable `StreamItem::Error`.
///
/// # Errors
///
/// Returns an I/O error only if the underlying reader fails.
pub async fn read_ndjson_stream<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
) -> std::io::Result<Vec<StreamItem>> {
    use tokio::io::AsyncBufReadExt;

    let mut items = Vec::new();
    let mut line = String::new();

    loop {
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
mod tests {
    use super::*;

    #[test]
    fn data_item_roundtrip() {
        let item = StreamItem::data(serde_json::json!({"spine_id": "abc123"}));
        let line = item.to_ndjson_line().unwrap();
        assert!(line.ends_with('\n'));
        let parsed = StreamItem::parse_ndjson_line(&line).unwrap();
        assert_eq!(item, parsed);
    }

    #[test]
    fn end_marker() {
        let item = StreamItem::end();
        assert!(item.is_end());
        assert!(!item.is_fatal());
        let line = item.to_ndjson_line().unwrap();
        let parsed = StreamItem::parse_ndjson_line(&line).unwrap();
        assert!(parsed.is_end());
    }

    #[test]
    fn progress_with_total() {
        let item = StreamItem::progress(42, Some(100));
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["type"], "Progress");
        assert_eq!(json["processed"], 42);
        assert_eq!(json["total"], 100);
    }

    #[test]
    fn progress_without_total() {
        let item = StreamItem::progress(7, None);
        let json = serde_json::to_value(&item).unwrap();
        assert!(json["total"].is_null());
    }

    #[test]
    fn recoverable_error() {
        let item = StreamItem::error("transient failure");
        assert!(!item.is_fatal());
        assert!(!item.is_end());
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["recoverable"], true);
    }

    #[test]
    fn fatal_error() {
        let item = StreamItem::fatal("corruption detected");
        assert!(item.is_fatal());
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["recoverable"], false);
    }

    #[test]
    fn parse_invalid_ndjson() {
        assert!(StreamItem::parse_ndjson_line("not json").is_err());
    }

    #[test]
    fn protocol_version_is_semver() {
        assert!(NDJSON_PROTOCOL_VERSION.contains('.'));
    }

    #[tokio::test]
    async fn read_ndjson_stream_parses_items() {
        let input = format!(
            "{}{}{}\n",
            StreamItem::data(serde_json::json!({"id": 1}))
                .to_ndjson_line()
                .unwrap(),
            StreamItem::progress(1, Some(2)).to_ndjson_line().unwrap(),
            StreamItem::end().to_ndjson_line().unwrap().trim_end(),
        );
        let mut cursor = std::io::Cursor::new(input.as_bytes().to_vec());
        let mut reader = tokio::io::BufReader::new(&mut cursor);
        let items = super::read_ndjson_stream(&mut reader).await.unwrap();
        assert_eq!(items.len(), 3);
        assert!(items[2].is_end());
    }

    #[tokio::test]
    async fn read_ndjson_stream_stops_on_fatal() {
        let input = format!(
            "{}{}",
            StreamItem::fatal("boom").to_ndjson_line().unwrap(),
            StreamItem::data(serde_json::json!({"after_fatal": true}))
                .to_ndjson_line()
                .unwrap(),
        );
        let mut cursor = std::io::Cursor::new(input.as_bytes().to_vec());
        let mut reader = tokio::io::BufReader::new(&mut cursor);
        let items = super::read_ndjson_stream(&mut reader).await.unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].is_fatal());
    }

    #[tokio::test]
    async fn read_ndjson_stream_handles_parse_errors() {
        let input = "not valid json\n";
        let mut cursor = std::io::Cursor::new(input.as_bytes().to_vec());
        let mut reader = tokio::io::BufReader::new(&mut cursor);
        let items = super::read_ndjson_stream(&mut reader).await.unwrap();
        assert_eq!(items.len(), 1);
        assert!(matches!(
            items[0],
            StreamItem::Error {
                recoverable: true,
                ..
            }
        ));
    }

    #[test]
    fn ndjson_multi_line_stream() {
        let items = vec![
            StreamItem::data(serde_json::json!({"id": 1})),
            StreamItem::data(serde_json::json!({"id": 2})),
            StreamItem::progress(2, Some(3)),
            StreamItem::data(serde_json::json!({"id": 3})),
            StreamItem::end(),
        ];

        let stream: String = items.iter().map(|i| i.to_ndjson_line().unwrap()).collect();

        let parsed: Vec<StreamItem> = stream
            .lines()
            .map(|line| StreamItem::parse_ndjson_line(line).unwrap())
            .collect();

        assert_eq!(items, parsed);
    }

    #[tokio::test]
    async fn read_ndjson_stream_skips_empty_lines() {
        let input = format!(
            "\n\n{}\n\n{}\n",
            StreamItem::data(serde_json::json!({"ok": true}))
                .to_ndjson_line()
                .unwrap()
                .trim_end(),
            StreamItem::end().to_ndjson_line().unwrap().trim_end(),
        );
        let mut cursor = std::io::Cursor::new(input.as_bytes().to_vec());
        let mut reader = tokio::io::BufReader::new(&mut cursor);
        let items = super::read_ndjson_stream(&mut reader).await.unwrap();
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0], StreamItem::Data { .. }));
        assert!(items[1].is_end());
    }
}
