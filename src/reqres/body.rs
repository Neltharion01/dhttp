use std::fmt;

use tokio::fs::File;

use crate::util::escape;
use crate::reqres::sse::HttpSseRaw;

/// Body of the response
#[non_exhaustive]
pub enum HttpBody {
    /// No data, **does not have Content-Length**
    Empty,
    /// In-memory bytes
    Bytes(Vec<u8>),
    /// File handle to read
    File { file: File, len: u64 },
    /// Server sent events
    Sse(Box<dyn HttpSseRaw>),
}

impl fmt::Debug for HttpBody {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpBody::Empty => fmt.write_str("HttpBody::Empty"),
            HttpBody::Bytes(v) => write!(fmt, r#"HttpBody::Bytes(b"{}")"#, escape::to_utf8(v)),
            HttpBody::File { file, len } => fmt.debug_struct("HttpBody::File").field("file", file).field("len", len).finish(),
            HttpBody::Sse(_) => fmt.write_str("HttpBody::Sse(..)"),
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
