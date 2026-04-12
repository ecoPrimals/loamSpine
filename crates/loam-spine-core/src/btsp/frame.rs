// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP length-prefixed frame I/O.
//!
//! All BTSP wire frames use a 4-byte big-endian length prefix per
//! `BTSP_PROTOCOL_STANDARD.md` §Wire Framing.

use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::{IpcErrorPhase, LoamSpineError};

/// Maximum BTSP frame size (16 MiB) per `BTSP_PROTOCOL_STANDARD.md`.
pub(crate) const MAX_FRAME_SIZE: u32 = 0x0100_0000;

/// Read a length-prefixed BTSP frame from the stream.
///
/// Returns `Bytes` for zero-copy downstream processing.
///
/// # Errors
///
/// Returns `LoamSpineError::Ipc` if the frame exceeds the maximum size or
/// the stream is closed prematurely.
pub async fn read_frame<R: AsyncReadExt + Unpin>(reader: &mut R) -> Result<Bytes, LoamSpineError> {
    let len = reader.read_u32().await.map_err(|e| {
        LoamSpineError::ipc(IpcErrorPhase::Read, format!("BTSP frame length read: {e}"))
    })?;

    if len > MAX_FRAME_SIZE {
        return Err(LoamSpineError::ipc(
            IpcErrorPhase::Read,
            format!("BTSP frame too large: {len} bytes (max {MAX_FRAME_SIZE})"),
        ));
    }

    let mut buf = BytesMut::zeroed(len as usize);
    reader.read_exact(&mut buf).await.map_err(|e| {
        LoamSpineError::ipc(IpcErrorPhase::Read, format!("BTSP frame body read: {e}"))
    })?;

    Ok(buf.freeze())
}

/// Write a length-prefixed BTSP frame to the stream.
///
/// # Errors
///
/// Returns `LoamSpineError::Ipc` on write failure.
pub async fn write_frame<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    data: &[u8],
) -> Result<(), LoamSpineError> {
    let len = u32::try_from(data.len()).map_err(|_| {
        LoamSpineError::ipc(
            IpcErrorPhase::Write,
            format!("BTSP frame too large: {} bytes", data.len()),
        )
    })?;

    if len > MAX_FRAME_SIZE {
        return Err(LoamSpineError::ipc(
            IpcErrorPhase::Write,
            format!("BTSP frame too large: {len} bytes (max {MAX_FRAME_SIZE})"),
        ));
    }

    writer.write_u32(len).await.map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Write,
            format!("BTSP frame length write: {e}"),
        )
    })?;
    writer.write_all(data).await.map_err(|e| {
        LoamSpineError::ipc(IpcErrorPhase::Write, format!("BTSP frame body write: {e}"))
    })?;
    writer
        .flush()
        .await
        .map_err(|e| LoamSpineError::ipc(IpcErrorPhase::Write, format!("BTSP frame flush: {e}")))?;

    Ok(())
}

/// Serialize a BTSP wire message to JSON bytes.
pub(crate) fn serialize_btsp_msg<T: serde::Serialize>(
    msg: &T,
    label: &str,
) -> Result<Vec<u8>, LoamSpineError> {
    serde_json::to_vec(msg).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Serialization,
            format!("BTSP {label} serialize: {e}"),
        )
    })
}

/// Deserialize a BTSP wire message from JSON bytes.
pub(crate) fn deserialize_btsp_msg<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    label: &str,
) -> Result<T, LoamSpineError> {
    serde_json::from_slice(bytes).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::InvalidJson,
            format!("BTSP {label} parse: {e}"),
        )
    })
}
