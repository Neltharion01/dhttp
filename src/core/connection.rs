//! Connection types

use std::io::{self, ErrorKind};
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll, ready};
use std::fmt::Debug;

use tokio::io::{AsyncRead, AsyncBufRead, AsyncWrite, BufReader, ReadBuf, Take};
use tokio::net::TcpStream;
use tracing::instrument;

/// Async buffered reader stream
pub trait HttpRead: Debug + AsyncBufRead + Unpin + Send + Sync {}
impl<T: Debug + AsyncBufRead + Unpin + Send + Sync> HttpRead for T {}

/// Async writer stream
pub trait HttpWrite: Debug + AsyncWrite + Unpin + Send + Sync {}
impl<T: Debug + AsyncWrite + Unpin + Send + Sync> HttpWrite for T {}

/// Async BufRead/Write stream that represents an HTTP connection
pub trait HttpConnection: HttpRead + HttpWrite {
    /// Retrieve client's IP address of this connection
    fn getpeername(&self) -> io::Result<SocketAddr>;
    /// Is it HTTPS?
    fn is_secure(&self) -> bool;
}
impl HttpConnection for BufReader<TcpStream> {
    #[instrument]
    fn getpeername(&self) -> io::Result<SocketAddr> {
        self.get_ref().peer_addr()
    }

    fn is_secure(&self) -> bool {
        false
    }
}
// rustc why is this not automatic?????
impl<T: HttpConnection> HttpConnection for &mut T {
    fn getpeername(&self) -> io::Result<SocketAddr> {
        (**self).getpeername()
    }

    fn is_secure(&self) -> bool {
        (**self).is_secure()
    }
}

#[derive(Debug)]
pub(crate) struct EmitContinue<T: HttpConnection> {
    pub conn: Take<T>,
    pub to_send: &'static [u8],
}

impl<T: HttpConnection> AsyncRead for EmitContinue<T> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        while !self.to_send.is_empty() {
            let to_send = self.to_send;
            let written = ready!(Pin::new(self.conn.get_mut()).poll_write(cx, to_send))?;
            if written == 0 { return Poll::Ready(Err(ErrorKind::WriteZero.into())); }
            self.to_send = &self.to_send[written..];
        }

        Pin::new(&mut self.conn).poll_read(cx, buf)
    }
}

impl<T: HttpConnection> AsyncBufRead for EmitContinue<T> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        Pin::new(&mut Pin::into_inner(self).conn).poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        Pin::new(&mut Pin::into_inner(self).conn).consume(amt)
    }
}
