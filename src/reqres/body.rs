use std::io;
use std::fmt;
use std::pin::Pin;

use tokio::fs::File;

use crate::core::connection::HttpConnection;
use crate::util::escape;

/// Http protocol upgrade
///
/// Allows to access underlying TCP socket with your handler
///
/// Use in [`HttpBody::Upgrade`]. Only applicable to HTTP/1.1
pub trait HttpUpgrade: Send {
    /// Handles the upgrade
    ///
    /// Equivalent signature:
    /// `async fn upgrade(&mut self, conn: &mut dyn HttpConnection) -> io::Result<()>`
    fn upgrade(&mut self, conn: &mut dyn HttpConnection) -> impl Future<Output = io::Result<()>> + Send;
}

/// Dyn version of [`HttpUpgrade`]
pub trait HttpUpgradeRaw: Send {
    /// Handles the upgrade (dyn version)
    fn upgrade_raw<'a>(&'a mut self, conn: &'a mut dyn HttpConnection) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>>;
}

impl<T: HttpUpgrade> HttpUpgradeRaw for T {
    fn upgrade_raw<'a>(&'a mut self, conn: &'a mut dyn HttpConnection) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>> {
        Box::pin(self.upgrade(conn))
    }
}

/// Body of the response
#[non_exhaustive]
pub enum HttpBody {
    /// In-memory bytes
    Bytes(Vec<u8>),
    /// File handle to read
    File { file: File, len: u64 },
    /// Protocol upgrade
    Upgrade(Box<dyn HttpUpgradeRaw>),
}

impl fmt::Debug for HttpBody {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpBody::Bytes(v) => write!(fmt, r#"HttpBody::Bytes(b"{}")"#, escape::to_utf8(v)),
            HttpBody::File { file, len } => fmt.debug_struct("HttpBody::File").field("file", file).field("len", len).finish(),
            HttpBody::Upgrade(_) => fmt.write_str("HttpBody::Upgrade(..)"),
        }
    }
}

impl From<Vec<u8>> for HttpBody {
    fn from(v: Vec<u8>) -> HttpBody {
        HttpBody::Bytes(v)
    }
}

impl From<String> for HttpBody {
    fn from(s: String) -> HttpBody {
        HttpBody::Bytes(s.into_bytes())
    }
}

impl From<&str> for HttpBody {
    fn from(s: &str) -> HttpBody {
        HttpBody::Bytes(s.to_string().into_bytes())
    }
}
