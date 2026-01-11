//! HTTP/1.1 request parsing/reading

use std::io::{self, ErrorKind, Read, BufReader, BufRead, Write};
use std::fmt;
use std::str::Utf8Error;
use std::net::{IpAddr, Ipv4Addr};

use crate::reqres::{HttpRequest, HttpResponse, HttpVersion, HttpMethod, HttpBody};
use crate::core::connection::{HttpRead, HttpConnection};

fn parse_ver(ver: &str) -> Option<HttpVersion> {
    let mut split = ver.strip_prefix("HTTP/")?.split('.');
    let major = split.next()?.parse().ok()?;
    let minor = split.next()?.parse().ok()?;
    if split.next().is_some() {
        // 3rd element is invalid
        return None;
    }
    Some(HttpVersion { major, minor })
}

fn split3(line: &str) -> Option<(&str, &str, &str)> {
    let mut split = line.split_whitespace();
    let (method, route, version) = (split.next()?, split.next()?, split.next()?);
    if split.next().is_some() {
        // 4th element is invalid
        return None;
    }
    Some((method, route, version))
}

/// Reads a request from the provided stream
pub(crate) fn read<'a>(buf: &'a mut Vec<u8>, conn: impl HttpRead) -> Result<HttpRequest<'a>, HttpRequestError> {
    let mut conn = BufReader::new(conn);

    let mut read = 0;
    while read != 1 && read != 2 { // '\n' or '\r\n'
        read = conn.read_until(b'\n', buf)?;
        if read == 0 { return Err(HttpRequestError::EarlyEof); }
    }

    let buf = str::from_utf8(buf)?;
    let first_line = buf.find("\r\n").ok_or(HttpRequestError::EarlyEof)?;

    let (method, route, version) = split3(&buf[..first_line]).ok_or(HttpRequestError::InvalidPrelude)?;
    let method = HttpMethod::new(method);
    let version = parse_ver(version).ok_or(HttpRequestError::InvalidVersion)?;
    let headers = &buf[first_line+2..];

    let addr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
    let mut req = HttpRequest { method, route, version, headers, len: 0, addr };

    if let Some(content_length) = req.get_header("Content-Length") {
        req.len = content_length.parse().map_err(|_| HttpRequestError::InvalidLength)?;
    }

    Ok(req)
}

/// Send the request
pub(crate) fn send(req: HttpRequest<'_>, mut res: HttpResponse, conn: &mut dyn HttpConnection) -> io::Result<()> {
    let buf = &mut res.contents;
    match &res.body {
        HttpBody::Bytes(bytes) => write!(buf, "Content-Length: {}\r\n", bytes.len()).unwrap(),
        HttpBody::File { len, .. } => write!(buf, "Content-Length: {}\r\n", len).unwrap(),
        HttpBody::Empty | HttpBody::Upgrade(_) => {},
    };
    buf.extend(b"\r\n");

    // Send headers
    conn.write_all(&buf)?;

    // Don't send body on head requests
    if req.method == HttpMethod::Head { return Ok(()); }

    // Now, handle the body
    match res.body {
        HttpBody::Empty => {},
        HttpBody::Bytes(bytes) => {
            conn.write_all(&bytes)?;
        }
        HttpBody::File { file, len } => {
            io::copy(&mut file.take(len), conn)?;
        }
        HttpBody::Upgrade(mut handler) => {
            handler.upgrade(conn)?;
            conn.shutdown()?;
        }
    }

    Ok(())
}

/// Error when parsing an HTTP/1.1 request.
/// For debugging purposes only
#[derive(Debug)]
pub(crate) enum HttpRequestError {
    /// IO error
    Io(io::Error),
    /// Some part of request contained invalid Unicode
    NotUnicode(Utf8Error),
    /// Request exceed its size limit
    TooLong,
    /// Request ended too early
    EarlyEof,
    /// First line didn't follow the `METHOD /route HTTP/1.1` format
    InvalidPrelude,
    /// Could not parse HTTP version
    InvalidVersion,
    /// Header line did not contain a colon
//    InvalidHeader,
    /// `Content-Length` header did not contain a number
    InvalidLength,
}

impl fmt::Display for HttpRequestError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpRequestError::Io(err) => err.fmt(fmt),
            HttpRequestError::NotUnicode(err) => err.fmt(fmt),
            HttpRequestError::TooLong => fmt.write_str("request too long"),
            HttpRequestError::EarlyEof => fmt.write_str("incomplete request"),
            // first line of request did not contain exactly 3 elements (method, path and version)
            HttpRequestError::InvalidPrelude => fmt.write_str("invalid prelude"),
            HttpRequestError::InvalidVersion => fmt.write_str("invalid http version"),
//            HttpRequestError::InvalidHeader => fmt.write_str("header without a colon"),
            HttpRequestError::InvalidLength => fmt.write_str("content-length header did not contain a number"),
        }
    }
}

impl From<Utf8Error> for HttpRequestError {
    fn from(err: Utf8Error) -> HttpRequestError {
        HttpRequestError::NotUnicode(err)
    }
}

impl From<io::Error> for HttpRequestError {
    fn from(err: io::Error) -> HttpRequestError {
        if err.kind() == ErrorKind::UnexpectedEof {
            HttpRequestError::TooLong
        } else {
            HttpRequestError::Io(err)
        }
    }
}
