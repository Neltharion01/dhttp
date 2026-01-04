use std::fmt;
use std::error::Error;

use crate::core::{HttpError, HttpErrorType};

/// An HTTP status code
#[derive(Debug, Clone, Copy)]
pub struct StatusCode(pub u16);

impl StatusCode {
    /// Provides a textual representation of this status code
    /// # Example
    /// ```
    /// # use dhttp::reqres::StatusCode;
    /// assert_eq!(StatusCode(200).as_str(), "OK");
    /// assert_eq!(StatusCode(404).as_str(), "Not found");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self.0 {
            200 => "OK",
            206 => "Partial content",
            301 => "Moved permanently",
            304 => "Not modified",
            400 => "Bad request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not found",
            405 => "Method not allowed",
            413 => "Request entity too large",
            416 => "Range not satisfiable",
            500 => "Internal server error",
            505 => "HTTP version not supported",
            _ => "Unknown",
        }
    }
}

impl StatusCode {
    // 2xx

    /// 200
    pub const OK: StatusCode = StatusCode(200);
    /// 206
    pub const PARTIAL_CONTENT: StatusCode = StatusCode(206);

    // 3xx

    /// 301
    pub const MOVED_PERMANENTLY: StatusCode = StatusCode(301);
    /// 304
    pub const NOT_MODIFIED: StatusCode = StatusCode(304);

    // 4xx

    /// 400
    pub const BAD_REQUEST: StatusCode = StatusCode(400);
    /// 401
    pub const UNAUTHORIZED: StatusCode = StatusCode(401);
    /// 403
    pub const FORBIDDEN: StatusCode = StatusCode(403);
    /// 404
    pub const NOT_FOUND: StatusCode = StatusCode(404);
    /// 405
    pub const METHOD_NOT_ALLOWED: StatusCode = StatusCode(405);
    /// 413
    pub const REQUEST_ENTITY_TOO_LARGE: StatusCode = StatusCode(413);
    /// 416
    pub const RANGE_NOT_SATISFIABLE: StatusCode = StatusCode(416);

    // 5xx

    /// 500
    pub const INTERNAL_SERVER_ERROR: StatusCode = StatusCode(500);
    /// 505
    pub const HTTP_VERSION_NOT_SUPPORTED: StatusCode = StatusCode(505);
}

impl fmt::Display for StatusCode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(fmt)
    }
}

impl Error for StatusCode {}

impl HttpError for StatusCode {
    fn error_type(&self) -> HttpErrorType {
        HttpErrorType::Hidden
    }

    fn status_code(&self) -> StatusCode {
        *self
    }
}
