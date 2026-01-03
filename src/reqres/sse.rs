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
use std::io;

use tokio::io::AsyncWriteExt;

use crate::core::connection::HttpConnection;
use crate::reqres::HttpUpgrade;

pub struct HttpSseEvent(String);

fn add_data(event: &mut String, data: &str) {
    for line in data.split('\n') {
        write!(event, "data: {}", line).unwrap();
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
    fn next(&mut self) -> impl Future<Output = Option<HttpSseEvent>> + Send;
}

impl<T: HttpSse> HttpUpgrade for T {
    async fn upgrade(&mut self, conn: &mut dyn HttpConnection) -> io::Result<()> {
        while let Some(event) = self.next().await {
            conn.write_all(event.0.as_bytes()).await?;
        }

        Ok(())
    }
}
