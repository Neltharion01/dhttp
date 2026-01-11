//! Connection types

use std::io::{self, Read, Write, Take};
use std::net::{SocketAddr, Shutdown};

use may::net::TcpStream;

/// Async buffered reader stream
pub trait HttpRead: Read + Unpin + Send + Sync {}
impl<T: Read + Unpin + Send + Sync> HttpRead for T {}

/// Async writer stream
pub trait HttpWrite: Write + Unpin + Send + Sync {}
impl<T: Write + Unpin + Send + Sync> HttpWrite for T {}

/// Async Read/Write stream that represents an HTTP connection
pub trait HttpConnection: HttpRead + HttpWrite {
    /// Retrieve client's IP address of this connection
    fn getpeername(&self) -> io::Result<SocketAddr>;
    /// Is it HTTPS?
    fn is_secure(&self) -> bool;

    fn shutdown(&self) -> io::Result<()>;
}
impl HttpConnection for TcpStream {
    fn getpeername(&self) -> io::Result<SocketAddr> {
        self.peer_addr()
    }

    fn is_secure(&self) -> bool {
        false
    }

    fn shutdown(&self) -> io::Result<()> {
        self.shutdown(Shutdown::Both)
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

    fn shutdown(&self) -> io::Result<()> {
        (**self).shutdown()
    }
}

pub(crate) struct EmitContinue<T: HttpConnection> {
    pub conn: Take<T>,
    pub to_send: &'static [u8],
}

impl<T: HttpConnection> Read for EmitContinue<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.conn.get_mut().write_all(self.to_send)?;
        self.conn.read(buf)
    }
}
