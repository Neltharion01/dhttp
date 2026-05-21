//! Files service

use std::path::PathBuf;

use tokio::fs;

use crate::core::{HttpService, HttpResult, HttpRead};
use crate::reqres::{res, HttpRequest, StatusCode};
use crate::util::path;

/// Hosts a directory with static files
pub struct Files {
    path: PathBuf,
}

impl Files {
    pub fn new(path: impl Into<PathBuf>) -> Files {
        Files { path: path.into() }
    }
}

impl HttpService for Files {
    async fn request(&self, route: &str, req: &HttpRequest, _body: &mut dyn HttpRead) -> HttpResult {
        let path = self.path.join(path::sanitize(route)?);

        let metadata = fs::metadata(&path).await?;

        if metadata.is_dir() {
            Err(StatusCode::NOT_FOUND.into())
        } else {
            res::file(req, &path).await
        }
    }
}
