//! Server Sent Events (SSE)
//! # Example
//! ```
//! use dhttp::reqres::res;
//!
//! # use dhttp::reqres::sse::{HttpSse, HttpSseEvent};
//! struct MyEvents;
//! impl HttpSse for MyEvents {
//!     async fn next(&mut self) -> Option<HttpSseEvent> {
//!         Some(HttpSseEvent::new("hello world"))
//!     }
//! }
//! # use dhttp::core::{HttpService, HttpResult};
//! # use dhttp::reqres::HttpRequest;
//! # use dhttp::core::connection::HttpRead;
//! struct MyService;
//! impl HttpService for MyService {
//!     async fn request(&self, _route: &str, _req: &HttpRequest, _body: &mut dyn HttpRead) -> HttpResult {
//!         Ok(res::sse(MyEvents))
//!     }
//! }
//! ```

use std::fmt::Write;
use std::pin::Pin;

pub struct HttpSseEvent(pub(crate) String);

fn add_data(event: &mut String, data: &str) {
    for line in data.split('\n') {
        writeln!(event, "data: {}", line).unwrap();
    }
    event.push('\n');
}

impl HttpSseEvent {
    pub fn new(data: &str) -> HttpSseEvent {
        let mut event = String::new();
        add_data(&mut event, data);
        HttpSseEvent(event)
    }

    pub fn named(name: &str, data: &str) -> HttpSseEvent {
        let mut event = format!("event: {}\n", name.replace('\n', ""));
        add_data(&mut event, data);
        HttpSseEvent(event)
    }
}

/// SSE stream
///
/// Can be used through [`res::sse`]
///
/// [`res::sse`]: crate::reqres::res::sse
#[doc(alias = "EventSource")]
pub trait HttpSse: Send + 'static {
    /// Produces a new event or `None` if there are no more events
    ///
    /// Equivalent signature: `async fn next(&mut self) -> Option<HttpSseEvent>`
    fn next(&mut self) -> impl Future<Output = Option<HttpSseEvent>> + Send;
}

/// Dyn version of [`HttpSse`]
pub trait HttpSseRaw: Send {
    /// Dyn version of `next`
    fn next_raw<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = Option<HttpSseEvent>> + Send + 'a>>;
}

impl<T: HttpSse> HttpSseRaw for T {
    fn next_raw<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = Option<HttpSseEvent>> + Send + 'a>> {
        Box::pin(self.next())
    }
}
