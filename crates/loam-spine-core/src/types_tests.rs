// SPDX-License-Identifier: AGPL-3.0-or-later

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

#[test]
fn did_from_owned_string() {
    let did: Did = String::from("did:key:z6MkOwned").into();
    assert_eq!(did.as_str(), "did:key:z6MkOwned");
}

#[test]
fn signature_default_is_empty() {
    let sig = Signature::default();
    assert!(sig.is_empty());
}

#[test]
fn signature_as_byte_buffer() {
    let sig = Signature::from_vec(vec![0xAB, 0xCD]);
    let buf = sig.as_byte_buffer();
    assert_eq!(&buf[..], &[0xAB, 0xCD]);
}

#[test]
fn timestamp_default_is_now() {
    let ts = Timestamp::default();
    assert!(ts.as_nanos() > 0);
}

#[test]
fn timestamp_display() {
    let ts = Timestamp::from_nanos(42_000);
    assert_eq!(ts.to_string(), "42000ns");
}

#[test]
fn byte_buffer_from_str_slice() {
    let buffer: ByteBuffer = "hello".into_byte_buffer();
    assert_eq!(&buffer[..], b"hello");
}

#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod proptest_roundtrips {
    use crate::types::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn did_serde_roundtrip(s in "[a-z0-9:._-]{1,64}") {
            let did = Did::new(s.as_str());
            let json = serde_json::to_string(&did).unwrap();
            let back: Did = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(did.as_str(), back.as_str());
        }

        #[test]
        fn did_display_roundtrip(s in "[a-z0-9:._-]{1,64}") {
            let did = Did::new(s.as_str());
            let displayed = did.to_string();
            let back = Did::new(displayed.as_str());
            prop_assert_eq!(did, back);
        }

        #[test]
        fn did_clone_equality(s in "[a-z0-9:._-]{1,64}") {
            let did = Did::new(s.as_str());
            let cloned = did.clone();
            prop_assert_eq!(did, cloned);
        }

        #[test]
        fn spine_id_serde_roundtrip(() in Just(())) {
            let id: SpineId = uuid::Uuid::now_v7();
            let json = serde_json::to_string(&id).unwrap();
            let back: SpineId = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(id, back);
        }

        #[test]
        fn content_hash_roundtrip(data in prop::collection::vec(any::<u8>(), 0..256)) {
            let hash = hash_bytes(&data);
            let json = serde_json::to_string(&hash).unwrap();
            let back: ContentHash = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(hash, back);
        }

        #[test]
        fn signature_serde_roundtrip(data in prop::collection::vec(any::<u8>(), 0..128)) {
            let sig = Signature::new(bytes::Bytes::from(data));
            let json = serde_json::to_string(&sig).unwrap();
            let back: Signature = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(sig.as_bytes(), back.as_bytes());
        }

        #[test]
        fn byte_buffer_roundtrip(data in prop::collection::vec(any::<u8>(), 0..512)) {
            let buf: ByteBuffer = data.clone().into_byte_buffer();
            prop_assert_eq!(&buf[..], &data[..]);
        }
    }
}
