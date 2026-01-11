use std::io;
use std::fmt;
use std::fs::File;

use crate::core::connection::HttpConnection;
use crate::util::escape;

/// Http protocol upgrade
///
/// Allows to access underlying TCP socket with your handler
///
/// Use in [`HttpBody::Upgrade`]. Only applicable to HTTP/1.1
pub trait HttpUpgrade: Send {
    /// Handles the upgrade
    fn upgrade(&mut self, conn: &mut dyn HttpConnection) -> io::Result<()>;
}

/// Body of the response
#[non_exhaustive]
pub enum HttpBody {
    /// No data, **does not have Content-Length**
    Empty,
    /// In-memory bytes
    Bytes(Vec<u8>),
    /// File handle to read
    File { file: File, len: u64 },
    /// Protocol upgrade
    Upgrade(Box<dyn HttpUpgrade>),
}

impl fmt::Debug for HttpBody {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpBody::Empty => fmt.write_str("HttpBody::Empty"),
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
