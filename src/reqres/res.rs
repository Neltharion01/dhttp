//! HTTP response and its constructors

use percent_encoding_lite::{is_encoded, encode, Bitmask};

use crate::reqres::{HttpHeader, HttpBody, StatusCode};
use crate::reqres::sse::HttpSse;

/// Your response
#[derive(Debug)]
#[non_exhaustive]
pub struct HttpResponse {
    pub code: StatusCode,
    pub headers: Vec<HttpHeader>,
    pub body: HttpBody,
    pub content_type: String,
}

impl HttpResponse {
    /// An empty response
    pub fn new() -> HttpResponse {
        HttpResponse::with_type("", vec![])
    }

    /// Pushes a new header
    pub fn add_header(&mut self, name: impl Into<String>, value: impl Into<String>) -> &mut HttpResponse {
        self.headers.push(HttpHeader { name: name.into(), value: value.into() });
        self
    }

    /// Constructs new response with a specified `Content-Type`
    pub fn with_type(content_type: impl Into<String>, body: impl Into<HttpBody>) -> HttpResponse {
        HttpResponse {
            code: StatusCode::OK,
            headers: vec![],
            body: body.into(),
            content_type: content_type.into(),
        }
    }
}

impl Default for HttpResponse {
    fn default() -> HttpResponse {
        HttpResponse::new()
    }
}

/// Response of bytes (`application/octet-stream`)
pub fn bytes(bytes: Vec<u8>) -> HttpResponse {
    HttpResponse::with_type("application/octet-stream", bytes)
}

/// Plaintext response (`text/plain`)
pub fn text(text: impl Into<String>) -> HttpResponse {
    HttpResponse::with_type("text/plain; charset=utf-8", text.into())
}

/// HTML response (`text/html`)
pub fn html(html: impl Into<String>) -> HttpResponse {
    HttpResponse::with_type("text/html; charset=utf-8", html.into())
}

/// JSON response (`application/json`)
pub fn json(json: impl Into<String>) -> HttpResponse {
    HttpResponse::with_type("application/json", json.into())
}

/// HTTP redirect with the `Location` header
pub fn redirect(dest: impl Into<String>) -> HttpResponse {
    let mut dest = dest.into();
    // To avoid XSS for URLs containing a double quote or back slash
    if !is_encoded(&dest, Bitmask::URI) {
        dest = encode(dest, Bitmask::URI);
    }
    HttpResponse {
        code: StatusCode::MOVED_PERMANENTLY,
        headers: vec![HttpHeader { name: "Location".to_string(), value: dest.clone() }],
        body: format!("<a href=\"{dest}\">Click here if you weren't redirected</a>\n").into(),
        content_type: "text/html; charset=utf-8".to_string(),
    }
}

pub fn sse(handler: impl HttpSse) -> HttpResponse {
    HttpResponse::with_type("text/event-stream", HttpBody::Upgrade(Box::new(handler)))
}

pub use super::file::file;
