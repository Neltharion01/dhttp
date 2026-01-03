//! Request, response, status code, and their components

mod status_code;
pub use status_code::StatusCode;
mod req;
pub use req::{HttpRequest, HttpVersion, HttpMethod};
mod body;
pub use body::{HttpBody, HttpUpgrade};

pub mod res;
pub use res::HttpResponse;

pub mod sse;

mod file;

use std::fmt;

/// Header key/value
#[derive(Clone)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

impl fmt::Debug for HttpHeader {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}: {}", self.name, self.value)
    }
}
