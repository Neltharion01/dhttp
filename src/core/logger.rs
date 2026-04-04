use crate::reqres::{HttpRequest, HttpResponse};
use crate::core::HttpError;

/// Logs http requests and errors
pub trait HttpLogger: Send + Sync + 'static {
    /// Log a successful request
    fn log(&self, req: &HttpRequest, res: &HttpResponse);
    /// Log an error
    fn err(&self, req: &HttpRequest, res: &HttpResponse, error: &dyn HttpError);
}
