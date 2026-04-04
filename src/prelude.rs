//! Most used types
//!
//! Almost all (except [`StatusCode`]) are prefixed with "Http", so don't worry about name conflicts

pub use crate::core::{HttpService, HttpServiceRaw, HttpResult, HttpError, HttpErrorHandler, HttpErrorType, HttpRead};
pub use crate::reqres::{HttpRequest, HttpResponse, HttpMethod, StatusCode};
pub use crate::reqres::sse::{HttpSse, HttpSseEvent};
pub use crate::server::HttpServer;
