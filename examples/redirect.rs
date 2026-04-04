use std::io;

use dhttp::prelude::*;
use dhttp::reqres::res;

struct Redirect {
    dest: String,
}

impl HttpService for Redirect {
    async fn request(&self, _route: &str, _req: &HttpRequest, _body: &mut dyn HttpRead) -> HttpResult {
        Ok(res::redirect(&self.dest))
    }
}

fn main() -> io::Result<()> {
    dhttp::tokio_rt()?.block_on(http_main())
}

async fn http_main() -> io::Result<()> {
    let mut server = HttpServer::new();
    let dest = "https://google.com".to_string();
    server.service(Redirect { dest });

    dhttp::serve_tcp("[::]:8080", server).await
}
