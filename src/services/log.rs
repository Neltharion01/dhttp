//use chrono_lite::{Tm, time, localtime};
use crate::core::{HttpLogger, HttpError};
use crate::reqres::{HttpRequest, HttpResponse};
//use crate::util::escape;

/// Default logger implementation
pub struct DefaultLogger;
impl DefaultLogger {
/*    fn format(&self, req: &HttpRequest, res: &HttpResponse) -> String {
        let addr = req.addr;
        let method = escape::control_sequences(req.method.as_str());
        let route = escape::control_sequences(&req.route);

        let code = res.code;
        let desc = code.as_str();

        let Tm { tm_mday, tm_mon, tm_year, tm_hour, tm_min, tm_sec, .. } = localtime(time()).expect("date out of range");
        let year = tm_year + 1900;
        let month = tm_mon + 1;
        let date = format_args!("{tm_hour:02}:{tm_min:02}:{tm_sec:02} {tm_mday:02}-{month:02}-{year}");

        // User-Agents are long, print only first segment
        let agent = req.get_header("User-Agent").and_then(|a| a.split(' ').next()).unwrap_or("-");
        let agent = escape::control_sequences(agent);
        format!("[{date}] {addr} {agent} {method} {route} -> {code} {desc}")
    }*/
}

#[allow(unused_variables)]
impl HttpLogger for DefaultLogger {
    fn log(&self, req: &HttpRequest, res: &HttpResponse) {
//        println!("{}", self.format(req, res));
    }

    fn err(&self, req: &HttpRequest, res: &HttpResponse, error: &dyn HttpError) {
//        println!("{} ({}: {})", self.format(req, res), error.name(), error);
    }
}
