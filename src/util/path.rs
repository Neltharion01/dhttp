//! Path utilities

use std::path::{Path, PathBuf};
use std::error::Error;
use std::fmt;
#[cfg(unix)]
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

use percent_encoding_lite::Bitmask;

use crate::core::{HttpError, HttpErrorType};
use crate::reqres::StatusCode;

/// URL-decodes and converts request route into a relative path and checks its safety.
///
/// See [`DangerousPathError`] for details of these checks
pub fn sanitize(route: &str) -> Result<PathBuf, DangerousPathError> {
    let decoded = percent_encoding_lite::decode(route);
    // Why from_utf8? Long story short, we can't SAFELY construct an OsStr from WTF-8 bytes
    // We for sure can roundtrip through OsString::from_wide... only to encode it into WTF-8 again
    // WTF, std?
    #[cfg(windows)]
    return sanitize_win(str::from_utf8(&decoded).map_err(|_| DangerousPathError::InvalidCharacters)?);
    #[cfg(unix)]
    return sanitize_unix(&decoded);
    #[cfg(target_os = "cygwin")] {
        not_implemented("This function assumes unix paths on unix targets. Cygwin uses windows paths and so this code has to be refactored to support it");
    }
    #[cfg(not(any(windows, unix)))]
    not_implemented
}

// unfortunately we need separate functions for windows/unix because windows one uses &str
#[cfg(any(unix, test))]
fn sanitize_unix(route: &[u8]) -> Result<PathBuf, DangerousPathError> {
    if route.contains(&0) { return Err(DangerousPathError::InvalidCharacters); }

    let mut out = PathBuf::new();
    for segment in route.split(|&c| c == b'/') {
        if segment.is_empty() || segment == b"." { continue; }
        if segment == b".." { return Err(DangerousPathError::DangerousPath); }
        out.push(OsStr::from_bytes(segment));
    }

    if out.as_os_str().is_empty() { out.push("."); }
    Ok(out)
}

#[cfg(any(windows, test))]
fn sanitize_win(route: &str) -> Result<PathBuf, DangerousPathError> {
    if route.contains(|c| c < ' ') { return Err(DangerousPathError::InvalidCharacters); }
    // windows invalid characters except path separators '/' '\\'
    // out of all these ':' is most important because it rejects drive letters
    const INVALID_CHARS: &[char] = &['<', '>', ':', '"', '|', '?', '*'];
    if route.contains(INVALID_CHARS) { return Err(DangerousPathError::InvalidCharacters); }

    let mut out = PathBuf::new();
    for segment in route.split(['/', '\\']) {
        if segment.is_empty() || segment == "." { continue; }
        if segment == ".." { return Err(DangerousPathError::DangerousPath); }
        out.push(segment);
    }

    if out.as_os_str().is_empty() { out.push("."); }
    Ok(out)
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum DangerousPathError {
    /// Path contains dangerous segments (`..` and drive letters on Windows)
    DangerousPath,
    /// Path was either invalid UTF-8 (only on Windows), file name or contained forbidden characters:
    /// - `\0` on unix
    /// - 0-31 and `<>:"/\|?*` on Windows
    InvalidCharacters,
}

impl fmt::Display for DangerousPathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DangerousPathError::DangerousPath => f.write_str("path contains `..` or drive letters"),
            DangerousPathError::InvalidCharacters => f.write_str("path contains forbidden characters"),
        }
    }
}

impl Error for DangerousPathError {}
impl HttpError for DangerousPathError {
    fn error_type(&self) -> HttpErrorType { HttpErrorType::Hidden }
    fn status_code(&self) -> StatusCode { StatusCode::BAD_REQUEST }
}

/// Performs URL encoding for a given [`Path`] (lossy on Windows)
pub fn encode(path: &Path) -> String {
    #[cfg(windows)]
    return percent_encoding_lite::encode(&path.to_string_lossy(), Bitmask::PATH);
    #[cfg(unix)]
    return percent_encoding_lite::encode(path.as_os_str().as_bytes(), Bitmask::PATH);
    #[cfg(not(any(windows, unix)))]
    not_implemented
}

// TODO: test C:file on actix and " .." on dhttp
#[cfg(test)]
mod tests {
    use super::{sanitize_win, sanitize_unix, DangerousPathError};
    use DangerousPathError::*;
    #[test]
    fn win() {
        assert_eq!(sanitize_win("/.."), Err(DangerousPath));
        assert_eq!(sanitize_win("/\\..\\"), Err(DangerousPath));
        assert_eq!(sanitize_win("Dir\\.."), Err(DangerousPath));
        assert_eq!(sanitize_win("/C:/Windows"), Err(InvalidCharacters));
        assert_eq!(sanitize_win("/C:\\Windows"), Err(InvalidCharacters));
        assert_eq!(sanitize_win("C:file.txt"), Err(InvalidCharacters));
        assert_eq!(sanitize_win("/\\\\?\\"), Err(InvalidCharacters));
        assert_eq!(sanitize_win("/\0"), Err(InvalidCharacters));
        assert!(sanitize_win("/status.json").is_ok());
        assert!(sanitize_win("/files/examples/fileserver.rs").is_ok());
        assert!(sanitize_win("/F1SHMSOaYAA1M2G.jpeg").is_ok());
    }
    #[test]
    fn unix() {
        assert_eq!(sanitize_unix(b"/.."), Err(DangerousPath));
        assert_eq!(sanitize_unix(b"../"), Err(DangerousPath));
        assert_eq!(sanitize_unix(b".."), Err(DangerousPath));
        assert_eq!(sanitize_unix(b"/dir/.."), Err(DangerousPath));
        assert_eq!(sanitize_unix(b"/\0"), Err(InvalidCharacters));
        assert_eq!(sanitize_unix(b"/nulls\0instead\0of\0spaces"), Err(InvalidCharacters));
        assert!(sanitize_unix(b"<>:\"/\\|?*").is_ok());
        assert!(sanitize_unix(b"/just/a/path").is_ok());
        assert!(sanitize_unix(b"/dev/sda").is_ok());
        assert!(sanitize_unix(b"\\..\\This is a filename").is_ok());
        assert!(sanitize_unix(b"/C:/Windows").is_ok());
    }
}
