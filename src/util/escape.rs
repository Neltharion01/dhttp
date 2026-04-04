use std::fmt::Write;

/// Escapes invalid UTF-8 sequences in a byte string (instead of replacing with U+FFFD)
pub(crate) fn to_utf8(s: &[u8]) -> String {
    let mut out = String::with_capacity(s.len());
    for chunk in s.utf8_chunks() {
        for ch in chunk.valid().chars() {
            // also filter out ASCII control characters
            if ch < ' ' {
                write!(&mut out, "\\x{:02x}", ch as u8).unwrap();
            } else {
                out.push(ch);
            }
        }
        for byte in chunk.invalid() {
            write!(&mut out, "\\x{:02x}", byte).unwrap();
        }
    }
    out
}

/// Escapes ASCII control sequences to avoid messing up your terminal
pub(crate) fn control_sequences(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch < ' ' {
            write!(&mut out, "\\x{:02x}", ch as u8).unwrap();
        } else {
            out.push(ch);
        }
    }
    out
}
