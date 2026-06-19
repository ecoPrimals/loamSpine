// SPDX-License-Identifier: AGPL-3.0-or-later

//! Core types for LoamSpine.
//!
//! This module defines the fundamental types used throughout LoamSpine:
//! - Content addressing (`ContentHash`, `EntryHash`)
//! - Identifiers (`SpineId`, `CertificateId`, `SliceId`)
//! - Basic primitives (`Did`, `Signature`, `Timestamp`)
//! - Size constants for testing and configuration
//! - Zero-copy buffer types (`ByteBuffer`)

use std::fmt;
use std::sync::Arc;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

// ============================================================================
// Size Constants
// ============================================================================

/// One kilobyte in bytes.
pub const KB: u64 = 1024;

/// One megabyte in bytes.
pub const MB: u64 = 1024 * KB;

/// One gigabyte in bytes.
pub const GB: u64 = 1024 * MB;

/// 32-byte content hash (Blake3).
pub type ContentHash = [u8; 32];

/// Entry hash - content address of an entry.
pub type EntryHash = ContentHash;

/// Spine identifier (UUID v7 for time-ordering).
pub type SpineId = uuid::Uuid;

/// Certificate identifier (UUID v7).
pub type CertificateId = uuid::Uuid;

/// Slice identifier (UUID v7).
pub type SliceId = uuid::Uuid;

/// Session identifier (UUID v7).
pub type SessionId = uuid::Uuid;

/// Braid identifier (UUID v7).
pub type BraidId = uuid::Uuid;

/// Peer identifier for replication.
///
/// Newtype wrapper over `Arc<str>` for O(1) cloning and type safety.
/// Prevents accidental interchange with arbitrary strings.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct PeerId(Arc<str>);

impl PeerId {
    /// Create a new peer ID from a string.
    #[must_use]
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }

    /// Get the peer ID as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for PeerId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for PeerId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl PartialEq<&str> for PeerId {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl std::borrow::Borrow<str> for PeerId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

/// Decentralized Identifier (DID).
///
/// Format: `did:key:z6Mk...` or `did:web:...`
/// Used for ownership, signing identity, and access control.
///
/// Backed by `Arc<str>` for O(1) cloning — DIDs are shared across spines,
/// entries, certificates, and RPC boundaries without allocation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Did(Arc<str>);

/// Sentinel value for contexts where no identity is known or configured.
const ANONYMOUS_DID: &str = "did:primal:anonymous";

impl Did {
    /// Sentinel DID for contexts where no identity is known or configured.
    ///
    /// Used instead of hardcoded placeholder strings. Callers should check
    /// `did.is_anonymous()` rather than string-matching.
    #[must_use]
    pub fn anonymous() -> Self {
        Self(Arc::from(ANONYMOUS_DID))
    }

    /// Create a new DID.
    #[must_use]
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self(value.into())
    }

    /// Returns `true` if this DID is the anonymous sentinel.
    #[must_use]
    pub fn is_anonymous(&self) -> bool {
        self.0.as_ref() == ANONYMOUS_DID
    }

    /// Get the DID as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for Did {
    fn from(s: String) -> Self {
        Self(Arc::from(s))
    }
}

impl From<&str> for Did {
    fn from(s: &str) -> Self {
        Self(Arc::from(s))
    }
}

impl Serialize for Did {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Did {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(Arc::from(s)))
    }
}

/// Cryptographic signature.
///
/// Uses `Bytes` for zero-copy sharing of signature data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature(pub ByteBuffer);

impl Signature {
    /// Create a new signature from zero-copy bytes.
    #[must_use]
    pub const fn new(bytes: ByteBuffer) -> Self {
        Self(bytes)
    }

    /// Create a signature from a `Vec<u8>` (convenience method).
    #[must_use]
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self(ByteBuffer::from(bytes))
    }

    /// Create an empty signature (for unsigned entries).
    #[must_use]
    #[expect(
        clippy::missing_const_for_fn,
        reason = "Bytes::new() is not const-stable"
    )]
    pub fn empty() -> Self {
        Self(ByteBuffer::new())
    }

    /// Check if the signature is empty.
    #[must_use]
    #[expect(
        clippy::missing_const_for_fn,
        reason = "Bytes::is_empty() is not const-stable"
    )]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the signature bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Encode the signature as standard base64.
    #[must_use]
    pub fn to_base64(&self) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(&self.0)
    }

    /// Get the underlying ByteBuffer for zero-copy sharing.
    #[must_use]
    pub const fn as_byte_buffer(&self) -> &ByteBuffer {
        &self.0
    }
}

