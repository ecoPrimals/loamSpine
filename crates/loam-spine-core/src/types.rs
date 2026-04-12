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

impl Did {
    /// Create a new DID.
    #[must_use]
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self(value.into())
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
        Self(bytes.into_byte_buffer())
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
/// allocates, but binary codecs (bincode, postcard) use `visit_byte_buf`
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
            .map(|d| d.as_nanos().try_into().unwrap_or(u64::MAX))
            .unwrap_or(0);
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

/// Extension trait for converting between `Vec<u8>` and `ByteBuffer`.
pub trait IntoByteBuffer {
    /// Convert to a zero-copy byte buffer.
    fn into_byte_buffer(self) -> ByteBuffer;
}

impl IntoByteBuffer for Vec<u8> {
    fn into_byte_buffer(self) -> ByteBuffer {
        ByteBuffer::from(self)
    }
}

impl IntoByteBuffer for &[u8] {
    fn into_byte_buffer(self) -> ByteBuffer {
        ByteBuffer::copy_from_slice(self)
    }
}

impl IntoByteBuffer for &str {
    fn into_byte_buffer(self) -> ByteBuffer {
        ByteBuffer::copy_from_slice(self.as_bytes())
    }
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
