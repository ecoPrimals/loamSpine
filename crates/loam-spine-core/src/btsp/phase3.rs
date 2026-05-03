// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP Phase 3 — Encrypted post-handshake channel.
//!
//! After Phase 2 authentication, Phase 3 negotiates an encrypted transport.
//! Session keys are derived from the Tower-provided `handshake_key` via
//! HKDF-SHA256, then used for ChaCha20-Poly1305 AEAD framing.
//!
//! # Key Acquisition (Pattern B — Tower-Provided)
//!
//! loamSpine receives `session_key` from `BearDog`'s `btsp.session.verify`
//! response and stores it as the `handshake_key`. No local derivation from
//! `FAMILY_SEED` — asymmetric and key derivation authority stays with Tower.
//!
//! # Session Key Derivation
//!
//! ```text
//! session_key = HKDF-SHA256(
//!     ikm  = handshake_key,
//!     salt = client_nonce || server_nonce,
//!     info = "btsp-session-v1-{direction}"
//! )
//! ```
//!
//! Client and server derive mirrored keys: the client's encrypt key is the
//! server's decrypt key and vice versa.
//!
//! # Wire Format
//!
//! Each encrypted frame:
//! ```text
//! [4 bytes: length (big-endian u32)] [12 bytes: nonce] [ciphertext + 16-byte Poly1305 tag]
//! ```

use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{IpcErrorPhase, LoamSpineError};

/// Cipher suite wire name for ChaCha20-Poly1305 AEAD.
pub const CIPHER_CHACHA20_POLY1305: &str = "chacha20-poly1305";

/// Cipher suite wire name for plaintext (null cipher).
pub const CIPHER_NULL: &str = "null";

/// Derived session keys for the encrypted channel.
///
/// Both sides derive the same keys from the handshake key + nonces.
/// Keys are zeroed on drop to prevent lingering in memory.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SessionKeys {
    encrypt_key: [u8; 32],
    decrypt_key: [u8; 32],
}

impl std::fmt::Debug for SessionKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionKeys")
            .field("encrypt_key", &"[redacted]")
            .field("decrypt_key", &"[redacted]")
            .finish()
    }
}

impl SessionKeys {
    /// Derive session keys from the Phase 2 handshake key and both nonces.
    ///
    /// The client and server derive mirrored keys: the client's encrypt key
    /// is the server's decrypt key, and vice versa.
    ///
    /// # Errors
    ///
    /// Returns [`LoamSpineError`] if HKDF expansion fails.
    pub fn derive(
        handshake_key: &[u8; 32],
        client_nonce: &[u8],
        server_nonce: &[u8],
        is_server: bool,
    ) -> Result<Self, LoamSpineError> {
        use hkdf::Hkdf;
        use sha2::Sha256;

        let mut salt = Vec::with_capacity(client_nonce.len() + server_nonce.len());
        salt.extend_from_slice(client_nonce);
        salt.extend_from_slice(server_nonce);

        let hk = Hkdf::<Sha256>::new(Some(&salt), handshake_key);

        let mut client_to_server = [0u8; 32];
        hk.expand(b"btsp-session-v1-c2s", &mut client_to_server)
            .map_err(|e| {
                LoamSpineError::ipc(IpcErrorPhase::Read, format!("BTSP Phase 3 HKDF c2s: {e}"))
            })?;

        let mut server_to_client = [0u8; 32];
        hk.expand(b"btsp-session-v1-s2c", &mut server_to_client)
            .map_err(|e| {
                LoamSpineError::ipc(IpcErrorPhase::Read, format!("BTSP Phase 3 HKDF s2c: {e}"))
            })?;

        if is_server {
            Ok(Self {
                encrypt_key: server_to_client,
                decrypt_key: client_to_server,
            })
        } else {
            Ok(Self {
                encrypt_key: client_to_server,
                decrypt_key: server_to_client,
            })
        }
    }

    /// Encrypt a plaintext message for transmission.
    ///
    /// Returns `nonce || ciphertext` (12 + plaintext.len() + 16 bytes).
    ///
    /// # Errors
    ///
    /// Returns [`LoamSpineError`] if encryption or nonce generation fails.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, LoamSpineError> {
        use chacha20poly1305::aead::{Aead, KeyInit};
        use chacha20poly1305::{ChaCha20Poly1305, Nonce};

        let cipher = ChaCha20Poly1305::new((&self.encrypt_key).into());

