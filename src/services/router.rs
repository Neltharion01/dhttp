use std::collections::HashMap;

use crate::core::{HttpServiceRaw, HttpService, HttpResult, HttpRead};
use crate::reqres::{HttpRequest, StatusCode};

/// Router is a service that nests other services on chosen routes
///
/// For example:
/// ```
/// # use dhttp::services::{DefaultService, Router};
/// # use dhttp::reqres::HttpMethod;
/// let mut router = Router::new();
/// router.add("/hello", DefaultService);
/// ```
/// This will show the hello message on this route, and fire a 404 on others.
///
/// Routes can be of two types:
/// - exact (does not end with `/`)
/// - nested (ends with `/`)
///
/// Exact route is a hashmap match, nested route matches anything under chosen route.
///
/// Nested route example:
/// ```
/// # use dhttp::services::{Router, FilesService};
/// # let mut router = Router::new();
/// router.add("/files/", FilesService::new("files"));
/// ```
/// This code will host all files in the "files" dir, under route `/files/`.
/// Note that it also matches just `/files`, and that nested routes strip their prefix -
/// so `/files/something` becomes `/something` in the `route` argument. Original route is still
/// accessible via `req.route`
///
/// Nested routes are implemented with a linear search, consider something more optimized
/// if you have thousands of them
///
/// # Errors
/// When a route cannot be matched, [`Router`] fires a `StatusCode(404)`
#[derive(Default)]
pub struct Router {
    /// Exact routes
    exact: HashMap<String, Box<dyn HttpServiceRaw>>,
    /// Nested routes
    nested: Vec<(String, Box<dyn HttpServiceRaw>)>,
}

impl Router {
    /// Creates an empty `Router`
    pub fn new() -> Router {
        Router::default()
    }

    /// Adds a new route
    pub fn add(&mut self, route: &str, service: impl HttpServiceRaw) -> &mut Self {
        let mut route = route.to_string();
        if route.ends_with("/") {
            route.pop();
            self.nested.push((route, Box::new(service)));
        } else {
            self.exact.insert(route, Box::new(service));
        }
        self
    }

    fn find<'a, 'b>(&'a self, route: &'b str) -> Option<(&'b str, &'a dyn HttpServiceRaw)> {
        // remove url params part
        let mut route_withoutparams = route;
        if let Some(params_index) = route.find('?') {
            route_withoutparams = &route[..params_index];
        }
        if let Some(service) = self.exact.get(route_withoutparams) {
            return Some((route, &**service));
        }

        for (r, service) in &self.nested {
            // compare prefix...
            if let Some(route) = route.strip_prefix(r) {
                // if nothing left, it matched fully...
                if route.is_empty() {
                    return Some(("/", &**service));
                // if leftover starts with /, then it matched a subsegment...
                } else if route.starts_with("/") {
                    return Some((route, &**service));
                }
                // otherwise, it didn't match anything (think of /files vs /files123)
            }
        }

        None
    }
}

impl HttpService for Router {
    async fn request(&self, route: &str, req: &HttpRequest, body: &mut dyn HttpRead) -> HttpResult {
        match self.find(route) {
            Some((route, service)) => service.request_raw(route, req, body).await,
            None => Err(StatusCode::NOT_FOUND.into()),
        }
    }

    fn filter(&self, route: &str, req: &HttpRequest) -> HttpResult<()> {
        match self.find(route) {
            Some((route, service)) => service.filter_raw(route, req),
            None => Err(StatusCode::NOT_FOUND.into()),
        }
    }
}
