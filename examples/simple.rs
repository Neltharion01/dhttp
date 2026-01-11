use std::io;

use dhttp::prelude::*;
use dhttp::reqres::res;
use tracing::instrument;

#[derive(Debug)]
struct MyService {
    name: String,
}

impl HttpService for MyService {
    #[instrument]
    async fn request(&self, _route: &str, req: &HttpRequest, _body: &mut dyn HttpRead) -> HttpResult {
        // Get first segment of the User-Agent
        let user = req.get_header("User-Agent").and_then(|a| a.split(' ').next()).unwrap_or("stranger");
        let name = &self.name;

        let greeting = format!("{name}: Krif voth ahkrin, {user}!\n");
        Ok(res::text(greeting))
    }
}

fn main() -> io::Result<()> {
    dhttp::tokio_rt()?.block_on(http_main())
}

#[instrument]
async fn http_main() -> io::Result<()> {
    let mut server = HttpServer::new();
    let name = "DrakoHTTP".to_string();
    server.service(MyService { name });

    dhttp::serve_tcp("[::]:8080", server).await
}
