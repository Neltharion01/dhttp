use std::pin::Pin;
use std::fmt::Debug;

use tracing::instrument;

use crate::reqres::{HttpRequest, HttpMethod, StatusCode};
use crate::core::{HttpResult, HttpRead};

/// Basic building block of your web application
///
/// Use it to implement the service, and use [`HttpServiceRaw`] to call it from a `&dyn` reference
pub trait HttpService: Debug + Send + Sync + 'static {
    /// Serve the request
    ///
    /// Equivalent signature:
    /// `async fn request(&self, route: &str, req: &HttpRequest, body: &dyn HttpRead) -> HttpResult`
    ///
    /// The `route` argument contains the resolved route, while `req.route` contains the full original route.
    /// Always use `route` instead of `req.route`!
    fn request(&self, route: &str, req: &HttpRequest, body: &mut dyn HttpRead) -> impl Future<Output = HttpResult> + Send;

    /// Checks if request is valid
    ///
    /// By default, it checks that route is `"/"`, method is [`HttpMethod::Get`] and `req.len` is 0
    #[instrument]
    fn filter(&self, route: &str, req: &HttpRequest) -> HttpResult<()> {
        if route != "/" { return Err(StatusCode::NOT_FOUND.into()); }
        if req.method != HttpMethod::Get { return Err(StatusCode::METHOD_NOT_ALLOWED.into()); }
        if req.len > 0 { return Err(StatusCode::REQUEST_ENTITY_TOO_LARGE.into()); }
        Ok(())
    }
}

/// Dyn version of [`HttpService`]
///
/// The raw version is required to overcome the ugliness of dyn Future signatures
///
/// Use it to call the service, and use [`HttpService`] to implement it.
pub trait HttpServiceRaw: Send + Sync + 'static {
    /// Serve the request (dyn version)
    fn request_raw<'a>(&'a self, route: &'a str, req: &'a HttpRequest, body: &'a mut dyn HttpRead) -> Pin<Box<dyn Future<Output = HttpResult> + Send + 'a>>;
    /// Checks if request is valid (dyn version)
    fn filter_raw(&self, route: &str, req: &HttpRequest) -> HttpResult<()>;
}

impl<T: HttpService> HttpServiceRaw for T {
    fn request_raw<'a>(&'a self, route: &'a str, req: &'a HttpRequest, body: &'a mut dyn HttpRead) -> Pin<Box<dyn Future<Output = HttpResult> + Send + 'a>> {
        Box::pin(self.request(route, req, body))
    }

    fn filter_raw(&self, route: &str, req: &HttpRequest) -> HttpResult<()> {
        self.filter(route, req)
    }
}
