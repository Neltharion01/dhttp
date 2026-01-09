//! Files service

use std::path::PathBuf;

use tokio::fs;

use crate::core::{HttpService, HttpResult, HttpRead};
use crate::reqres::{res, HttpRequest, HttpMethod, StatusCode};
use crate::util::path;

/// Hosts a directory with static files
pub struct FilesService {
    pub path: PathBuf,
}

impl FilesService {
    pub fn new(path: impl Into<PathBuf>) -> FilesService {
        FilesService { path: path.into() }
    }
}

impl HttpService for FilesService {
    async fn request(&self, route: &str, req: &HttpRequest, _body: &mut dyn HttpRead) -> HttpResult {
        let path = self.path.join(path::sanitize(route)?);

        let metadata = fs::metadata(&path).await?;

        if metadata.is_dir() {
            Err(StatusCode::NOT_FOUND.into())
        } else {
            res::file(req, &path).await
        }
    }

    fn filter(&self, _route: &str, req: &HttpRequest) -> HttpResult<()> {
        if req.method != HttpMethod::Get && req.method != HttpMethod::Head {
            return Err(StatusCode::METHOD_NOT_ALLOWED.into());
        }
        if req.len > 0 { return Err(StatusCode::REQUEST_ENTITY_TOO_LARGE.into()); }
        Ok(())
    }
}
