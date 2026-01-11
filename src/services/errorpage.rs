use tracing::instrument;

use crate::core::{HttpError, HttpErrorHandler};
use crate::reqres::{res, HttpRequest, HttpResponse, StatusCode};

fn error_page(code: u16, code_desc: &str, desc: &str, name: &str) -> String {
format!(r#"<!doctype html>
<html><title>{code} {code_desc}</title><meta name="viewport" content="width=device-width"><style>*{{font-family:sans-serif;color:#0e1219;border-color:#0e1219;background:#f9f9f9}}@media(prefers-color-scheme:dark){{*{{color:#47d8bb;border-color:#47d8bb;background:#0e1219}}}}h1{{margin:0;}}div{{position:fixed;top:50%;left:50%;transform:translate(-50%,-50%);padding:8px;border:4px solid}}</style><div>

<h1>    {code} {code_desc}    </h1>
        {desc}
<hr><!--------------------------->
<center>    {name}    </center>

</div></html>
"#)
}

/// Default error handler, shows a nice error page
#[derive(Debug)]
pub struct ErrorPageHandler {
    pub name: String,
}

impl HttpErrorHandler for ErrorPageHandler {
    fn error(&self, req: &HttpRequest, error: &dyn HttpError) -> HttpResponse {
        let code = error.status_code();
        let desc = error.http_description();
        res::html(req, &error_page(code.0, code.as_str(), &desc, &self.name))
    }

    #[instrument]
    fn plain_code(&self, code: StatusCode) -> HttpResponse {
        res::html(&HttpRequest::default(), &error_page(code.0, code.as_str(), "", &self.name))
    }
}
