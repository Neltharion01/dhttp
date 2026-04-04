use std::io;

use dhttp::prelude::*;
use dhttp::reqres::res;
use tokio::io::AsyncReadExt;

struct EchoService;

impl HttpService for EchoService {
    async fn request(&self, _route: &str, _req: &HttpRequest, body: &mut dyn HttpRead) -> HttpResult {
        let mut bytes = vec![];
        body.read_to_end(&mut bytes).await?;
        let s = String::from_utf8(bytes)?;
        let res;
        if s.is_empty() {
            res = res::text("You sent nothing!\n");
        } else {
            res = res::text(format!("You sent: {s}\n"));
        }
        Ok(res)
    }

    fn filter(&self, _route: &str, req: &HttpRequest) -> HttpResult<()> {
        if req.method != HttpMethod::Post { return Err(StatusCode::METHOD_NOT_ALLOWED.into()); }
        if req.len > 65536 { return Err(StatusCode::REQUEST_ENTITY_TOO_LARGE.into()); }
        Ok(())
    }
}

fn main() -> io::Result<()> {
    dhttp::tokio_rt()?.block_on(http_main())
}

async fn http_main() -> io::Result<()> {
    let mut server = HttpServer::new();
    server.service(EchoService);

    dhttp::serve_tcp("[::]:8080", server).await
}
