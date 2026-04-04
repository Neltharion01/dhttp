//! Utilities to format an HTTP date
//! # Example
//! ```
//! # use dhttp::util::httpdate;
//! # use dhttp::reqres::HttpResponse;
//! # let mut res = HttpResponse::new();
//! let your_time;
//! # your_time = std::fs::metadata("src/util/httpdate.rs").unwrap().modified().unwrap();
//! if let Some(date) = httpdate::from_systime(your_time) {
//!     res.add_header("Last-Modified", date);
//! }
//! ```

use std::time::{SystemTime, UNIX_EPOCH};

use chrono_lite::{time_t, Tm, gmtime, timegm, time};

const WEEKDAYS: &[&str] = &["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: &[&str] = &["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

fn httpdate(tm: Tm) -> String {
    let Tm { tm_wday, tm_mday, tm_mon, tm_year, tm_hour, tm_min, tm_sec, .. } = tm;
    let weekday = WEEKDAYS[tm_wday as usize];
    let month = MONTHS[tm_mon as usize];
    let year = tm_year + 1900;
    // example output: Tue, 25 Feb 2025 21:05:51 GMT
    format!("{weekday}, {tm_mday} {month} {year} {tm_hour:02}:{tm_min:02}:{tm_sec:02} GMT")
}

/// Formats an HTTP date from a [`SystemTime`]
///
/// Returns `None` when date formatting fails (i. e. when provided timestamp was invalid)
pub fn from_systime(systime: SystemTime) -> Option<String> {
    let time = systime.duration_since(UNIX_EPOCH).ok()?.as_secs();
    let tm = gmtime(time as time_t)?;

    Some(httpdate(tm))
}

/// Returns the current time in an HTTP date
///
/// May return `None` on Windows after 31 Dec, 3000
pub fn now() -> Option<String> {
    let tm = gmtime(time())?;
    Some(httpdate(tm))
}

/// Parses an HTTP date
pub fn parse(mut date: &str) -> Option<time_t> {
    // It looks terrible, I know
    // Could be better if I had sscanf in Rust
    date = date.get(5..)?; // "Sat, "

    let mdaylen = if date.chars().nth(1)? == ' ' { 1 } else { 2 };
    let tm_mday = date.get(0..mdaylen)?.parse().ok()?; // "03"
    date = date.get(mdaylen+1..)?; // "03 "

    let month = date.get(0..3)?; // "Jan"
    let month = MONTHS.iter().position(|&i| i == month)?;
    date = date.get(4..)?; // "Jan "
    let year: i32 = date.get(0..4)?.parse().ok()?; // "2026"
    date = date.get(5..)?; // "2026 "
    let tm_hour = date.get(0..2)?.parse().ok()?; // "17"
    date = date.get(3..)?; // "17:"
    let tm_min = date.get(0..2)?.parse().ok()?; // "49"
    date = date.get(3..)?; // "49:"
    let tm_sec = date.get(0..2)?.parse().ok()?; // "29"
    date = date.get(3..)?; // "29 "
    if date != "GMT" { return None; } // "GMT"

    let tm = Tm {
        tm_year: year - 1900,
        tm_mon: month as i32,
        tm_mday,
        tm_hour,
        tm_min,
        tm_sec,
        ..Default::default()
    };
    timegm(tm)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_httpdate() {
        assert_eq!("Wed, 26 Feb 2025 22:10:59 GMT", &httpdate(gmtime(1740607859).unwrap()));
    }

    #[test]
    fn test_parse() {
        assert_eq!(1767462569, parse("Sat, 03 Jan 2026 17:49:29 GMT").unwrap());
        assert_eq!(1767484771, parse("Sat, 3 Jan 2026 23:59:31 GMT").unwrap());
    }

    #[test]
    fn test_panic() {
        assert_eq!(None, parse("🐉🐉🐉🐉🐉"));
    }
}