// Custom serde implementation for zero-copy Bytes
impl serde::Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_byte_buf(ByteBufferVisitor)
            .map(Self)
    }
}

/// Visitor that deserializes directly into [`ByteBuffer`] (`Bytes`),
/// avoiding the intermediate `Vec<u8>` allocation for binary formats.
/// JSON falls back through `visit_seq` (array of numbers) which still
/// allocates, but binary codecs (e.g. MessagePack via rmp-serde) use `visit_byte_buf`
/// for true zero-copy handoff.
struct ByteBufferVisitor;

impl<'de> serde::de::Visitor<'de> for ByteBufferVisitor {
    type Value = ByteBuffer;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("byte data")
    }

    fn visit_byte_buf<E: serde::de::Error>(self, v: Vec<u8>) -> Result<ByteBuffer, E> {
        Ok(ByteBuffer::from(v))
    }

    fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<ByteBuffer, E> {
        Ok(ByteBuffer::copy_from_slice(v))
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<ByteBuffer, A::Error> {
        let mut bytes = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(byte) = seq.next_element()? {
            bytes.push(byte);
        }
        Ok(ByteBuffer::from(bytes))
    }
}

impl Default for Signature {
    fn default() -> Self {
        Self::empty()
    }
}

/// Timestamp in nanoseconds since Unix epoch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    /// Create a timestamp for now.
    #[must_use]
    pub fn now() -> Self {
        // u128 nanoseconds won't overflow u64 until year 2554
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos().try_into().unwrap_or(u64::MAX));
        Self(nanos)
    }

    /// Create a timestamp from nanoseconds.
    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    /// Get the timestamp as nanoseconds.
    #[must_use]
    pub const fn as_nanos(&self) -> u64 {
        self.0
    }

    /// Get the timestamp as seconds.
    #[must_use]
    pub const fn as_secs(&self) -> u64 {
        self.0 / 1_000_000_000
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}ns", self.0)
    }
}

/// Reference to content-addressed payload.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PayloadRef {
    /// Content hash of the payload.
    pub hash: ContentHash,
    /// Size in bytes.
    pub size: u64,
    /// Optional MIME type.
    pub mime_type: Option<String>,
}

impl PayloadRef {
    /// Create a new payload reference.
    #[must_use]
    pub const fn new(hash: ContentHash, size: u64) -> Self {
        Self {
            hash,
            size,
            mime_type: None,
        }
    }

    /// Set the MIME type.
    #[must_use]
    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }
}

/// Compute Blake3 hash of bytes.
#[must_use]
pub fn hash_bytes(data: &[u8]) -> ContentHash {
    blake3::hash(data).into()
}

/// Format a hash as hex string (first 8 bytes).
#[must_use]
pub fn format_hash_short(hash: &ContentHash) -> String {
    hex::encode(&hash[..8])
}

/// Hex encoding utilities.
mod hex {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

    pub fn encode(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            s.push(char::from(HEX_CHARS[usize::from(b >> 4)]));
            s.push(char::from(HEX_CHARS[usize::from(b & 0xf)]));
        }
        s
    }
}

// ============================================================================
// Zero-Copy Buffer Types
// ============================================================================

/// Zero-copy byte buffer for network operations.
///
/// This is a type alias for `bytes::Bytes`, providing reference-counted,
/// zero-copy slicing of byte data. Use this in RPC request/response types
/// instead of `Vec<u8>` for better performance.
///
/// # Example
///
/// ```rust
/// use loam_spine_core::types::ByteBuffer;
///
/// let buffer: ByteBuffer = ByteBuffer::from_static(b"hello world");
/// let slice = buffer.slice(0..5); // Zero-copy slice
/// assert_eq!(&slice[..], b"hello");
/// ```
pub type ByteBuffer = Bytes;

