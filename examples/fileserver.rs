use std::io;
use std::path::Path;
use std::fmt::Write;

use dhttp::prelude::*;
use dhttp::reqres::res;
use dhttp::util::path;
use tokio::fs;

struct FileServer;
impl HttpService for FileServer {
    async fn request(&self, route: &str, req: &HttpRequest, _body: &mut dyn HttpRead) -> HttpResult {
        let path = path::sanitize(route)?;

        let metadata = fs::metadata(&path).await?;

        if metadata.is_dir() {
            Ok(res::html(req, list_dir(route, &path).await?))
        } else {
            Ok(res::file(req, &path).await?)
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

async fn list_dir(route: &str, path: &Path) -> io::Result<String> {
    let mut out = indoc::formatdoc!(r#"
        <!DOCTYPE html>
        <html>
        <head>
        <meta name="viewport" content="width=device-width">
        <title>Listing of {route}</title>
        </head>
        <body>
        <h1>Listing of {route}</h1>
    "#);

    let mut entries = fs::read_dir(&path).await?;
    while let Some(file) = entries.next_entry().await? {
        let path = path::encode(&file.path());
        // TODO this is broken on windows
        let mut path = format!("/{}", path.trim_start_matches("./").trim_start_matches(".%5C"));
        let filetype = file.file_type().await?;
        if filetype.is_dir() || filetype.is_symlink() {
            path.push('/');
        }
        write!(&mut out, "<a href='{}'>{}</a><br>\n", path, file.file_name().display()).unwrap();
    }
    out.push_str("</body>\n</html>\n");
    Ok(out)
}

async fn http_main() -> io::Result<()> {
    let mut server = HttpServer::new();
    server.service(FileServer);
    dhttp::serve_tcp("[::]:8080", server).await
}

fn main() -> io::Result<()> {
    dhttp::tokio_rt()?.block_on(http_main())
}
