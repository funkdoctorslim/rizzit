//! # 👑 rizzit 👑
//! The GOATED video/torrent filename parser written in pure, unadulterated Rust.
//!
//! `rizzit` parses torrent and video file names with sub-microsecond speed into structured
//! metadata structs (`TorrentInfo`), including title, year, season, episode, video/audio codecs,
//! resolution, source, language, and release group.

pub mod token;
pub mod patterns;
pub mod rules;
pub mod types;

pub use rules::{parse, parse_with_options, TorrentInfo};
pub use token::{Bracket, Category, Token};
pub use types::{ParseOptions, Parser, Resolution, Source, VideoCodec};

impl TorrentInfo {
    /// Parse a torrent filename into a [`TorrentInfo`] struct.
    #[must_use]
    pub fn parse<S: AsRef<str>>(filename: S) -> Self {
        parse(filename)
    }

    /// Serialize the parsing results to a JSON string.
    ///
    /// # Errors
    /// Returns a [`serde_json::Error`] if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serialize the parsing results to a pretty-printed JSON string.
    ///
    /// # Errors
    /// Returns a [`serde_json::Error`] if serialization fails.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl std::str::FromStr for TorrentInfo {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(parse(s))
    }
}
