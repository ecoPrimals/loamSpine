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

/// Platform-abstracted listener for IPC and network transport.
///
/// Accepts incoming connections and wraps them as [`TransportStream`].
/// The `#[cfg(unix)]` boundary lives here — callers see only
/// `TransportListener` and `TransportStream`.
#[derive(Debug)]
pub enum TransportListener {
    /// Unix Domain Socket listener (Unix platforms only).
    #[cfg(unix)]
    Uds(tokio::net::UnixListener),

    /// TCP listener (all platforms).
    Tcp(tokio::net::TcpListener),
}

impl TransportListener {
    /// Accept the next incoming connection.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` on accept failure.
    pub async fn accept(&self) -> io::Result<TransportStream> {
        match self {
            #[cfg(unix)]
            Self::Uds(l) => {
                let (stream, _) = l.accept().await?;
                Ok(TransportStream::Uds(stream))
            }
            Self::Tcp(l) => {
                let (stream, _) = l.accept().await?;
                if let Err(e) = stream.set_nodelay(true) {
                    tracing::trace!("TCP set_nodelay on accepted stream (non-fatal): {e}");
                }
                Ok(TransportStream::Tcp(stream))
            }
        }
    }

    /// Bind a listener on the given transport endpoint.
    ///
    /// # Errors
    ///
    /// Returns `LoamSpineError::Ipc` on bind failure or platform unavailability.
    pub async fn bind(endpoint: &TransportEndpoint) -> Result<Self, LoamSpineError> {
        match endpoint {
            TransportEndpoint::Uds { path } => bind_local(std::path::Path::new(path)).await,
            TransportEndpoint::Tcp { host, port } => {
                let addr = format!("{host}:{port}");
                let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
                    LoamSpineError::ipc(
                        IpcErrorPhase::Connect,
                        format!("TCP bind at {addr} failed: {e}"),
                    )
                })?;
                Ok(Self::Tcp(listener))
            }
            TransportEndpoint::MeshRelay { .. } => Err(LoamSpineError::ipc(
                IpcErrorPhase::Connect,
                "cannot bind a listener on mesh relay transport".to_string(),
            )),
        }
    }
}

/// Bind a local listener — UDS on Unix, error on non-Unix.
#[cfg(unix)]
#[expect(clippy::unused_async, reason = "async signature matches non-unix stub")]
async fn bind_local(path: &std::path::Path) -> Result<TransportListener, LoamSpineError> {
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::Connect,
                format!("cannot create socket directory {}: {e}", parent.display()),
            )
        })?;
    }
    let listener = tokio::net::UnixListener::bind(path).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Connect,
            format!("UDS bind at {} failed: {e}", path.display()),
        )
    })?;
    Ok(TransportListener::Uds(listener))
}

#[cfg(not(unix))]
async fn bind_local(path: &std::path::Path) -> Result<TransportListener, LoamSpineError> {
    Err(LoamSpineError::ipc(
        IpcErrorPhase::Connect,
        format!(
            "UDS listener unavailable on this platform; \
             socket: {}. Use TCP endpoint.",
            path.display()
        ),
    ))
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
            if let Err(e) = stream.set_nodelay(true) {
                tracing::trace!("TCP set_nodelay failed (non-fatal): {e}");
            }
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

/// Parse a `"host:port"` string into a TCP [`TransportEndpoint`].
///
/// # Errors
///
/// Returns `LoamSpineError::Ipc` if the string cannot be parsed.
pub fn endpoint_from_addr(addr: &str) -> Result<TransportEndpoint, LoamSpineError> {
    let (host, port_str) = addr.rsplit_once(':').ok_or_else(|| {
        LoamSpineError::ipc(
            IpcErrorPhase::Connect,
            format!("invalid address (expected host:port): {addr}"),
        )
    })?;
    let port: u16 = port_str.parse().map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Connect,
            format!("invalid port in {addr}: {e}"),
        )
    })?;
    Ok(TransportEndpoint::tcp(host, port))
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
#[expect(
    clippy::panic,
    reason = "test assertions use panic for failure clarity"
)]
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

    #[tokio::test]
    async fn transport_listener_tcp_accept() {
        let ep = TransportEndpoint::tcp("127.0.0.1", 0);
        let listener = TransportListener::bind(&ep).await.unwrap();
        let addr = match &listener {
            TransportListener::Tcp(l) => l.local_addr().unwrap(),
            #[cfg(unix)]
            _ => panic!("expected TCP listener"),
        };

        let connect_ep = TransportEndpoint::tcp("127.0.0.1", addr.port());
        let server = tokio::spawn(async move {
            let mut stream = listener.accept().await.unwrap();
            let mut buf = [0u8; 4];
            tokio::io::AsyncReadExt::read_exact(&mut stream, &mut buf)
                .await
                .unwrap();
            assert_eq!(&buf, b"g66!");
        });

        let mut client = connect_transport(&connect_ep).await.unwrap();
        tokio::io::AsyncWriteExt::write_all(&mut client, b"g66!")
            .await
            .unwrap();
        tokio::io::AsyncWriteExt::flush(&mut client).await.unwrap();
        server.await.unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn transport_listener_uds_accept() {
        let tmp = tempfile::tempdir().unwrap();
        let sock = tmp.path().join("listen.sock");
        let ep = TransportEndpoint::uds(sock.to_string_lossy());
        let listener = TransportListener::bind(&ep).await.unwrap();

        let server = tokio::spawn(async move {
            let mut stream = listener.accept().await.unwrap();
            let mut buf = [0u8; 4];
            tokio::io::AsyncReadExt::read_exact(&mut stream, &mut buf)
                .await
                .unwrap();
            assert_eq!(&buf, b"g66!");
        });

        let mut client = connect_transport(&ep).await.unwrap();
        tokio::io::AsyncWriteExt::write_all(&mut client, b"g66!")
            .await
            .unwrap();
        tokio::io::AsyncWriteExt::flush(&mut client).await.unwrap();
        server.await.unwrap();
    }

    #[tokio::test]
    async fn transport_listener_mesh_relay_fails() {
        let ep = TransportEndpoint::mesh_relay("peer", "cap");
        let result = TransportListener::bind(&ep).await;
        assert!(result.is_err());
    }

    #[test]
    fn endpoint_from_addr_valid_ipv4() {
        let ep = endpoint_from_addr("192.168.1.1:8080").unwrap();
        assert_eq!(ep, TransportEndpoint::tcp("192.168.1.1", 8080));
    }

    #[test]
    fn endpoint_from_addr_valid_localhost() {
        let ep = endpoint_from_addr("localhost:9001").unwrap();
        assert_eq!(ep, TransportEndpoint::tcp("localhost", 9001));
    }

    #[test]
    fn endpoint_from_addr_missing_port() {
        assert!(endpoint_from_addr("192.168.1.1").is_err());
    }

    #[test]
    fn endpoint_from_addr_invalid_port() {
        assert!(endpoint_from_addr("localhost:xyz").is_err());
    }
}
