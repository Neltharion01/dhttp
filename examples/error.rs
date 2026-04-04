use std::io;
use std::fmt;
use std::error::Error;

use dhttp::prelude::*;

struct ErrorService;

impl HttpService for ErrorService {
    async fn request(&self, _route: &str, _req: &HttpRequest, _body: &mut dyn HttpRead) -> HttpResult {
        Err(MyError.into())
    }
}

#[derive(Debug)]
struct MyError;

impl fmt::Display for MyError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("An example error")
    }
}

impl Error for MyError {}
impl HttpError for MyError {}

fn main() -> io::Result<()> {
    dhttp::tokio_rt()?.block_on(http_main())
}

async fn http_main() -> io::Result<()> {
    let mut server = HttpServer::new();
    server.service(ErrorService);

    dhttp::serve_tcp("[::]:8080", server).await
}
