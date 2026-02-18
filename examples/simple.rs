use std::io;

use parseagent::Guess;

use dhttp::prelude::*;
use dhttp::reqres::res;

struct MyService {
    name: String,
}

impl HttpService for MyService {
    async fn request(&self, _route: &str, req: &HttpRequest, _body: &mut dyn HttpRead) -> HttpResult {
        // Format the User-Agent
        let user = req.get_header("User-Agent").unwrap_or_default();
        let user = Guess::new(user);

        let greeting = format!("Hello {}! Powered by {}\n", user, &self.name);
        Ok(res::text(greeting))
    }
}

fn main() -> io::Result<()> {
    dhttp::tokio_rt()?.block_on(http_main())
}

async fn http_main() -> io::Result<()> {
    let mut server = HttpServer::new();
    let name = "DrakoHTTP".to_string();
    server.service(MyService { name });

    dhttp::serve_tcp("[::]:8080", server).await
}
