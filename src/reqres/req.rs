use std::fmt;
use std::net::{IpAddr, Ipv4Addr};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod<'a> {
    Get, Head, Post, Put, Delete, Connect, Options, Trace, Patch,
    // it is not possible to add new variants here because for example
    // Get and Other("GET") would not be equal
    // but it is not possible to private enum variants in rust
    Other(&'a str)
}

impl<'a> HttpMethod<'a> {
    /// Parses method from provided str
    pub fn new(method: &'a str) -> HttpMethod<'a> {
        match method {
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

    pub fn as_str(self) -> &'a str {
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

impl fmt::Display for HttpMethod<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(self.as_str())
    }
}

/// Request from client to handle
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct HttpRequest<'a> {
    pub method: HttpMethod<'a>,
    pub route: &'a str,
    pub version: HttpVersion,
    pub headers: &'a str,
    /// Contents of the `Content-Length` header
    pub len: u64,
    /// IP address of this request (`0.0.0.0` if none)
    pub addr: IpAddr,
}

impl<'a> HttpRequest<'a> {
    /// Retrieves a header value, if any
    pub fn get_header(&'a self, name: &str) -> Option<&'a str> {
        let mut header = None;
        for line in self.headers.split("\r\n") {
            let mut line = line.split(':');
            let hn = line.next().unwrap();
            let Some(hv) = line.next() else { continue };
            if hn.eq_ignore_ascii_case(name) {
                header = Some(hv.trim());
                break;
            }
        }
        header
    }

    /// Checks if this header exists
    pub fn has_header(&self, name: &str) -> bool {
        self.get_header(name).is_some()
    }

    /// Compares equality of header values
    pub fn cmp_header(&self, name: &str, value: &str) -> bool {
        let hdr = self.get_header(name);
        hdr.is_some() && hdr.unwrap().eq_ignore_ascii_case(value)
    }
}

impl Default for HttpRequest<'static> {
    fn default() -> HttpRequest<'static> {
        HttpRequest {
            method: HttpMethod::Get,
            route: "",
            version: HttpVersion { major: 0, minor: 0 },
            headers: "",
            len: 0,
            addr: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        }
    }
}
