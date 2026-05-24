//! Redirect service

use crate::core::{HttpService, HttpResult, HttpRead};
use crate::reqres::{res, HttpRequest};

/// Redirects to an another location
pub struct Redirect {
    location: String,
}

impl Redirect {
    pub fn new(location: impl Into<String>) -> Redirect {
        let location = location.into();
        Redirect { location }
    }
}

impl HttpService for Redirect {
    async fn request(&self, _route: &str, _req: &HttpRequest, _body: &mut dyn HttpRead) -> HttpResult {
        Ok(res::redirect(&self.location))
    }
}
