// SPDX-License-Identifier: AGPL-3.0-only

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
pub type PeerId = String;

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
    ///
    /// Cannot be `const fn` because `ByteBuffer::new()` (Bytes::new) is not const.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn empty() -> Self {
        Self(ByteBuffer::new())
    }

    /// Check if the signature is empty.
    ///
    /// Cannot be `const fn` because `Bytes::is_empty()` is not const.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
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
        let bytes: Vec<u8> = serde::Deserialize::deserialize(deserializer)?;
        Ok(Self::from_vec(bytes))
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
            s.push(HEX_CHARS[(b >> 4) as usize] as char);
            s.push(HEX_CHARS[(b & 0xf) as usize] as char);
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
mod tests {
    use super::*;

    #[test]
    fn did_display() {
        let did = Did::new("did:key:z6MkTest");
        assert_eq!(did.to_string(), "did:key:z6MkTest");
        assert_eq!(did.as_str(), "did:key:z6MkTest");
    }

    #[test]
    fn did_from_string() {
        let did: Did = "did:key:z6MkTest".into();
        assert_eq!(did.as_str(), "did:key:z6MkTest");
    }

    #[test]
    fn signature_empty() {
        let sig = Signature::empty();
        assert!(sig.is_empty());

        let sig = Signature::from_vec(vec![1, 2, 3]);
        assert!(!sig.is_empty());
        assert_eq!(sig.as_bytes(), &[1, 2, 3]);
    }

    #[test]
    fn timestamp_now() {
        let ts = Timestamp::now();
        assert!(ts.as_nanos() > 0);
        assert!(ts.as_secs() > 0);
    }

    #[test]
    fn timestamp_conversion() {
        let ts = Timestamp::from_nanos(1_000_000_000);
        assert_eq!(ts.as_secs(), 1);
        assert_eq!(ts.as_nanos(), 1_000_000_000);
    }

    #[test]
    fn hash_bytes_works() {
        let hash = hash_bytes(b"hello");
        assert_eq!(hash.len(), 32);

        // Same input should give same hash
        let hash2 = hash_bytes(b"hello");
        assert_eq!(hash, hash2);

        // Different input should give different hash
        let hash3 = hash_bytes(b"world");
        assert_ne!(hash, hash3);
    }

    #[test]
    fn payload_ref_builder() {
        let hash = hash_bytes(b"test");
        let payload = PayloadRef::new(hash, KB).with_mime_type("application/json");

        assert_eq!(payload.hash, hash);
        assert_eq!(payload.size, KB);
        assert_eq!(payload.mime_type, Some("application/json".to_string()));
    }

    #[test]
    fn format_hash_short_works() {
        let hash = hash_bytes(b"test");
        let short = format_hash_short(&hash);
        assert_eq!(short.len(), 16); // 8 bytes = 16 hex chars
    }

    #[test]
    fn byte_buffer_from_vec() {
        let vec = vec![1u8, 2, 3, 4, 5];
        let buffer: ByteBuffer = vec.into_byte_buffer();
        assert_eq!(&buffer[..], &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn byte_buffer_from_slice() {
        let data: &[u8] = &[1, 2, 3];
        let buffer: ByteBuffer = data.into_byte_buffer();
        assert_eq!(&buffer[..], &[1, 2, 3]);
    }

    #[test]
    fn byte_buffer_zero_copy_slice() {
        let buffer: ByteBuffer = ByteBuffer::from_static(b"hello world");
        let slice = buffer.slice(0..5);
        assert_eq!(&slice[..], b"hello");
        // Both share the same underlying data (zero-copy)
        assert_eq!(buffer.len(), 11);
        assert_eq!(slice.len(), 5);
    }
}
