use crate::core::{HttpService, HttpResult, HttpRead};
use crate::reqres::{res, HttpRequest};

/// Default service which only returns `"drakohttp is here!"`
pub struct DefaultService;

impl HttpService for DefaultService {
    async fn request(&self, _route: &str, _req: &HttpRequest, _body: &mut dyn HttpRead) -> HttpResult {
        Ok(res::text("drakohttp is here!\n"))
    }
}
