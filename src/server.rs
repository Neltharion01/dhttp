//! HTTP server

use std::io::{self, Read};
use std::sync::Arc;
use std::time::Duration;

use may::go;
use may::net::TcpListener;

use crate::h1::{self, HttpRequestError};
use crate::reqres::{HttpRequest, StatusCode};
use crate::core::{HttpService, HttpErrorHandler, HttpErrorType, HttpLogger};
use crate::core::connection::{HttpConnection, EmitContinue};
use crate::services::{DefaultService, DefaultLogger, ErrorPageHandler};

const DEFAULT_MAX_HEADERS_SIZE: u64 = 65536; // 64KB

/// An HTTP/1.1 server
pub struct HttpServer {
    pub name: String,
    pub max_headers_size: u64,
    pub service: Box<dyn HttpService>,
    pub error_handler: Box<dyn HttpErrorHandler>,
    pub logger: Box<dyn HttpLogger>,
}

impl HttpServer {
    pub fn new() -> HttpServer {
        HttpServer {
            name: "DrakoHTTP".to_string(),
            max_headers_size: DEFAULT_MAX_HEADERS_SIZE,
            service: Box::new(DefaultService),
            error_handler: Box::new(ErrorPageHandler { name: "DrakoHTTP".to_string() }),
            logger: Box::new(DefaultLogger),
        }
    }

    pub fn service(&mut self, service: impl HttpService) -> &mut Self {
        self.service = Box::new(service);
        self
    }

    pub fn error_handler(&mut self, error_handler: impl HttpErrorHandler) -> &mut Self {
        self.error_handler = Box::new(error_handler);
        self
    }

    pub fn logger(&mut self, logger: impl HttpLogger) -> &mut Self {
        self.logger = Box::new(logger);
        self
    }
}

impl Default for HttpServer {
    fn default() -> HttpServer {
        HttpServer::new()
    }
}

impl HttpServer {
    fn handle_connection(&self, mut conn: impl HttpConnection) -> io::Result<()> {
        let mut buf = Vec::with_capacity(1024);
        let mut connection_close = false;
        while !connection_close {
            buf.clear();
            let req = h1::read(&mut buf, Read::by_ref(&mut conn).take(self.max_headers_size));
            if let Err(err) = req {
                if let HttpRequestError::Io(err) = err {
                    // IO errors should not be handled
                    return Err(err);
                } else {
                    // Could not parse request, return Bad request
                    let res = self.error_handler.plain_code(StatusCode::BAD_REQUEST);
                    h1::send(HttpRequest::default(), res, &mut conn)?;
                    return conn.shutdown();
                }
            }
            // Request is Ok
            let mut req = req.unwrap();

            // Address has to be set by the connection handler
            if let Ok(addr) = conn.getpeername() {
                req.addr = addr.ip().to_canonical();
            }

            // HTTP/2 prior knowledge headers look like `PRI * HTTP/2.0`
            // These connections are not supported
            if req.version.major != 1 {
                let res = self.error_handler.plain_code(StatusCode::HTTP_VERSION_NOT_SUPPORTED);
                h1::send(req, res, &mut conn)?;
                return conn.shutdown();
            }

            // Before starting file upload, curl expects server to send `100 Continue` response
            // Otherwise, it will wait for a timeout
            // This adapter echoes `100 Continue` when service starts reading the body
            // (meaning, that service has accepted it)
            let mut body = EmitContinue {
                conn: (&mut conn).take(req.len),
                to_send: b"",
            };
            if req.cmp_header("Expect", "100-continue") {
                body.to_send = b"HTTP/1.1 100 Continue\r\n\r\n";
            }

            // Future TODO: HTTP/1.1 connection handler has a lot of hardcoded functionality
            // that still applies to HTTP/2 and QUIC. Some logic here could be separated

            // Before executing the service, we have to check if request is compatible
            // This is connection handler's responsibility
            let mut res = match self.service.filter(&req.route, &req) {
                Ok(()) => self.service.request(&req.route, &req, &mut body),
                Err(err) => Err(err),
            };

            if let Ok(res) = &res {
                // Log request+response with our defined logger
                self.logger.log(&req, res);
            } else if let Err(err) = res {
                // Response is Err, should be handled with defined error handler
                let handled = match err.error_type() {
                    // IO error
                    HttpErrorType::Fatal => return conn.shutdown(),
                    // Status code
                    HttpErrorType::Hidden => self.error_handler.plain_code(err.status_code()),
                    // Error with description
                    HttpErrorType::User => self.error_handler.error(&req, err.as_ref()),
                };
                // Log the error
                match err.error_type() {
                    HttpErrorType::Fatal => unreachable!(),
                    HttpErrorType::Hidden => self.logger.log(&req, &handled),
                    HttpErrorType::User => self.logger.err(&req, &handled, err.as_ref()),
                };
                res = Ok(handled);
            }
            // Response is Ok
            let mut res = res.unwrap();

            // Add our server name
            if !self.name.is_empty() {
                res.add_header("Server", &self.name);
            }

            // Stop pipelining if:
            // - service didn't consume the body completely
            // - HTTP/1.0 (doesn't support pipelining)
            if body.conn.limit() != 0 || req.version.is(1, 0) {
                res.add_header("Connection", "close");
                connection_close = true;
            } else if req.version.major == 1 {
                if !req.has_header("Connection") || req.cmp_header("Connection", "keep-alive") {
                    res.add_header("Connection", "keep-alive");
                } else {
                    res.add_header("Connection", "close");
                    connection_close = true;
                }
            }

            // Now, send the response
            h1::send(req, res, &mut conn)?;
        }
        // Loop ended, we close the connection now
        conn.shutdown()
    }
}

/// Starts handling connections on a given [`HttpServer`], without TLS
pub fn serve_tcp(addr: &str, server: impl Into<Arc<HttpServer>>) -> io::Result<()> {
    let tcp = TcpListener::bind(addr)?;
    let server = server.into();
    let mut err_shown = false;
    loop {
        match tcp.accept() {
            Ok((conn, _addr)) => {
                err_shown = false;
                let server2 = Arc::clone(&server);
                go!(move || {
                    // ignore network errors
                    let _ = server2.handle_connection(conn);
                });
            }
            Err(e) => {
                // this may fire when fd limit is exhausted
                if !err_shown {
                    println!("DrakoHTTP critical error: connection not accepted: {e}");
                    err_shown = true;
                }
                let d = Duration::from_millis(100);
                may::coroutine::sleep(d);
            }
        };
    }
}
