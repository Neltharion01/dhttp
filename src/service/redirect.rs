//! Redirect service

use crate::core::{HttpService, HttpResult, HttpRead};
use crate::reqres::{res, HttpRequest, Url};

/// Redirects to an another location
pub struct Redirect {
    location: Url,
}

impl Redirect {
    pub fn new(location: &'static str) -> Redirect {
        // It intentionally takes static str - it is a convenient method that takes known valid URL strings
        let location = Url::new(location).expect("should only contain characters allowed in Bitmask::URL");
        Redirect { location }
    }

    pub fn from_url(location: Url) -> Redirect {
        Redirect { location }
    }
}

impl HttpService for Redirect {
    async fn request(&self, _route: &str, _req: &HttpRequest, _body: &mut dyn HttpRead) -> HttpResult {
        Ok(res::redirect(self.location.clone()))
    }
}
