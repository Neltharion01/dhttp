use crate::reqres::{HttpRequest, HttpMethod, StatusCode};
use crate::core::{HttpResult, HttpRead};

/// Basic building block of your web application
pub trait HttpService: Send + Sync + 'static {
    /// Serve the request
    ///
    /// The `route` argument contains the resolved route, while `req.route` contains the full original route.
    /// Always use `route` instead of `req.route`!
    fn request(&self, route: &str, req: &HttpRequest, body: &mut dyn HttpRead) -> HttpResult;

    /// Checks if request is valid
    ///
    /// By default, it checks that route is `"/"`, method is [`HttpMethod::Get`] and `req.len` is 0
    fn filter(&self, route: &str, req: &HttpRequest) -> HttpResult<()> {
        if route != "/" { return Err(StatusCode::NOT_FOUND.into()); }
        if req.method != HttpMethod::Get { return Err(StatusCode::METHOD_NOT_ALLOWED.into()); }
        if req.len > 0 { return Err(StatusCode::REQUEST_ENTITY_TOO_LARGE.into()); }
        Ok(())
    }
}
