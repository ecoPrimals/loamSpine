// SPDX-License-Identifier: AGPL-3.0-or-later

//! Platform-abstracted IPC stream and connection dispatch.
//!
//! Phase 2 of Silicon Atheism: abstraction over gating.
//! Every `#[cfg(unix)] UnixStream::connect(path)` becomes
//! `connect_transport(&TransportEndpoint)` → `TransportStream`.
//!
//! ## Platform backends
//!
//! | Platform | UDS | TCP | Named Pipe |
//! |----------|-----|-----|------------|
//! | Unix     | `UnixStream` | `TcpStream` | — |
//! | Windows  | — (returns error) | `TcpStream` | future work |
//!
//! Reference: petalTongue `petal-tongue-platform` (`1af1a98`).

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;

use super::TransportEndpoint;
use crate::error::{IpcErrorPhase, LoamSpineError};

/// Platform-abstracted async stream for IPC and network transport.
///
/// Implements `AsyncRead + AsyncWrite`, usable anywhere a raw
/// `UnixStream` or `TcpStream` was previously hard-wired.
#[derive(Debug)]
pub enum TransportStream {
    /// Unix Domain Socket (Unix platforms only).
    #[cfg(unix)]
    Uds(tokio::net::UnixStream),

    /// TCP socket (all platforms).
    Tcp(TcpStream),
}

impl TransportStream {
    /// Split into independent read/write halves.
    ///
    /// Uses `tokio::io::split` which works on any `AsyncRead + AsyncWrite`.
    pub fn split(self) -> (tokio::io::ReadHalf<Self>, tokio::io::WriteHalf<Self>) {
        tokio::io::split(self)
    }
}

impl AsyncRead for TransportStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Uds(s) => Pin::new(s).poll_read(cx, buf),
            Self::Tcp(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TransportStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Uds(s) => Pin::new(s).poll_write(cx, buf),
            Self::Tcp(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Uds(s) => Pin::new(s).poll_flush(cx),
            Self::Tcp(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Uds(s) => Pin::new(s).poll_shutdown(cx),
            Self::Tcp(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

/// Connect to a primal or provider at the given transport endpoint.
///
/// Dispatches to platform-appropriate backend:
/// - `Uds` → `UnixStream` (Unix) or error (non-Unix)
/// - `Tcp` → `TcpStream` (all platforms)
/// - `MeshRelay` → not yet implemented (returns error)
///
/// # Errors
///
/// Returns `LoamSpineError::Ipc` on connection failure or platform unavailability.
pub async fn connect_transport(
    endpoint: &TransportEndpoint,
) -> Result<TransportStream, LoamSpineError> {
    match endpoint {
        TransportEndpoint::Uds { path } => connect_local(std::path::Path::new(path)).await,
        TransportEndpoint::Tcp { host, port } => {
            let stream = TcpStream::connect((host.as_str(), *port))
                .await
                .map_err(|e| {
                    LoamSpineError::ipc(
                        IpcErrorPhase::Connect,
                        format!("TCP connection to {host}:{port} failed: {e}"),
                    )
                })?;
            let _ = stream.set_nodelay(true);
            Ok(TransportStream::Tcp(stream))
        }
        TransportEndpoint::MeshRelay {
            peer_id,
            capability,
        } => Err(LoamSpineError::ipc(
            IpcErrorPhase::Connect,
            format!(
                "mesh relay transport not yet available \
                 (peer: {peer_id}, capability: {capability})"
            ),
        )),
    }
}

/// Connect via local IPC — UDS on Unix, error on non-Unix.
///
/// Future: Named Pipe on Windows (`\\.\pipe\ecoPrimals-{stem}`).
#[cfg(unix)]
async fn connect_local(path: &std::path::Path) -> Result<TransportStream, LoamSpineError> {
    let stream = tokio::net::UnixStream::connect(path).await.map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Connect,
            format!("UDS connection to {} failed: {e}", path.display()),
        )
    })?;
    Ok(TransportStream::Uds(stream))
}

#[cfg(not(unix))]
async fn connect_local(path: &std::path::Path) -> Result<TransportStream, LoamSpineError> {
    Err(LoamSpineError::ipc(
        IpcErrorPhase::Connect,
        format!(
            "UDS transport unavailable on this platform; \
             socket: {}. Use TCP endpoint or Named Pipe (future work).",
            path.display()
        ),
    ))
}

/// Construct a `TransportEndpoint` from a socket path.
///
/// Convenience for callers that have a `Path` (the common case for
/// provider sockets resolved from environment variables).
#[must_use]
pub fn endpoint_from_path(path: &std::path::Path) -> TransportEndpoint {
    TransportEndpoint::uds(path.to_string_lossy())
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn tcp_roundtrip() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 5];
            stream.read_exact(&mut buf).await.unwrap();
            stream.write_all(&buf).await.unwrap();
        });

        let ep = TransportEndpoint::tcp("127.0.0.1", addr.port());
        let mut stream = connect_transport(&ep).await.unwrap();
        stream.write_all(b"hello").await.unwrap();
        stream.flush().await.unwrap();

        let mut buf = [0u8; 5];
        stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"hello");

        server.await.unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn uds_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let sock = tmp.path().join("test.sock");
        let listener = tokio::net::UnixListener::bind(&sock).unwrap();

        let sock_clone = sock.clone();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 3];
            stream.read_exact(&mut buf).await.unwrap();
            stream.write_all(&buf).await.unwrap();
        });

        let ep = TransportEndpoint::uds(sock_clone.to_string_lossy());
        let mut stream = connect_transport(&ep).await.unwrap();
        stream.write_all(b"abc").await.unwrap();
        stream.flush().await.unwrap();

        let mut buf = [0u8; 3];
        stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"abc");

        server.await.unwrap();
    }

    #[tokio::test]
    async fn mesh_relay_returns_error() {
        let ep = TransportEndpoint::mesh_relay("peer", "cap");
        let result = connect_transport(&ep).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("mesh relay transport not yet available")
        );
    }

    #[tokio::test]
    async fn tcp_connect_failure() {
        let ep = TransportEndpoint::tcp("127.0.0.1", 1);
        let result = connect_transport(&ep).await;
        assert!(result.is_err());
    }

    #[test]
    fn endpoint_from_path_creates_uds() {
        let ep = endpoint_from_path(std::path::Path::new("/run/test.sock"));
        assert_eq!(ep.transport_name(), "uds");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn split_read_write() {
        let tmp = tempfile::tempdir().unwrap();
        let sock = tmp.path().join("split.sock");
        let listener = tokio::net::UnixListener::bind(&sock).unwrap();

        let sock_clone = sock.clone();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 4];
            stream.read_exact(&mut buf).await.unwrap();
            stream.write_all(b"pong").await.unwrap();
        });

        let ep = TransportEndpoint::uds(sock_clone.to_string_lossy());
        let stream = connect_transport(&ep).await.unwrap();
        let (mut reader, mut writer) = stream.split();
        writer.write_all(b"ping").await.unwrap();
        writer.flush().await.unwrap();

        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"pong");

        server.await.unwrap();
    }
}
