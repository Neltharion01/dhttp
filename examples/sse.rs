use std::io;
use std::time::Duration;

use dhttp::prelude::*;
use dhttp::reqres::res;

struct SseService;

impl HttpService for SseService {
    async fn request(&self, _route: &str, _req: &HttpRequest, _body: &mut dyn HttpRead) -> HttpResult {
        Ok(res::sse(SseHandler { counter: 0 }))
    }
}

struct SseHandler {
    counter: u8,
}

impl HttpSse for SseHandler {
    async fn next(&mut self) -> Option<HttpSseEvent> {
        self.counter += 1;

        tokio::time::sleep(Duration::from_millis(1000)).await;

        if self.counter == 1 {
            return Some(HttpSseEvent::named("warning", "Here be dragons"));
        } else if self.counter == 10 {
            // Server closes connection once this function
            // returns None
            return None;
        } else {
            return Some(HttpSseEvent::new(&self.counter.to_string()));
        }
    }
}

fn main() -> io::Result<()> {
    dhttp::tokio_rt()?.block_on(http_main())
}

async fn http_main() -> io::Result<()> {
    let mut server = HttpServer::new();
    server.service(SseService);

    dhttp::serve_tcp("[::]:8080", server).await
}
