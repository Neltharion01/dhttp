use crate::reqres::{HttpRequest, HttpResponse, StatusCode};
use crate::core::HttpError;

/// Handler for possible service errors
///
/// Returned status code (`res.code`) is not used, it is overriden by original status code.
/// This is made to avoid returning `200 Ok` errors
///
/// If you want to change the code, consider altering the [`HttpError`] implementation
pub trait HttpErrorHandler: Send + Sync + 'static {
    /// Constructs an [`HttpResponse`] from given [`HttpError`]
    fn error(&self, req: &HttpRequest, error: &dyn HttpError) -> HttpResponse;

    /// Shows a plain error code page for internal errors
    fn plain_code(&self, code: StatusCode) -> HttpResponse;
}
