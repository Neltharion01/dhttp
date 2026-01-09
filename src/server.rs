//! HTTP server

use std::io;
use std::sync::Arc;
use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::{BufReader, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpSocket;
use socket2::SockRef;

use crate::h1::{self, HttpRequestError};
use crate::reqres::{HttpRequest, StatusCode};
use crate::core::{HttpService, HttpServiceRaw, HttpErrorHandler, HttpErrorType, HttpLogger};
use crate::core::connection::{HttpConnection, EmitContinue};
use crate::services::{DefaultService, DefaultLogger, ErrorPageHandler};
use crate::util::future::Or;

const DEFAULT_MAX_HEADERS_SIZE: u64 = 65536; // 64KB

/// An HTTP/1.1 server
pub struct HttpServer {
    pub name: String,
    pub max_headers_size: u64,
    pub service: Box<dyn HttpServiceRaw>,
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
    async fn handle_connection(&self, mut conn: impl HttpConnection) -> io::Result<()> {
        let mut connection_close = false;
        while !connection_close {
            let req = h1::read((&mut conn).take(self.max_headers_size)).await;
            if let Err(err) = req {
                if let HttpRequestError::Io(err) = err {
                    // IO errors should not be handler
                    return Err(err);
                } else {
                    // Could not parse request, return Bad request
                    let res = self.error_handler.plain_code(StatusCode::BAD_REQUEST);
                    h1::send(&HttpRequest::default(), res, &mut conn).await?;
                    return conn.shutdown().await;
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
                h1::send(&req, res, &mut conn).await?;
                return conn.shutdown().await;
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
            let mut res = match self.service.filter_raw(&req.route, &req) {
                Ok(()) => self.service.request_raw(&req.route, &req, &mut body).await,
                Err(err) => Err(err),
            };

            if let Ok(res) = &res {
                // Log request+response with our defined logger
                self.logger.log(&req, res);
            } else if let Err(err) = res {
                // Response is Err, should be handled with defined error handler
                let mut handled = match err.error_type() {
                    // IO error
                    HttpErrorType::Fatal => return conn.shutdown().await,
                    // Status code
                    HttpErrorType::Hidden => self.error_handler.plain_code(err.status_code()),
                    // Error with description
                    HttpErrorType::User => self.error_handler.error(&req, err.as_ref()),
                };
                // Always use the original status code in the error response (connection handler sets this)
                handled.code = err.status_code();
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
            // - HTTP/1.1 but client didn't add `Connection: keep-alive`
            if body.conn.limit() != 0 || req.version.is(1, 0) {
                res.add_header("Connection", "close");
                connection_close = true;
            } else if req.version.major == 1 {
                // add keep-alive only if client wants it too
                if req.cmp_header("Connection", "keep-alive") {
                    res.add_header("Connection", "keep-alive");
                } else {
                    res.add_header("Connection", "close");
                    connection_close = true;
                }
            }

            // Now, send the response
            h1::send(&req, res, &mut conn).await?;
        }
        // Loop ended, we close the connection now
        conn.shutdown().await
    }
}

/// Starts handling connections on a given [`HttpServer`], without TLS
pub async fn serve_tcp(addr: &str, server: impl Into<Arc<HttpServer>>) -> io::Result<()> {
    let addr: SocketAddr = addr.parse().map_err(io::Error::other)?;

    let sock = match addr {
        SocketAddr::V4(_) => TcpSocket::new_v4()?,
        SocketAddr::V6(_) => TcpSocket::new_v6()?,
    };

    if addr.is_ipv6() && addr.ip().is_unspecified() {
        // allows to use [::] for both ipv4 and ipv6 on windows
        SockRef::from(&sock).set_only_v6(false)?;
    }

    // https://github.com/tokio-rs/mio/blob/b0578c2d166c2ebc78dfd5f70395591351ba8dde/src/net/tcp/listener.rs#L73
    // TL;DR socket is active some time after closing and you can't rebind it even if you have exited
    #[cfg(not(windows))]
    sock.set_reuseaddr(true)?;
    // already buffered
    sock.set_nodelay(true)?;

    sock.bind(addr)?;

    let tcp = sock.listen(128)?;
    let server = server.into();
    let mut err_shown = false;
    loop {
        // This way, shutdown is handled gracefully
        let result = Or::new(tcp.accept(), tokio::signal::ctrl_c()).await;
        if result.is_err() { break; }

        match result.unwrap() {
            Ok((conn, _addr)) => {
                err_shown = false;
                let server2 = Arc::clone(&server);
                tokio::spawn(async move {
                    // ignore network errors
                    let _ = server2.handle_connection(BufReader::new(conn)).await;
                });
            }
            Err(e) => {
                // this may fire when fd limit is exhausted
                if !err_shown {
                    println!("DrakoHTTP critical error: connection not accepted: {e}");
                    err_shown = true;
                }
                let d = Duration::from_millis(100);
                tokio::time::sleep(d).await;
            }
        };
    }

    Ok(())
}

/// Builds the tokio runtime
///
/// This function is a simple replacement for `#[tokio::main]` that does not use macros
pub fn tokio_rt() -> io::Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
}
