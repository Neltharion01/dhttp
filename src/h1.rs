//! HTTP/1.1 request parsing/reading

use std::io::{self, ErrorKind};
use std::fmt::{self, Write};
use std::string::FromUtf8Error;
use std::net::{IpAddr, Ipv4Addr};

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

use crate::reqres::{HttpRequest, HttpResponse, HttpHeader, HttpVersion, HttpMethod, HttpBody};
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

fn parse_header(header: &str) -> Option<HttpHeader> {
    let colon = header.find(':')?;
    let name = header[..colon].to_string();
    let value = header[colon+1..].trim().to_string();
    Some(HttpHeader { name, value })
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
pub(crate) async fn read(conn: impl HttpRead) -> Result<HttpRequest, HttpRequestError> {
    let mut lines = conn.lines();

    // get first line
    let first = lines.next_line().await?.ok_or(HttpRequestError::EarlyEof)?;
    // and slice it by 3 components
    let (method, route, version) = split3(&first).ok_or(HttpRequestError::InvalidPrelude)?;
    // then parse method, allocate route, parse version
    let method = HttpMethod::new(method);
    let route = route.to_string();
    let version = parse_ver(version).ok_or(HttpRequestError::InvalidVersion)?;
    // read headers
    let mut headers = vec![];
    loop {
        // will return if connection is shut down without \n\n
        let line = lines.next_line().await?.ok_or(HttpRequestError::EarlyEof)?;
        if line.is_empty() {
            // empty line = end of request
            break;
        }
        headers.push(parse_header(&line).ok_or(HttpRequestError::InvalidHeader)?);
    }

    let addr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
    let mut req = HttpRequest { method, route, version, headers, len: 0, addr };

    if let Some(content_length) = req.get_header("Content-Length") {
        req.len = content_length.parse().map_err(|_| HttpRequestError::InvalidLength)?;
    }

    Ok(req)
}

/// Send the request
pub(crate) async fn send(req: &HttpRequest, res: HttpResponse, conn: &mut dyn HttpConnection) -> io::Result<()> {
    let code = res.code;
    let status = code.as_str();
    let mut buf = format!("HTTP/1.1 {code} {status}\r\n");

    for header in &res.headers {
        write!(&mut buf, "{}: {}\r\n", &header.name, &header.value).unwrap();
    }

    if !res.content_type.is_empty() {
        write!(&mut buf, "Content-Type: {}\r\n", &res.content_type).unwrap();
    }

    match &res.body {
        HttpBody::Bytes(bytes) => write!(&mut buf, "Content-Length: {}", bytes.len()).unwrap(),
        HttpBody::File { len, .. } => write!(&mut buf, "Content-Length: {}", len).unwrap(),
        HttpBody::Upgrade(_) => {},
    };
    write!(&mut buf, "\r\n\r\n").unwrap();

    // Send headers
    conn.write_all(buf.as_bytes()).await?;

    // Don't send body on head requests
    if req.method == HttpMethod::Head { return Ok(()); }

    // Now, handle the body
    match res.body {
        HttpBody::Bytes(bytes) => {
            conn.write_all(&bytes).await?;
        }
        HttpBody::File { file, len } => {
            tokio::io::copy(&mut file.take(len), conn).await?;
        }
        HttpBody::Upgrade(mut handler) => {
            handler.upgrade_raw(conn).await?;
            conn.shutdown().await?;
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
    NotUnicode(FromUtf8Error),
    /// Request exceed its size limit
    TooLong,
    /// Request ended too early
    EarlyEof,
    /// First line didn't follow the `METHOD /route HTTP/1.1` format
    InvalidPrelude,
    /// Could not parse HTTP version
    InvalidVersion,
    /// Header line did not contain a colon
    InvalidHeader,
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
            HttpRequestError::InvalidHeader => fmt.write_str("header without a colon"),
            HttpRequestError::InvalidLength => fmt.write_str("content-length header did not contain a number"),
        }
    }
}

impl From<FromUtf8Error> for HttpRequestError {
    fn from(err: FromUtf8Error) -> HttpRequestError {
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
