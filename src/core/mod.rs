//! Core traits

mod service;
pub use service::{HttpService, HttpServiceRaw};
mod error;
pub use error::{HttpError, HttpErrorType};
mod logger;
pub use logger::HttpLogger;
mod errorhandler;
pub use errorhandler::HttpErrorHandler;
pub mod connection;
pub use connection::HttpRead;

use crate::reqres::HttpResponse;
/// Result for [`HttpService`]
pub type HttpResult<T = HttpResponse> = Result<T, Box<dyn HttpError>>;
