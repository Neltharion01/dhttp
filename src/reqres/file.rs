use std::io::SeekFrom;
use std::path::Path;
use std::ffi::OsStr;
use std::collections::HashMap;
use std::sync::LazyLock;

use tokio::io::AsyncSeekExt;
use tokio::fs::File;

use crate::core::HttpResult;
use crate::reqres::{HttpRequest, HttpResponse, HttpHeader, HttpBody, StatusCode};
use crate::util::httpdate;

/// Responds with a file
pub async fn file(req: &HttpRequest, name: &Path) -> HttpResult {
    let mut file = File::open(name).await?;
    let metadata = file.metadata().await?;
    let mut len = metadata.len();

    // becomes PARTIAL_CONTENT if range was served
    let mut code = StatusCode::OK;
    let content_type = get_content_type(name.extension()).unwrap_or_default().to_string();

    // Last-Modified
    let mut headers = vec![];
    if let Ok(time) = metadata.modified() { // fails if field not supported
        if let Some(s) = httpdate::from_systime(time) { // fails on overflow
            headers.push(HttpHeader { name: "Last-Modified".to_string(), value: s });
        }
    }

    // Date
    if let Some(date) = httpdate::now() {
        headers.push(HttpHeader { name: "Date".to_string(), value: date });
    }

    // Advertise byte ranges support
    headers.push(HttpHeader {
        name: "Accept-Ranges".to_string(),
        value: "bytes".to_string(),
    });

    // Parse byte range request
    if let Some(range) = req.get_header("Range") {
        if let Some((start, mut end)) = parse_range(range) && start <= len && start <= end {
            end = end.min(len);

            headers.push(HttpHeader {
                name: "Content-Range".to_string(),
                value: format!("bytes {start}-{end}/{len}"),
            });

            file.seek(SeekFrom::Start(start)).await?;
            len = end - start + 1;
            code = StatusCode::PARTIAL_CONTENT;
        } else {
            // we have to set Content-Range in case of error too but errors can't have headers in dhttp
            return Err(StatusCode::RANGE_NOT_SATISFIABLE.into());
        }
    }

    Ok(HttpResponse {
        code,
        headers,
        body: HttpBody::File { file, len },
        content_type,
    })
}

fn parse_range(range: &str) -> Option<(u64, u64)> {
    let (start, end) = range.strip_prefix("bytes=")?.split_once('-')?;
    let start = if start.is_empty() { 0 } else { start.parse().ok()? };
    let end = if end.is_empty() { u64::MAX } else { end.parse().ok()? };
    Some((start, end))
}

// This is only for files loaded/previewed by web browser
static CONTENT_TYPES: LazyLock<HashMap<&'static OsStr, &'static str>> = LazyLock::new(|| HashMap::from([
    // text/application
    (os!("html"), "text/html"),
    (os!("htm"), "text/html"),
    (os!("css"), "text/css"),
    (os!("js"), "application/javascript"),
    (os!("txt"), "text/plain"),
    (os!("xml"), "text/xml"),
    (os!("json"), "application/json"),
    (os!("wasm"), "application/wasm"),
    // images
    (os!("png"), "image/png"),
    (os!("jpg"), "image/jpeg"),
    (os!("jpeg"), "image/jpeg"),
    (os!("webp"), "image/webp"),
    (os!("avif"), "image/avif"),
    (os!("jxl"), "image/jxl"),
    (os!("gif"), "image/gif"),
    (os!("svg"), "image/svg+xml"),
    (os!("svgz"), "image/svg+xml"),
    // videos
    (os!("mp4"), "video/mp4"),
    (os!("mkv"), "video/matroska"),
    (os!("webm"), "video/webm"),
    (os!("avi"), "video/x-msvideo"),
    (os!("m3u8"), "application/vnd.apple.mpegurl"),
    (os!("mov"), "video/quicktime"),
    // audio
    (os!("mp3"), "audio/mpeg"),
    (os!("ogg"), "audio/ogg"),
    (os!("m4a"), "audio/mp4"),
    // fonts
    (os!("woff"), "font/woff"),
    (os!("woff2"), "font/woff2"),
    // documents
    (os!("pdf"), "application/pdf"),
]));

fn get_content_type(ext: Option<&OsStr>) -> Option<&'static str> {
    CONTENT_TYPES.get(ext?.to_ascii_lowercase().as_os_str()).copied()
}

macro_rules! os {
    ($s:literal) => { OsStr::new($s) }
}
use os;