// ============================================================================
// Serde helpers for ContentHash / EntryHash — accept hex strings or byte arrays
// ============================================================================

/// Deserialize a `[u8; 32]` from either a JSON byte array or a hex string.
///
/// Use via `#[serde(deserialize_with = "serde_content_hash::deserialize")]`.
pub mod serde_content_hash {
    use serde::de::{self, Deserializer, SeqAccess, Visitor};
    use std::fmt;

    pub(crate) struct ContentHashVisitor;

    impl<'de> Visitor<'de> for ContentHashVisitor {
        type Value = [u8; 32];

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a 32-byte array or 64-character hex string")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<[u8; 32], A::Error> {
            let mut arr = [0u8; 32];
            for (i, byte) in arr.iter_mut().enumerate() {
                *byte = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(i, &"32 bytes"))?;
            }
            Ok(arr)
        }

        fn visit_str<E: de::Error>(self, s: &str) -> Result<[u8; 32], E> {
            parse_hex(s).map_err(de::Error::custom)
        }

        fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<[u8; 32], E> {
            <[u8; 32]>::try_from(v).map_err(|_| de::Error::invalid_length(v.len(), &"32 bytes"))
        }
    }

    /// Deserialize `[u8; 32]` from a byte array or hex string.
    ///
    /// # Errors
    ///
    /// Returns a deserialization error for invalid hex or wrong length.
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<[u8; 32], D::Error> {
        deserializer.deserialize_any(ContentHashVisitor)
    }

    /// Hex-decode error for content hash parsing.
    #[derive(Debug)]
    pub(crate) enum HexError {
        BadLength(usize),
        InvalidByte {
            index: usize,
            source: std::num::ParseIntError,
        },
    }

    impl std::fmt::Display for HexError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::BadLength(n) => write!(f, "expected 64 hex chars, got {n}"),
                Self::InvalidByte { index, source } => {
                    write!(f, "invalid hex at byte {index}: {source}")
                }
            }
        }
    }

    pub(crate) fn parse_hex(s: &str) -> Result<[u8; 32], HexError> {
        let hex = s.strip_prefix("0x").unwrap_or(s);
        if hex.len() != 64 {
            return Err(HexError::BadLength(hex.len()));
        }
        let mut arr = [0u8; 32];
        for (i, byte) in arr.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).map_err(|e| {
                HexError::InvalidByte {
                    index: i,
                    source: e,
                }
            })?;
        }
        Ok(arr)
    }
}

/// Deserialize an `Option<[u8; 32]>` from either a JSON byte array, hex string, or null.
///
/// Use via `#[serde(default, deserialize_with = "serde_opt_content_hash::deserialize")]`.
pub mod serde_opt_content_hash {
    use serde::de::{self, Deserializer, SeqAccess, Visitor};
    use std::fmt;

    struct OptContentHashVisitor;

    impl<'de> Visitor<'de> for OptContentHashVisitor {
        type Value = Option<[u8; 32]>;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("null, a 32-byte array, or a 64-character hex string")
        }

        fn visit_none<E: de::Error>(self) -> Result<Option<[u8; 32]>, E> {
            Ok(None)
        }

        fn visit_unit<E: de::Error>(self) -> Result<Option<[u8; 32]>, E> {
            Ok(None)
        }

        fn visit_some<D: Deserializer<'de>>(
            self,
            deserializer: D,
        ) -> Result<Option<[u8; 32]>, D::Error> {
            super::serde_content_hash::deserialize(deserializer).map(Some)
        }

        fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<Option<[u8; 32]>, A::Error> {
            use super::serde_content_hash::ContentHashVisitor;
            ContentHashVisitor.visit_seq(seq).map(Some)
        }

        fn visit_str<E: de::Error>(self, s: &str) -> Result<Option<[u8; 32]>, E> {
            use super::serde_content_hash::ContentHashVisitor;
            ContentHashVisitor.visit_str(s).map(Some)
        }
    }

    /// Deserialize `Option<[u8; 32]>` from null, a byte array, or hex string.
    ///
    /// # Errors
    ///
    /// Returns a deserialization error for invalid hex or wrong length.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<[u8; 32]>, D::Error> {
        deserializer.deserialize_option(OptContentHashVisitor)
    }
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
