use std::fmt;
use std::str::FromStr;

/// Video Codec enum representation for strongly-typed pattern matching.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum VideoCodec {
    H264,
    H265,
    AV1,
    VP9,
    VP8,
    VC1,
    Xvid,
    DivX,
    MPEG2,
    Other(String),
}

impl fmt::Display for VideoCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::H264 => write!(f, "H.264"),
            Self::H265 => write!(f, "H.265"),
            Self::AV1 => write!(f, "AV1"),
            Self::VP9 => write!(f, "VP9"),
            Self::VP8 => write!(f, "VP8"),
            Self::VC1 => write!(f, "VC-1"),
            Self::Xvid => write!(f, "Xvid"),
            Self::DivX => write!(f, "DivX"),
            Self::MPEG2 => write!(f, "MPEG-2"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

impl FromStr for VideoCodec {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        Ok(if lower.contains("av1") || lower.contains("av01") {
            Self::AV1
        } else if lower.contains("265") || lower.contains("hevc") {
            Self::H265
        } else if lower.contains("264") || lower.contains("h264") || lower.contains("avc") {
            Self::H264
        } else if lower.contains("vp9") {
            Self::VP9
        } else if lower.contains("vp8") {
            Self::VP8
        } else if lower.contains("vc-1") || lower.contains("vc1") {
            Self::VC1
        } else if lower.contains("xvid") {
            Self::Xvid
        } else if lower.contains("divx") {
            Self::DivX
        } else if lower.contains("mpeg") {
            Self::MPEG2
        } else {
            Self::Other(s.to_string())
        })
    }
}

/// Video Resolution enum representation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Resolution {
    R4320p,
    R2160p,
    R1440p,
    R1080p,
    R720p,
    R576p,
    R480p,
    Other(String),
}

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::R4320p => write!(f, "4320p"),
            Self::R2160p => write!(f, "2160p"),
            Self::R1440p => write!(f, "1440p"),
            Self::R1080p => write!(f, "1080p"),
            Self::R720p => write!(f, "720p"),
            Self::R576p => write!(f, "576p"),
            Self::R480p => write!(f, "480p"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

impl FromStr for Resolution {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        Ok(if lower.contains("8k") || lower.contains("4320") {
            Self::R4320p
        } else if lower.contains("4k") || lower.contains("2160") || lower.contains("uhd") {
            Self::R2160p
        } else if lower.contains("1440") || lower.contains("qhd") {
            Self::R1440p
        } else if lower.contains("1080") || lower.contains("fhd") {
            Self::R1080p
        } else if lower.contains("720") {
            Self::R720p
        } else if lower.contains("576") {
            Self::R576p
        } else if lower.contains("480") || lower.contains("sd") {
            Self::R480p
        } else {
            Self::Other(s.to_string())
        })
    }
}

/// Media Source enum representation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Source {
    Bluray,
    WebDl,
    WebRip,
    Web,
    Hdtv,
    Dvd,
    Cam,
    Telesync,
    Telecine,
    Remux,
    Other(String),
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bluray => write!(f, "Blu-ray"),
            Self::WebDl => write!(f, "WEB-DL"),
            Self::WebRip => write!(f, "WEBRip"),
            Self::Web => write!(f, "WEB"),
            Self::Hdtv => write!(f, "HDTV"),
            Self::Dvd => write!(f, "DVD"),
            Self::Cam => write!(f, "CAM"),
            Self::Telesync => write!(f, "Telesync"),
            Self::Telecine => write!(f, "Telecine"),
            Self::Remux => write!(f, "Remux"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

impl FromStr for Source {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        Ok(if lower.contains("bluray") || lower.contains("blu-ray") || lower.contains("bdrip") || lower.contains("brrip") {
            Self::Bluray
        } else if lower.contains("web-dl") || lower.contains("webdl") {
            Self::WebDl
        } else if lower.contains("webrip") || lower.contains("web-rip") {
            Self::WebRip
        } else if lower.contains("web") {
            Self::Web
        } else if lower.contains("hdtv") {
            Self::Hdtv
        } else if lower.contains("dvd") {
            Self::Dvd
        } else if lower.contains("cam") {
            Self::Cam
        } else if lower.contains("telesync") || lower.contains("ts") {
            Self::Telesync
        } else if lower.contains("telecine") || lower.contains("tc") {
            Self::Telecine
        } else if lower.contains("remux") {
            Self::Remux
        } else {
            Self::Other(s.to_string())
        })
    }
}

/// Configuration options for custom parsing behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOptions {
    /// Enable directory metadata inheritance for generic files (default: `true`).
    pub inherit_parent_dir: bool,
    /// Delimiterless mode for filenames with no spaces/dots (default: `true`).
    pub delimiterless_mode: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            inherit_parent_dir: true,
            delimiterless_mode: true,
        }
    }
}

/// Configurable Torrent/Video filename parser builder.
#[derive(Debug, Clone, Default)]
pub struct Parser {
    options: ParseOptions,
}

impl Parser {
    /// Creates a new `Parser` builder with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures whether to inherit parent/grandparent directory metadata for generic files.
    #[must_use]
    pub fn inherit_parent_dir(mut self, inherit: bool) -> Self {
        self.options.inherit_parent_dir = inherit;
        self
    }

    /// Configures whether to enable delimiterless mode for filenames with no spaces or dots.
    #[must_use]
    pub fn delimiterless_mode(mut self, enabled: bool) -> Self {
        self.options.delimiterless_mode = enabled;
        self
    }

    /// Parses a filename or path using the configured options.
    #[must_use]
    pub fn parse<S: AsRef<str>>(&self, filename: S) -> crate::TorrentInfo {
        crate::rules::parse_with_options(filename.as_ref(), &self.options)
    }
}
