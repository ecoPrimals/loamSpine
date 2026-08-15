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
    let buffer: ByteBuffer = ByteBuffer::from(vec);
    assert_eq!(&buffer[..], &[1, 2, 3, 4, 5]);
}

#[test]
fn byte_buffer_from_slice() {
    let data: &[u8] = &[1, 2, 3];
    let buffer: ByteBuffer = ByteBuffer::copy_from_slice(data);
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
    let buffer: ByteBuffer = ByteBuffer::copy_from_slice(b"hello");
    assert_eq!(&buffer[..], b"hello");
}

// ============================================================================
// serde_content_hash / serde_opt_content_hash tests
// ============================================================================

#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod serde_hash_tests {
    use super::*;

    #[test]
    fn content_hash_from_byte_array() {
        let arr: [u8; 32] = [0xAB; 32];
        let json = serde_json::to_string(&arr).unwrap();
        let parsed: ContentHash =
            serde_content_hash::deserialize(&mut serde_json::Deserializer::from_str(&json))
                .unwrap();
        assert_eq!(parsed, arr);
    }

    #[test]
    fn content_hash_from_hex_string() {
        let hex = "\"".to_string() + &"ab".repeat(32) + "\"";
        let parsed: ContentHash =
            serde_content_hash::deserialize(&mut serde_json::Deserializer::from_str(&hex)).unwrap();
        assert_eq!(parsed, [0xAB; 32]);
    }

    #[test]
    fn content_hash_from_0x_hex_string() {
        let hex = "\"0x".to_string() + &"01".repeat(32) + "\"";
        let parsed: ContentHash =
            serde_content_hash::deserialize(&mut serde_json::Deserializer::from_str(&hex)).unwrap();
        assert_eq!(parsed, [0x01; 32]);
    }

    #[test]
    fn content_hash_rejects_wrong_length_hex() {
        let hex = "\"abcd\"";
        let result: Result<ContentHash, _> =
            serde_content_hash::deserialize(&mut serde_json::Deserializer::from_str(hex));
        assert!(result.is_err());
    }

    #[test]
    fn content_hash_rejects_invalid_hex_chars() {
        let hex = "\"".to_string() + &"zz".repeat(32) + "\"";
        let result: Result<ContentHash, _> =
            serde_content_hash::deserialize(&mut serde_json::Deserializer::from_str(&hex));
        assert!(result.is_err());
    }

    #[test]
    fn opt_content_hash_from_null() {
        let parsed: Option<ContentHash> =
            serde_opt_content_hash::deserialize(&mut serde_json::Deserializer::from_str("null"))
                .unwrap();
        assert!(parsed.is_none());
    }

    #[test]
    fn opt_content_hash_from_hex_string() {
        let hex = "\"".to_string() + &"ff".repeat(32) + "\"";
        let parsed: Option<ContentHash> =
            serde_opt_content_hash::deserialize(&mut serde_json::Deserializer::from_str(&hex))
                .unwrap();
        assert_eq!(parsed, Some([0xFF; 32]));
    }

    #[test]
    fn opt_content_hash_from_byte_array() {
        let arr: [u8; 32] = [0x42; 32];
        let json = serde_json::to_string(&arr).unwrap();
        let parsed: Option<ContentHash> =
            serde_opt_content_hash::deserialize(&mut serde_json::Deserializer::from_str(&json))
                .unwrap();
        assert_eq!(parsed, Some(arr));
    }

    #[test]
    fn content_hash_mixed_case_hex() {
        let hex = "\"".to_string() + &"Ab".repeat(32) + "\"";
        let parsed: ContentHash =
            serde_content_hash::deserialize(&mut serde_json::Deserializer::from_str(&hex)).unwrap();
        assert_eq!(parsed, [0xAB; 32]);
    }
}

#[test]
fn peer_id_create_and_display() {
    let peer = PeerId::new("peer-001");
    assert_eq!(peer.as_str(), "peer-001");
    assert_eq!(peer.to_string(), "peer-001");
}

#[test]
fn peer_id_from_str_and_string() {
    let from_str: PeerId = PeerId::from("peer-str");
    let from_string: PeerId = PeerId::from("peer-str".to_string());
    assert_eq!(from_str, from_string);
}

#[test]
fn peer_id_equality_with_str() {
    let peer = PeerId::new("peer-eq");
    assert!(peer == "peer-eq");
}

