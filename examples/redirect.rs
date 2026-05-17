use std::io;

use dhttp::prelude::*;
use dhttp::reqres::{Url, res};

struct Redirect {
    dest: Url,
}

impl HttpService for Redirect {
    async fn request(&self, _route: &str, _req: &HttpRequest, _body: &mut dyn HttpRead) -> HttpResult {
        Ok(res::redirect(self.dest.clone()))
    }
}

fn main() -> io::Result<()> {
    dhttp::tokio_rt()?.block_on(http_main())
}

async fn http_main() -> io::Result<()> {
    let mut server = HttpServer::new();
    let dest = Url::new("https://google.com").unwrap();
    server.service(Redirect { dest });

    dhttp::serve_tcp("[::]:8080", server).await
}
