use std::fmt;
use std::net::{IpAddr, Ipv4Addr};

use crate::reqres::HttpHeader;

/// Version used in request
#[derive(Clone, Copy)]
pub struct HttpVersion {
    pub major: u8,
    pub minor: u8,
}

impl HttpVersion {
    /// Compares this version for equality
    pub fn is(self, major: u8, minor: u8) -> bool {
        self.major == major && self.minor == minor
    }
}

impl fmt::Debug for HttpVersion {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "HTTP/{}.{}", self.major, self.minor)
    }
}

/// Method of request
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    Get, Head, Post, Put, Delete, Connect, Options, Trace, Patch,
    // it is not possible to add new variants here because for example
    // Get and Other("GET") would not be equal
    // but it is not possible to private enum variants in rust
    Other(String)
}

impl HttpMethod {
    /// Parses method from provided str
    pub fn new(method: &str) -> HttpMethod {
        let method = method.to_ascii_uppercase();
        match method.as_str() {
            "GET" => HttpMethod::Get,
            "HEAD" => HttpMethod::Head,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "CONNECT" => HttpMethod::Connect,
            "OPTIONS" => HttpMethod::Options,
            "TRACE" => HttpMethod::Trace,
            "PATCH" => HttpMethod::Patch,
            _ => HttpMethod::Other(method),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Head => "HEAD",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Connect => "CONNECT",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Trace => "TRACE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Other(other) => other,
        }
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(self.as_str())
    }
}

/// Request from client to handle
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub route: String,
    pub version: HttpVersion,
    pub headers: Vec<HttpHeader>,
    /// Contents of the `Content-Length` header
    pub len: u64,
    /// IP address of this request (`0.0.0.0` if none)
    pub addr: IpAddr,
}

impl HttpRequest {
    /// Retrieves a header value, if any
    pub fn get_header<'a>(&'a self, name: &str) -> Option<&'a str> {
        let mut header = None;
        for h in &self.headers {
            if h.name.eq_ignore_ascii_case(name) {
                header = Some(h.value.as_str());
                break;
            }
        }
        header
    }
}

impl Default for HttpRequest {
    fn default() -> HttpRequest {
        HttpRequest {
            method: HttpMethod::Get,
            route: String::new(),
            version: HttpVersion { major: 0, minor: 0 },
            headers: vec![],
            len: 0,
            addr: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        }
    }
}
