use std::io::{Seek, SeekFrom};
use std::path::Path;
use std::ffi::OsStr;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::UNIX_EPOCH;
use std::fs::File;

use crate::core::HttpResult;
use crate::reqres::{HttpRequest, HttpResponse, HttpHeader, HttpBody, StatusCode};
use crate::util::httpdate;

/// Responds with a file
pub fn file(req: &HttpRequest, name: &Path) -> HttpResult {
    let mut file = File::open(name)?;
    let metadata = file.metadata()?;
    let mut len = metadata.len();

    // becomes PARTIAL_CONTENT if range was served
    let mut code = StatusCode::OK;
    let content_type = get_content_type(name.extension()).unwrap_or_default().to_string();
    let mut headers = vec![];
    let mut body;

    // Last-Modified
    let time = metadata.modified().ok();
    if let Some(value) = time // fails when field not supported
        .and_then(httpdate::from_systime) // fails on overflow
    {
        headers.push(HttpHeader { name: "Last-Modified".to_string(), value });
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

            file.seek(SeekFrom::Start(start))?;
            len = end - start + 1;
            code = StatusCode::PARTIAL_CONTENT;
        } else {
            // we have to set Content-Range in case of error too but errors can't have headers in dhttp
            return Err(StatusCode::RANGE_NOT_SATISFIABLE.into());
        }
    }

    body = HttpBody::File { file, len };

    // If-Modified-Since🐛🐛🐛
    if let Some(time) = time
        && let Some(time) = time.duration_since(UNIX_EPOCH).ok()
        && let Some(if_modified_since) = req.get_header("If-Modified-Since")
        && let Some(parsed) = httpdate::parse(if_modified_since)
        && parsed >= time.as_secs() as i64
    {
        code = StatusCode::NOT_MODIFIED;
        body = HttpBody::Empty;
    }

    Ok(HttpResponse { code, headers, body, content_type })
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
