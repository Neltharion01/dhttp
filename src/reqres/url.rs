use percent_encoding_lite::{Bitmask, is_encoded, encode};

// Reason for this type - we want to pass it around HTTP/1.1 headers and HTML tags
// Also, Bitmask::URI allows '

/// Valid URL string that only contains allowed characters (as in [`Bitmask::URI`])
#[derive(Default, Debug, Clone)]
pub struct Url(pub(crate) String);

impl Url {
    /// Constructs a new URL and checks if it is valid
    pub fn new(s: impl Into<String>) -> Option<Url> {
        let s = s.into();
        let mask = Bitmask::URI.add(b'%');
        if !is_encoded(&s, mask) {
            return None;
        }
        Some(Url(s))
    }

    /// Encodes given string into a URL
    pub fn encode(s: &str) -> Url {
        Url(encode(s, Bitmask::URI))
    }

    /// Returns contained URL string
    pub fn get(&self) -> &str {
        &self.0
    }
}