#[test]
fn peer_id_borrow_str() {
    use std::borrow::Borrow;
    let peer = PeerId::new("peer-borrow");
    let s: &str = peer.borrow();
    assert_eq!(s, "peer-borrow");
}

#[test]
fn peer_id_hash_works() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(PeerId::new("a"));
    set.insert(PeerId::new("b"));
    set.insert(PeerId::new("a"));
    assert_eq!(set.len(), 2);
}

#[test]
fn peer_id_serde_roundtrip() {
    let peer = PeerId::new("peer-serde");
    let json = serde_json::to_string(&peer).expect("serialize");
    let back: PeerId = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(peer, back);
}

#[test]
fn did_anonymous() {
    let anon = Did::anonymous();
    assert!(anon.is_anonymous());
    assert_eq!(anon.as_str(), "did:primal:anonymous");
}

#[test]
fn did_not_anonymous() {
    let did = Did::new("did:key:z6MkReal");
    assert!(!did.is_anonymous());
}

#[test]
fn did_hash_works() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Did::new("did:key:a"));
    set.insert(Did::new("did:key:b"));
    set.insert(Did::new("did:key:a"));
    assert_eq!(set.len(), 2);
}

#[test]
fn signature_to_base64() {
    use base64::Engine;
    let sig = Signature::from_vec(vec![1, 2, 3]);
    let b64 = sig.to_base64();
    assert!(!b64.is_empty());
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&b64)
        .expect("decode");
    assert_eq!(decoded, vec![1, 2, 3]);
}

#[test]
fn payload_ref_without_mime() {
    let payload = PayloadRef::new([0u8; 32], 512);
    assert!(payload.mime_type.is_none());
}

#[test]
fn size_constants() {
    assert_eq!(KB, 1024);
    assert_eq!(MB, 1024 * 1024);
    assert_eq!(GB, 1024 * 1024 * 1024);
}

#[test]
fn timestamp_ordering() {
    let t1 = Timestamp::from_nanos(100);
    let t2 = Timestamp::from_nanos(200);
    assert!(t1 < t2);
    assert!(t2 > t1);
    assert_eq!(t1, Timestamp::from_nanos(100));
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
            let buf: ByteBuffer = ByteBuffer::from(data.clone());
            prop_assert_eq!(&buf[..], &data[..]);
        }
    }
}

#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod hex_error_display_tests {
    use super::serde_content_hash::{HexError, parse_hex};

    #[test]
    fn bad_length_display() {
        assert_eq!(
            HexError::BadLength(4).to_string(),
            "expected 64 hex chars, got 4"
        );
    }

    #[test]
    fn invalid_byte_display() {
        let source = u8::from_str_radix("zz", 16).unwrap_err();
        let expected = format!("invalid hex at byte 3: {source}");
        let msg = HexError::InvalidByte { index: 3, source }.to_string();
        assert_eq!(msg, expected);
    }

    #[test]
    fn parse_hex_bad_length_message() {
        let err = parse_hex("abcd").unwrap_err();
        assert_eq!(err.to_string(), "expected 64 hex chars, got 4");
    }

    #[test]
    fn parse_hex_invalid_chars_message() {
        let err = parse_hex(&"zz".repeat(32)).unwrap_err();
        assert!(err.to_string().starts_with("invalid hex at byte 0:"));
    }

    #[test]
    fn parse_hex_rejects_0x_prefix_wrong_length() {
        let err = parse_hex("0xabcd").unwrap_err();
        assert_eq!(err.to_string(), "expected 64 hex chars, got 4");
    }
}

