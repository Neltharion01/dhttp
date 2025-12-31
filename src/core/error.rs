use std::convert::Infallible;
use std::io::{self, ErrorKind};
use std::error::Error;

use crate::reqres::StatusCode;

/// How should this error be handled
///
/// Returned from [`HttpError::error_type`]
#[derive(Debug, Clone, Copy)]
pub enum HttpErrorType {
    /// Terminates the connection (examples: network I/O error)
    Fatal,
    /// Status code (examples: 404 Not found, 403 Forbidden)
    Hidden,
    /// Error with detailed description (could not parse json, wrong file type, etc)
    User,
}

/// Error trait for any service error
/// # Example implementation
/// `impl HttpError for MyError {}`
pub trait HttpError: Error + Send + 'static {
    /// Name of this error (type name by default)
    fn name(&self) -> &'static str {
        // everything after last ::
        std::any::type_name::<Self>().split("::").last().unwrap()
    }

    /// How should this error be handled, check [`HttpErrorType`] for more info (`User` by default)
    fn error_type(&self) -> HttpErrorType {
        HttpErrorType::User
    }

    /// Provides HTTP-friendly description of this error (`.to_string()` by default)
    fn http_description(&self) -> String {
        self.to_string()
    }

    /// Which status code should be used for this error
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl<E: HttpError> From<E> for Box<dyn HttpError> {
    fn from(value: E) -> Box<dyn HttpError> {
        Box::new(value)
    }
}

fn is_net(kind: ErrorKind) -> bool {
    matches!(kind, ErrorKind::ConnectionRefused
    | ErrorKind::ConnectionReset
    | ErrorKind::HostUnreachable
    | ErrorKind::NetworkUnreachable
    | ErrorKind::ConnectionAborted
    | ErrorKind::NotConnected
    | ErrorKind::AddrInUse
    | ErrorKind::AddrNotAvailable
    | ErrorKind::NetworkDown
    | ErrorKind::BrokenPipe
    | ErrorKind::TimedOut)
}

impl HttpError for io::Error {
    fn name(&self) -> &'static str {
        "io::Error"
    }

    fn error_type(&self) -> HttpErrorType {
        if is_net(self.kind()) {
            HttpErrorType::Fatal
        } else {
            HttpErrorType::User
        }
    }

    fn http_description(&self) -> String {
        match self.kind() {
            ErrorKind::NotFound | ErrorKind::NotADirectory => "The requested resource was not found on this server".to_string(),
            ErrorKind::PermissionDenied => "Access denied".to_string(),
            _ => self.to_string(),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self.kind() {
            ErrorKind::NotFound | ErrorKind::NotADirectory => StatusCode::NOT_FOUND,
            ErrorKind::PermissionDenied => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl HttpError for Infallible {}

impl HttpError for tokio::task::JoinError {
    fn error_type(&self) -> HttpErrorType {
        // This is a panic message, should not be displayed
        HttpErrorType::Hidden
    }
}

impl HttpError for std::string::FromUtf8Error {}
