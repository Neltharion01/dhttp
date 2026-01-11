//! HTTP response and its constructors

use std::io::Write;

use blake3_lite::Hasher;
use percent_encoding_lite::{is_encoded, encode, Bitmask};

use crate::reqres::{HttpRequest, HttpBody, StatusCode};
use crate::reqres::sse::HttpSse;

/// Your response
#[derive(Debug)]
#[non_exhaustive]
pub struct HttpResponse {
    pub contents: Vec<u8>,
    pub body: HttpBody,
}

impl HttpResponse {
    /// An empty response
    pub fn new(code: StatusCode) -> HttpResponse {
        let mut contents = vec![];
        write!(&mut contents, "HTTP/1.1 {} {}\r\n", code, code.as_str()).unwrap();
        HttpResponse { contents, body: vec![].into() }
    }

    /// Pushes a new header
    pub fn add_header(&mut self, name: &str, value: &str) -> &mut HttpResponse {
        write!(&mut self.contents, "{name}: {value}\r\n").unwrap();
        self
    }

    /// Constructs new response with a specified `Content-Type`
    pub fn with_type(code: StatusCode, content_type: &str, body: impl Into<HttpBody>) -> HttpResponse {
        let mut res = HttpResponse::new(code);
        res.add_header("Content-Type", content_type);
        res.body = body.into();
        res
    }
}

/*impl Default for HttpResponse {
    fn default() -> HttpResponse {
        HttpResponse::new(StatusCode::OK)
    }
}*/

/// Response of bytes (`application/octet-stream`)
pub fn bytes(code: StatusCode, bytes: Vec<u8>) -> HttpResponse {
    HttpResponse::with_type(code, "application/octet-stream", bytes)
}

/// Plaintext response (`text/plain`)
pub fn text(code: StatusCode, text: impl Into<String>) -> HttpResponse {
    HttpResponse::with_type(code, "text/plain; charset=utf-8", text.into())
}

/// HTML response (`text/html`)
///
/// Also stamps an ETag to enable caching
pub fn html(req: &HttpRequest, code: StatusCode, html: impl Into<String>) -> HttpResponse {
    let html = html.into();

    let mut hasher = Hasher::new();
    hasher.update(html.as_bytes());
    // You'd probably execute me for that but I see no security flaws to truncate Blake3 for caching purposes
    let mut hash = [0; 8];
    hasher.finalize(&mut hash);
    let hex = format!("\"{}\"", crate::util::hex(&hash));

    if req.cmp_header("If-None-Match", &hex) {
        let mut res = HttpResponse::with_type(
            StatusCode::NOT_MODIFIED,
            "text/html; charset=utf-8",
            HttpBody::Empty,
        );
        res.add_header("ETag", &hex);
        return res;
    }
    let mut res = HttpResponse::with_type(code, "text/html; charset=utf-8", html);
    res.add_header("ETag", &hex);
    res
}

/// JSON response (`application/json`)
pub fn json(code: StatusCode, json: impl Into<String>) -> HttpResponse {
    HttpResponse::with_type(code, "application/json", json.into())
}

/// HTTP redirect with the `Location` header
pub fn redirect(dest: &str) -> HttpResponse {
    let dest_encoded;
    // To avoid XSS for URLs containing a double quote or back slash
    if !is_encoded(dest, Bitmask::URI) {
        dest_encoded = encode(dest, Bitmask::URI);
    } else {
        dest_encoded = dest.to_string();
    }

    let mut res = HttpResponse::with_type(
        StatusCode::MOVED_PERMANENTLY,
        "text/html; charset=utf-8",
        format!("<a href=\"{dest}\">Click here if you weren't redirected</a>\n"),
    );
    res.add_header("Location", &dest_encoded);
    res
}

pub fn sse(handler: impl HttpSse) -> HttpResponse {
    HttpResponse::with_type(StatusCode::OK, "text/event-stream", HttpBody::Upgrade(Box::new(handler)))
}

pub use super::file::file;