#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod binary_serde_tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn signature_msgpack_roundtrip() {
        let sig = Signature::from_vec(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        let packed = rmp_serde::to_vec(&sig).unwrap();
        let back: Signature = rmp_serde::from_slice(&packed).unwrap();
        assert_eq!(sig.as_bytes(), back.as_bytes());
    }

    #[test]
    fn signature_deserialize_from_bytes_visitor() {
        let bytes = [0xDEu8, 0xAD, 0xBE, 0xEF];
        let de = serde::de::value::BytesDeserializer::<serde::de::value::Error>::new(&bytes);
        let sig = Signature::deserialize(de).unwrap();
        assert_eq!(sig.as_bytes(), &bytes);
    }

    #[test]
    fn content_hash_deserialize_from_bytes() {
        let bytes = [0xABu8; 32];
        let parsed: ContentHash =
            serde_content_hash::deserialize(serde::de::value::BytesDeserializer::<
                serde::de::value::Error,
            >::new(&bytes))
            .unwrap();
        assert_eq!(parsed, bytes);
    }

    #[test]
    fn content_hash_rejects_wrong_length_bytes() {
        let bytes = [0x01u8; 16];
        let result: Result<ContentHash, _> =
            serde_content_hash::deserialize(serde::de::value::BytesDeserializer::<
                serde::de::value::Error,
            >::new(&bytes));
        assert!(result.is_err());
    }

    #[test]
    fn opt_content_hash_from_unit() {
        let parsed: Option<ContentHash> =
            serde_opt_content_hash::deserialize(serde::de::value::UnitDeserializer::<
                serde::de::value::Error,
            >::new())
            .unwrap();
        assert!(parsed.is_none());
    }

    #[test]
    fn opt_content_hash_from_str_direct() {
        let hex = "ab".repeat(32);
        let parsed: Option<ContentHash> =
            serde_opt_content_hash::deserialize(serde::de::value::StrDeserializer::<
                serde::de::value::Error,
            >::new(&hex))
            .unwrap();
        assert_eq!(parsed, Some([0xAB; 32]));
    }

    #[test]
    fn opt_content_hash_from_seq_direct() {
        let arr = [0x42u8; 32];
        let parsed: Option<ContentHash> =
            serde_opt_content_hash::deserialize(serde::de::value::SeqDeserializer::<
                _,
                serde::de::value::Error,
            >::new(arr.into_iter()))
            .unwrap();
        assert_eq!(parsed, Some(arr));
    }

    #[test]
    fn payload_ref_json_roundtrip() {
        let pr = PayloadRef::new(hash_bytes(b"x"), 1024).with_mime_type("text/plain");
        let json = serde_json::to_string(&pr).unwrap();
        let back: PayloadRef = serde_json::from_str(&json).unwrap();
        assert_eq!(pr, back);
    }

    #[test]
    fn did_json_roundtrip_direct() {
        let did = Did::new("did:key:z6MkSerde");
        let json = serde_json::to_string(&did).unwrap();
        let back: Did = serde_json::from_str(&json).unwrap();
        assert_eq!(did, back);
    }

    #[test]
    fn opt_content_hash_msgpack_none() {
        #[derive(Serialize, Deserialize)]
        struct Holder {
            #[serde(default, deserialize_with = "serde_opt_content_hash::deserialize")]
            hash: Option<ContentHash>,
        }
        let packed = rmp_serde::to_vec(&Holder { hash: None }).unwrap();
        let back: Holder = rmp_serde::from_slice(&packed).unwrap();
        assert!(back.hash.is_none());
    }

    #[test]
    fn opt_content_hash_msgpack_nil_direct() {
        let nil = [0xc0];
        let parsed: Option<ContentHash> =
            serde_opt_content_hash::deserialize(&mut rmp_serde::Deserializer::new(&nil[..]))
                .unwrap();
        assert!(parsed.is_none());
    }

    #[test]
    fn content_hash_rejects_short_byte_array() {
        let result: Result<ContentHash, _> =
            serde_content_hash::deserialize(&mut serde_json::Deserializer::from_str("[1, 2, 3]"));
        assert!(result.is_err());
    }

    #[test]
    fn opt_content_hash_rejects_short_seq() {
        let result: Result<Option<ContentHash>, _> =
            serde_opt_content_hash::deserialize(serde::de::value::SeqDeserializer::<
                _,
                serde::de::value::Error,
            >::new([1u8, 2, 3].into_iter()));
        assert!(result.is_err());
    }

    #[test]
    fn signature_rejects_invalid_json_type() {
        let result: Result<Signature, _> = serde_json::from_str("not-bytes");
        assert!(result.is_err());
    }

    #[test]
    fn signature_empty_json_array() {
        let sig: Signature = serde_json::from_str("[]").unwrap();
        assert!(sig.is_empty());
    }
}

#[test]
fn peer_id_debug_contains_id() {
    let peer = PeerId::new("test-peer");
    let debug = format!("{peer:?}");
    assert!(debug.contains("test-peer"));
}

#[test]
fn timestamp_copy_semantics() {
    let t1 = Timestamp::from_nanos(999);
    let t2 = t1;
    assert_eq!(t1, t2);
}