        let mut nonce_bytes = [0u8; 12];
        getrandom::fill(&mut nonce_bytes).map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::Write,
                format!("BTSP Phase 3 nonce generation: {e}"),
            )
        })?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext).map_err(|e| {
            LoamSpineError::ipc(IpcErrorPhase::Write, format!("BTSP Phase 3 encrypt: {e}"))
        })?;

        let mut frame = Vec::with_capacity(12 + ciphertext.len());
        frame.extend_from_slice(&nonce_bytes);
        frame.extend_from_slice(&ciphertext);
        Ok(frame)
    }

    /// Decrypt a received frame (`nonce || ciphertext`).
    ///
    /// # Errors
    ///
    /// Returns [`LoamSpineError`] if decryption fails (tampered data, wrong key).
    pub fn decrypt(&self, frame: &[u8]) -> Result<Vec<u8>, LoamSpineError> {
        use chacha20poly1305::aead::{Aead, KeyInit};
        use chacha20poly1305::{ChaCha20Poly1305, Nonce};

        const MIN_FRAME: usize = 12 + 16; // nonce + Poly1305 tag
        if frame.len() < MIN_FRAME {
            return Err(LoamSpineError::ipc(
                IpcErrorPhase::Read,
                format!(
                    "BTSP Phase 3 frame too short: {} bytes (need >= {MIN_FRAME})",
                    frame.len()
                ),
            ));
        }

        let (nonce_bytes, ciphertext) = frame.split_at(12);
        let cipher = ChaCha20Poly1305::new((&self.decrypt_key).into());
        let nonce = Nonce::from_slice(nonce_bytes);

        cipher.decrypt(nonce, ciphertext).map_err(|e| {
            LoamSpineError::ipc(IpcErrorPhase::Read, format!("BTSP Phase 3 decrypt: {e}"))
        })
    }
}

/// Generate a random 32-byte nonce for Phase 3 negotiation.
///
/// # Errors
///
/// Returns [`LoamSpineError`] if random generation fails.
pub fn generate_nonce() -> Result<[u8; 32], LoamSpineError> {
    let mut nonce = [0u8; 32];
    getrandom::fill(&mut nonce).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Read,
            format!("BTSP Phase 3 nonce generation: {e}"),
        )
    })?;
    Ok(nonce)
}

/// Read one encrypted frame from a length-prefixed stream.
///
/// Wire format: `[4B length BE u32][payload]` where payload = `[12B nonce][ciphertext + tag]`.
///
/// # Errors
///
/// Returns [`LoamSpineError`] on I/O error or if the frame exceeds `MAX_FRAME_SIZE`.
pub async fn read_encrypted_frame<R: tokio::io::AsyncReadExt + Unpin>(
    reader: &mut R,
    keys: &SessionKeys,
) -> Result<Vec<u8>, LoamSpineError> {
    let frame = super::frame::read_frame(reader).await?;
    keys.decrypt(&frame)
}

/// Write one encrypted frame to a length-prefixed stream.
///
/// # Errors
///
/// Returns [`LoamSpineError`] on encryption or I/O error.
pub async fn write_encrypted_frame<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    keys: &SessionKeys,
    plaintext: &[u8],
) -> Result<(), LoamSpineError> {
    let encrypted = keys.encrypt(plaintext)?;
    super::frame::write_frame(writer, &encrypted).await
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "tests use expect for concise assertions"
)]
mod tests {
    use super::*;

    #[test]
    fn derive_deterministic() {
        let hk = [0xAA; 32];
        let cn = [0xBB; 32];
        let sn = [0xCC; 32];

        let k1 = SessionKeys::derive(&hk, &cn, &sn, true).expect("k1");
        let k2 = SessionKeys::derive(&hk, &cn, &sn, true).expect("k2");
        assert_eq!(k1.encrypt_key, k2.encrypt_key);
        assert_eq!(k1.decrypt_key, k2.decrypt_key);
    }

