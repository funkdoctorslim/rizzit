pub mod token;
pub mod patterns;
pub mod rules;

pub use rules::{TorrentInfo, parse};

impl TorrentInfo {
    /// Parse a torrent filename into a TorrentInfo struct.
    pub fn parse(filename: &str) -> Self {
        parse(filename)
    }

    /// Serialize the parsing results to a JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serialize the parsing results to a pretty JSON string.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
