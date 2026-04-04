use std::io;

use dhttp::prelude::*;
use dhttp::reqres::res;
use dhttp::services::{Router, FilesService};

struct StatusService;
impl HttpService for StatusService {
    async fn request(&self, _route: &str, _req: &HttpRequest, _body: &mut dyn HttpRead) -> HttpResult {
        Ok(res::json(concat!(r#"{"status": "All systems running normally!"}"#, '\n')))
    }
}

struct JsonErrorHandler;
impl HttpErrorHandler for JsonErrorHandler {
    fn error(&self, _req: &HttpRequest, error: &dyn HttpError) -> HttpResponse {
        let code = error.status_code();
        let desc = error.http_description();
        let mut json = format!(r#"{{"code": {code}, "error": "{desc}"}}"#);
        json.push('\n');

        res::json(json)
    }

    fn plain_code(&self, code: StatusCode) -> HttpResponse {
        let mut json = format!(r#"{{"code": {code}}}"#);
        json.push('\n');

        res::json(json)
    }
}

fn main() -> io::Result<()> {
    dhttp::tokio_rt()?.block_on(http_main())
}

async fn http_main() -> io::Result<()> {
    let mut server = HttpServer::new();
    let mut router = Router::new();
    router
        .add("/status", StatusService)
        .add("/files/", FilesService::new("examples"));
    server.service(router);
    server.error_handler(JsonErrorHandler);

    dhttp::serve_tcp("[::]:8080", server).await
}