    #[test]
    fn client_server_mirror() {
        let hk = [0xAA; 32];
        let cn = [0xBB; 32];
        let sn = [0xCC; 32];

        let server_keys = SessionKeys::derive(&hk, &cn, &sn, true).expect("server");
        let client_keys = SessionKeys::derive(&hk, &cn, &sn, false).expect("client");

        assert_eq!(
            client_keys.encrypt_key, server_keys.decrypt_key,
            "client encrypt = server decrypt"
        );
        assert_eq!(
            client_keys.decrypt_key, server_keys.encrypt_key,
            "client decrypt = server encrypt"
        );
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
        let hk = [0x42; 32];
        let cn = [0x01; 32];
        let sn = [0x02; 32];

        let client = SessionKeys::derive(&hk, &cn, &sn, false).expect("client");
        let server = SessionKeys::derive(&hk, &cn, &sn, true).expect("server");

        let plaintext = b"hello from BTSP Phase 3";
        let frame = client.encrypt(plaintext).expect("encrypt");
        let decrypted = server.decrypt(&frame).expect("decrypt");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_decrypt_large_payload() {
        let hk = [0x42; 32];
        let cn = [0x01; 32];
        let sn = [0x02; 32];

        let client = SessionKeys::derive(&hk, &cn, &sn, false).expect("client");
        let server = SessionKeys::derive(&hk, &cn, &sn, true).expect("server");

        let plaintext = vec![0xAB; 64 * 1024];
        let frame = client.encrypt(&plaintext).expect("encrypt");
        let decrypted = server.decrypt(&frame).expect("decrypt");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn server_encrypt_client_decrypt() {
        let hk = [0x42; 32];
        let cn = [0x01; 32];
        let sn = [0x02; 32];

        let server = SessionKeys::derive(&hk, &cn, &sn, true).expect("server");
        let client = SessionKeys::derive(&hk, &cn, &sn, false).expect("client");

        let plaintext = b"response from server";
        let frame = server.encrypt(plaintext).expect("encrypt");
        let decrypted = client.decrypt(&frame).expect("decrypt");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_rejects_tampered_frame() {
        let hk = [0x42; 32];
        let cn = [0x01; 32];
        let sn = [0x02; 32];

        let client = SessionKeys::derive(&hk, &cn, &sn, false).expect("client");
        let server = SessionKeys::derive(&hk, &cn, &sn, true).expect("server");

        let mut frame = client.encrypt(b"authentic data").expect("encrypt");
        let last = frame.len() - 1;
        frame[last] ^= 0xFF;
        assert!(server.decrypt(&frame).is_err());
    }

    #[test]
    fn decrypt_rejects_short_frame() {
        let server =
            SessionKeys::derive(&[0x42; 32], &[0x01; 32], &[0x02; 32], true).expect("server");
        assert!(server.decrypt(&[0u8; 10]).is_err());
    }

    #[test]
    fn decrypt_rejects_wrong_key() {
        let cn = [0x01; 32];
        let sn = [0x02; 32];

        let client = SessionKeys::derive(&[0xAA; 32], &cn, &sn, false).expect("client");
        let wrong_server = SessionKeys::derive(&[0xBB; 32], &cn, &sn, true).expect("wrong_server");

        let frame = client.encrypt(b"secret").expect("encrypt");
        assert!(wrong_server.decrypt(&frame).is_err());
    }

    #[test]
    fn generate_nonce_produces_32_bytes() {
        let nonce = generate_nonce().expect("nonce");
        assert_eq!(nonce.len(), 32);
    }

    #[test]
    fn generate_nonce_is_random() {
        let n1 = generate_nonce().expect("n1");
        let n2 = generate_nonce().expect("n2");
        assert_ne!(n1, n2, "nonces should be different");
    }

    #[test]
    fn debug_impl_redacts_keys() {
        let keys = SessionKeys::derive(&[0x42; 32], &[0x01; 32], &[0x02; 32], true).expect("keys");
        let debug = format!("{keys:?}");
        assert!(debug.contains("[redacted]"));
        assert!(!debug.contains("42"));
    }

    #[tokio::test]
    async fn encrypted_frame_round_trip() {
        let hk = [0x42; 32];
        let cn = [0x01; 32];
        let sn = [0x02; 32];

        let server = SessionKeys::derive(&hk, &cn, &sn, true).expect("server");
        let client = SessionKeys::derive(&hk, &cn, &sn, false).expect("client");

        let plaintext = b"JSON-RPC over encrypted BTSP";

        let mut buf = Vec::new();
        write_encrypted_frame(&mut buf, &client, plaintext)
            .await
            .expect("write");

        let mut cursor = std::io::Cursor::new(buf);
        let decrypted = read_encrypted_frame(&mut cursor, &server)
            .await
            .expect("read");

        assert_eq!(decrypted, plaintext);
    }
}
