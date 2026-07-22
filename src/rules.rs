use crate::token::{Bracket, Category, Token, tokenize};
use crate::patterns;
use crate::types::{ParseOptions, Resolution, Source, VideoCodec};
use std::collections::HashSet;

/// Parsed torrent/video metadata extracted from a filename or path.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TorrentInfo {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub season: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_codec: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_codec: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_channels: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
}

impl TorrentInfo {
    /// Returns `true` if the parsed item is a movie (has a release year, no season/episode numbers).
    #[must_use]
    pub fn is_movie(&self) -> bool {
        self.year.is_some() && self.season.is_none() && self.episode.is_none()
    }

    /// Returns `true` if the parsed item is a TV show (has season or episode numbers).
    #[must_use]
    pub fn is_tv_show(&self) -> bool {
        self.season.is_some() || self.episode.is_some()
    }

    /// Returns the primary (first) season number, if present.
    #[must_use]
    pub fn season_num(&self) -> Option<u32> {
        self.season.as_ref().and_then(|s| s.first().copied())
    }

    /// Returns the primary (first) episode number, if present.
    #[must_use]
    pub fn episode_num(&self) -> Option<u32> {
        self.episode.as_ref().and_then(|e| e.first().copied())
    }

    /// Returns `true` if this is a complete season pack (season is present, episode is `None`).
    #[must_use]
    pub fn is_season_pack(&self) -> bool {
        self.season.is_some() && self.episode.is_none()
    }

    /// Returns `true` if the release shows typical anime conventions.
    #[must_use]
    pub fn is_anime(&self) -> bool {
        if let Some(ref grp) = self.release_group {
            let grp_lower = grp.to_lowercase();
            if ["subsplease", "erai-raws", "vcb-studio", "beansub", "judas", "horriblesubs"].contains(&grp_lower.as_str()) {
                return true;
            }
        }
        if self.other.as_ref().map_or(false, |oth| oth.iter().any(|o| o.starts_with("CRC32:") || o == "Subbed" || o == "Dubbed" || o == "Hardsubbed")) {
            return true;
        }
        false
    }

    /// Returns the strongly-typed [`VideoCodec`] enum representation, if present.
    #[must_use]
    pub fn typed_video_codec(&self) -> Option<VideoCodec> {
        self.video_codec.as_deref().and_then(|s| s.parse().ok())
    }

    /// Returns the strongly-typed [`Resolution`] enum representation, if present.
    #[must_use]
    pub fn typed_resolution(&self) -> Option<Resolution> {
        self.resolution.as_deref().and_then(|s| s.parse().ok())
    }

    /// Returns the strongly-typed [`Source`] enum representation, if present.
    #[must_use]
    pub fn typed_source(&self) -> Option<Source> {
        self.source.as_deref().and_then(|s| s.parse().ok())
    }

    /// Formats a clean, human-readable title incorporating year or season/episode if available.
    #[must_use]
    pub fn full_title(&self) -> String {
        let mut result = self.title.clone();
        if let Some(ref s) = self.season {
            let season_str: String = s.iter().map(|n| format!("{:02}", n)).collect::<Vec<_>>().join("-");
            result.push_str(&format!(" S{}", season_str));
            if let Some(ref e) = self.episode {
                let ep_str: String = e.iter().map(|n| format!("{:02}", n)).collect::<Vec<_>>().join("-");
                result.push_str(&format!("E{}", ep_str));
            }
        } else if let Some(y) = self.year {
            result.push_str(&format!(" ({})", y));
        }
        result
    }

    /// Formats a standardized scene-style filename (e.g. `"The.Matrix.1999.1080p.BluRay.H.264-FGT.mkv"`).
    #[must_use]
    pub fn normalized_filename(&self) -> String {
        let mut parts = Vec::new();
        parts.push(self.title.replace(' ', "."));
        if let Some(y) = self.year {
            parts.push(y.to_string());
        }
        if let Some(s) = self.season_num() {
            if let Some(e) = self.episode_num() {
                parts.push(format!("S{:02}E{:02}", s, e));
            } else {
                parts.push(format!("S{:02}", s));
            }
        }
        if let Some(ref res) = self.resolution {
            parts.push(res.clone());
        }
        if let Some(ref src) = self.source {
            parts.push(src.replace(' ', "."));
        }
        if let Some(ref vc) = self.video_codec {
            parts.push(vc.replace(' ', "."));
        }
        
        let mut name = parts.join(".");
        if let Some(ref grp) = self.release_group {
            name.push('-');
            name.push_str(grp);
        }
        if let Some(ref ext) = self.container {
            name.push('.');
            name.push_str(ext);
        }
        name
    }
}

/// Parses a filename or full path into a `TorrentInfo` struct using default settings.
#[must_use]
pub fn parse<S: AsRef<str>>(filename: S) -> TorrentInfo {
    parse_with_options(filename.as_ref(), &ParseOptions::default())
}

/// Parses a filename or path into a `TorrentInfo` struct with custom options.
#[must_use]
pub fn parse_with_options(filename: &str, options: &ParseOptions) -> TorrentInfo {
    let segments: Vec<&str> = filename.split(['/', '\\'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if segments.is_empty() {
        return TorrentInfo::default();
    }

    let last_segment = segments.last().unwrap();
    let mut file_info = parse_segment(last_segment);

    if options.inherit_parent_dir && segments.len() > 1 {
        let is_generic = is_generic_file(&file_info.title, file_info.container.as_deref());
        for seg in segments.iter().rev().skip(1) {
            let seg_info = parse_segment(seg);
            
            if (is_generic || file_info.title.is_empty() || is_generic_word(&file_info.title) || file_info.title.to_lowercase() == "sample")
                && !seg_info.title.is_empty() && !is_generic_word(&seg_info.title) {
                    file_info.title = seg_info.title;
                }
            if file_info.year.is_none() {
                file_info.year = seg_info.year;
            }
            if file_info.season.is_none() {
                file_info.season = seg_info.season;
            }
            if file_info.episode.is_none() {
                file_info.episode = seg_info.episode;
            }
            if file_info.episode_title.is_none() {
                file_info.episode_title = seg_info.episode_title;
            }
            if file_info.resolution.is_none() {
                file_info.resolution = seg_info.resolution;
            }
            if file_info.source.is_none() {
                file_info.source = seg_info.source;
            }
            if file_info.video_codec.is_none() {
                file_info.video_codec = seg_info.video_codec;
            }
            if file_info.audio_codec.is_none() {
                file_info.audio_codec = seg_info.audio_codec;
            }
            if file_info.audio_channels.is_none() {
                file_info.audio_channels = seg_info.audio_channels;
            }
            if file_info.release_group.is_none() {
                file_info.release_group = seg_info.release_group;
            }
            if file_info.website.is_none() {
                file_info.website = seg_info.website;
            }
            if file_info.language.is_none() {
                file_info.language = seg_info.language;
            }
            
            if let Some(p_other) = seg_info.other {
                let mut merged_other = file_info.other.take().unwrap_or_default();
                for o in p_other {
                    if !merged_other.contains(&o) {
                        merged_other.push(o);
                    }
                }
                if !merged_other.is_empty() {
                    file_info.other = Some(merged_other);
                }
            }
        }
    }

    file_info
}

fn is_generic_file(title: &str, container: Option<&str>) -> bool {
    let t_lower = title.to_lowercase();
    let is_image_or_txt = if let Some(ext) = container {
        ["jpg", "jpeg", "png", "gif", "txt", "nfo", "srt"].contains(&ext.to_lowercase().as_str())
    } else {
        false
    };
    
    is_image_or_txt || t_lower == "sample" || t_lower == "preview" || t_lower == "featurette" || t_lower.is_empty() || is_generic_word(&t_lower)
}

fn is_generic_word(word: &str) -> bool {
    let w_lower = word.to_lowercase();
    let generic_words = [
        "sample", "info", "nfo", "rarbg", "cover", "poster", "fanart", "banner", "logo",
        "discart", "screenshot", "screen", "subs", "sub", "english", "sdh", "downloaded",
        "yts", "yify", "torrent", "website", "txt", "about", "readme", "read me", "read-me",
        "season 1", "season 2", "season 3", "season 01", "season 02", "season 03",
    ];
    generic_words.contains(&w_lower.as_str())
}

fn parse_segment(raw_filename: &str) -> TorrentInfo {
    // Reconstruct filename with dots at split points to provide word boundaries for delimiterless metadata runs
    let mut split_points = std::collections::BTreeSet::new();
    let split_regexes = [
        patterns::split_year(),
        patterns::split_res(),
        patterns::split_codec(),
        patterns::split_channels(),
        patterns::split_source(),
    ];
    for re in &split_regexes {
        for caps in re.captures_iter(raw_filename) {
            if let Some(m) = caps.get(1) {
                split_points.insert(m.start());
                split_points.insert(m.end());
            }
        }
    }
    
    let reconstructed_filename = if !split_points.is_empty() {
        let mut result = String::new();
        let mut last_idx = 0;
        for &point in &split_points {
            if point > last_idx && point < raw_filename.len() {
                result.push_str(&raw_filename[last_idx..point]);
                
                let prev_char = raw_filename[..point].chars().last();
                let next_char = raw_filename[point..].chars().next();
                let is_delim = |c: Option<char>| -> bool {
                    match c {
                        Some(ch) => ['.', '_', '-', ' ', '+', ',', '/', '\\', ':', '*', '×', '[', ']', '(', ')', '{', '}'].contains(&ch),
                        None => true,
                    }
                };
                if !is_delim(prev_char) && !is_delim(next_char) {
                    result.push('.');
                }
                last_idx = point;
            }
        }
        if last_idx < raw_filename.len() {
            result.push_str(&raw_filename[last_idx..]);
        }
        result
    } else {
        raw_filename.to_string()
    };

    let reconstructed_filename = reconstructed_filename.replace('_', ".");
    let filename = &reconstructed_filename;
    let mut info = TorrentInfo::default();
    let mut tokens = tokenize(filename);

    if tokens.is_empty() {
        return info;
    }

    // 1. Identify Container
    let known_containers = [
        "mkv", "mp4", "avi", "flv", "mov", "webm", "wmv", "m4v", "ts", "3gp", "ogm", "divx",
        "rar", "zip", "7z", "torrent", "nfo", "srt",
    ];
    if let Some(last_token) = tokens.last() {
        let text_lower = last_token.text.to_lowercase();
        if known_containers.contains(&text_lower.as_str()) {
            info.container = Some(text_lower);
            tokens.pop();
        }
    }

    // Helper to tag tokens matching a range
    let tag_range = |tokens: &mut [Token], start: usize, end: usize, cat: Category| {
        for token in tokens.iter_mut() {
            let mid = (token.start + token.end) / 2;
            if mid >= start && mid < end && token.category.is_none() {
                token.category = Some(cat);
            }
        }
    };

    // 2. Match Website
    if let Some(m) = patterns::website().find(filename) {
        info.website = Some(m.as_str().to_string());
        tag_range(&mut tokens, m.start(), m.end(), Category::Website);
    }

    // 3. Match Resolution
    if let Some(m) = patterns::resolution().find(filename) {
        let res_str = m.as_str().to_lowercase();
        let normalized = if res_str.contains("8k") || res_str.contains("4320") {
            "4320p".to_string()
        } else if res_str.contains("4k") || res_str.contains("uhd") || res_str.contains("3840x2160") {
            "2160p".to_string()
        } else if res_str.contains("fhd") || res_str.contains("1920x1080") {
            "1080p".to_string()
        } else if res_str.contains("qhd") || res_str.contains("2560x1440") {
            "1440p".to_string()
        } else if res_str.contains("1280x720") {
            "720p".to_string()
        } else if res_str.contains("sd") {
            "480p".to_string()
        } else if let Some(caps) = patterns::resolution().captures(m.as_str()) {
            if let Some(height) = caps.get(3) {
                format!("{}p", height.as_str())
            } else if let (Some(_w), Some(h)) = (caps.get(1), caps.get(2)) {
                format!("{}p", h.as_str())
            } else {
                res_str
            }
        } else {
            res_str
        };
        info.resolution = Some(normalized);
        tag_range(&mut tokens, m.start(), m.end(), Category::Resolution);
    }

    // 4. Match Video Codec
    if let Some(m) = patterns::video_codec().find(filename) {
        let codec_str = m.as_str().to_lowercase();
        let normalized = if codec_str.contains("av1") || codec_str.contains("av01") {
            "AV1".to_string()
        } else if codec_str.contains("265") || codec_str.contains("hevc") {
            "H.265".to_string()
        } else if codec_str.contains("264") || codec_str.contains("h264") || codec_str.contains("avc") {
            "H.264".to_string()
        } else if codec_str.contains("vp9") {
            "VP9".to_string()
        } else if codec_str.contains("vp8") {
            "VP8".to_string()
        } else if codec_str.contains("vc-1") || codec_str.contains("vc1") {
            "VC-1".to_string()
        } else if codec_str.contains("xvid") {
            "Xvid".to_string()
        } else if codec_str.contains("divx") {
            "DivX".to_string()
        } else if codec_str.contains("mpeg") {
            "MPEG-2".to_string()
        } else {
            m.as_str().to_string()
        };
        info.video_codec = Some(normalized);
        tag_range(&mut tokens, m.start(), m.end(), Category::VideoCodec);
    }

    // 5. Match Audio Codec
    if let Some(m) = patterns::audio_codec().find(filename) {
        let codec_str = m.as_str().to_lowercase();
        let normalized = if codec_str.contains("atmos") {
            "Dolby Atmos".to_string()
        } else if codec_str.contains("truehd") {
            "Dolby TrueHD".to_string()
        } else if codec_str.contains("dts-hd") || codec_str.contains("dtshd") || codec_str.contains("dtsma") || codec_str.contains("dts-ma") {
            "DTS-HD".to_string()
        } else if codec_str.contains("dts-x") || codec_str.contains("dtsx") {
            "DTS:X".to_string()
        } else if codec_str.contains("dts") {
            "DTS".to_string()
        } else if codec_str.contains("ddp") || codec_str.contains("dd+") || codec_str.contains("eac3") || codec_str.contains("e-ac3") {
            "Dolby Digital Plus".to_string()
        } else if codec_str.contains("ac3") || codec_str.contains("ac-3") || codec_str.contains("dd") || codec_str.contains("dolby") {
            "Dolby Digital".to_string()
        } else if codec_str.contains("aac") {
            "AAC".to_string()
        } else if codec_str.contains("flac") {
            "FLAC".to_string()
        } else if codec_str.contains("opus") {
            "Opus".to_string()
        } else if codec_str.contains("pcm") || codec_str.contains("lpcm") {
            "PCM".to_string()
        } else if codec_str.contains("mp3") || codec_str.contains("lame") {
            "MP3".to_string()
        } else {
            m.as_str().to_string()
        };
        info.audio_codec = Some(normalized);
        tag_range(&mut tokens, m.start(), m.end(), Category::AudioCodec);
    }

    // 6. Match Audio Channels
    if let Some(m) = patterns::audio_channels().find(filename) {
        let chan_str = m.as_str().to_lowercase().replace([' ', '_'], ".");
        let normalized = if chan_str.contains("7.1") || chan_str.contains("8ch") {
            "7.1".to_string()
        } else if chan_str.contains("5.1") || chan_str.contains("6ch") {
            "5.1".to_string()
        } else if chan_str.contains("2.0") || chan_str.contains("2ch") || chan_str.contains("stereo") {
            "2.0".to_string()
        } else if chan_str.contains("1.0") || chan_str.contains("1ch") || chan_str.contains("mono") {
            "1.0".to_string()
        } else {
            chan_str
        };
        info.audio_channels = Some(normalized);
        tag_range(&mut tokens, m.start(), m.end(), Category::AudioChannels);
    }

    // 7. Match Source
    if let Some(m) = patterns::source().find(filename) {
        let src_str = m.as_str().to_lowercase();
        let normalized = if src_str.contains("bluray") || src_str.contains("blu-ray") || src_str.contains("bdrip") || src_str.contains("brrip") || src_str.starts_with("bd") {
            "Blu-ray".to_string()
        } else if src_str.contains("web-dl") || src_str.contains("webdl") {
            "WEB-DL".to_string()
        } else if src_str.contains("webrip") || src_str.contains("web-rip") {
            "WEBRip".to_string()
        } else if src_str.contains("web") {
            "WEB".to_string()
        } else if src_str.contains("hdtv") {
            "HDTV".to_string()
        } else if src_str.contains("dvd") {
            "DVD".to_string()
        } else if src_str.contains("cam") {
            "CAM".to_string()
        } else if src_str.contains("ts") || src_str.contains("telesync") {
            "Telesync".to_string()
        } else if src_str.contains("tc") || src_str.contains("telecine") {
            "Telecine".to_string()
        } else if src_str.contains("remux") {
            "Remux".to_string()
        } else {
            m.as_str().to_string()
        };
        info.source = Some(normalized);
        tag_range(&mut tokens, m.start(), m.end(), Category::Source);
    }

    // 8. Match Language
    let mut langs = Vec::new();
    for m in patterns::language().find_iter(filename) {
        let lang_str = m.as_str().to_lowercase();
        let normalized = if lang_str == "multi" || lang_str == "dual" || lang_str.contains("bilingual") {
            "Multi-Audio".to_string()
        } else if lang_str == "vostfr" || lang_str == "vf" {
            "French".to_string()
        } else {
            let iso_lang = if lang_str.len() == 2 {
                isolang::Language::from_639_1(&lang_str)
            } else if lang_str.len() == 3 {
                isolang::Language::from_639_3(&lang_str)
            } else {
                lang_str.parse::<isolang::Language>().ok()
            };
            
            if let Some(l) = iso_lang {
                l.to_name().to_string()
            } else {
                let mut c = lang_str.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            }
        };
        if !langs.contains(&normalized) {
            langs.push(normalized);
        }
        tag_range(&mut tokens, m.start(), m.end(), Category::Language);
    }
    if !langs.is_empty() {
        info.language = Some(langs);
    }

    // 9. Match Streaming Services
    if let Some(m) = patterns::streaming_service().find(filename) {
        let svc_str = m.as_str().to_lowercase();
        let normalized = if svc_str == "nf" || svc_str.contains("netflx") || svc_str.contains("netflix") {
            "Netflix".to_string()
        } else if svc_str == "amzn" || svc_str.contains("amazon") {
            "Amazon Prime".to_string()
        } else if svc_str == "dsnp" || svc_str.contains("disney") {
            "Disney+".to_string()
        } else if svc_str == "atvp" || svc_str.contains("apple") {
            "Apple TV+".to_string()
        } else if svc_str == "hmax" || svc_str.contains("hbomax") || svc_str == "hbo" {
            "HBO Max".to_string()
        } else if svc_str == "pmod" || svc_str.contains("paramount") {
            "Paramount+".to_string()
        } else if svc_str == "stan" {
            "Stan".to_string()
        } else if svc_str == "cr" || svc_str.contains("crunchyroll") {
            "Crunchyroll".to_string()
        } else {
            m.as_str().to_string()
        };
        tag_range(&mut tokens, m.start(), m.end(), Category::Other);
        let mut oth = info.other.take().unwrap_or_default();
        if !oth.contains(&normalized) {
            oth.push(normalized);
        }
        info.other = Some(oth);
    }

    // 10. Match Other Tags
    let mut others = info.other.take().unwrap_or_default();
    for m in patterns::other_tags().find_iter(filename) {
        let tag_str = m.as_str().to_lowercase();
        let normalized = if tag_str.contains("hdr10+") {
            "HDR10+".to_string()
        } else if tag_str.contains("hdr10") {
            "HDR10".to_string()
        } else if tag_str.contains("hdr") {
            "HDR".to_string()
        } else if tag_str.contains("hlg") {
            "HLG".to_string()
        } else if tag_str.contains("sdr") {
            "SDR".to_string()
        } else if tag_str == "dv" || tag_str == "dovi" || tag_str.contains("dolbyvision") || tag_str.contains("dolby-vision") {
            "Dolby Vision".to_string()
        } else if tag_str.contains("10bit") || tag_str == "10b" || tag_str.contains("10-bit") {
            "10-bit".to_string()
        } else if tag_str == "3d" {
            "3D".to_string()
        } else if tag_str.contains("repack") {
            "Repack".to_string()
        } else if tag_str == "proper" {
            "Proper".to_string()
        } else if tag_str == "real" {
            "Real".to_string()
        } else if tag_str == "unrated" {
            "Unrated".to_string()
        } else if tag_str == "extended" {
            "Extended".to_string()
        } else if tag_str == "complete" {
            "Complete".to_string()
        } else if tag_str == "remastered" {
            "Remastered".to_string()
        } else if tag_str == "directors-cut" || tag_str == "dc" {
            "Director's Cut".to_string()
        } else if tag_str == "imax" {
            "IMAX".to_string()
        } else if tag_str.contains("subbed") || tag_str == "subs" {
            "Subbed".to_string()
        } else if tag_str == "dubbed" {
            "Dubbed".to_string()
        } else if tag_str == "hardsub" {
            "Hardsubbed".to_string()
        } else if tag_str == "obfuscated" {
            "Obfuscated".to_string()
        } else if tag_str == "scrambled" {
            "Scrambled".to_string()
        } else {
            m.as_str().to_string()
        };
        if !others.contains(&normalized) {
            others.push(normalized);
        }
        tag_range(&mut tokens, m.start(), m.end(), Category::Other);
    }

    // 11. Match CRC
    if let Some(m) = patterns::crc().find(filename) {
        tag_range(&mut tokens, m.start(), m.end(), Category::Other);
        let crc_val = format!("CRC32:{}", m.as_str());
        if !others.contains(&crc_val) {
            others.push(crc_val);
        }
    }
    if !others.is_empty() {
        info.other = Some(others);
    }

    // 12. Match Seasons and Episodes
    let mut seasons_found = Vec::new();
    let mut episodes_found = Vec::new();

    // 12a. Season/Episode Range (S01E02-E04 or S01E02-04)
    for cap in patterns::season_ep_range().captures_iter(filename) {
        if let (Some(s_match), Some(ep_start_match), Some(ep_end_match)) = (cap.get(1), cap.get(2), cap.get(3))
            && let (Ok(s), Ok(ep_start), Ok(ep_end)) = (
                s_match.as_str().parse::<u32>(),
                ep_start_match.as_str().parse::<u32>(),
                ep_end_match.as_str().parse::<u32>(),
            ) {
                seasons_found.push(s);
                for ep in ep_start..=ep_end {
                    episodes_found.push(ep);
                }
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Episode);
                }
            }
    }

    // 12b. Season/Episode Single (S01E02)
    for cap in patterns::season_ep().captures_iter(filename) {
        if let (Some(s_match), Some(ep_match)) = (cap.get(1), cap.get(2))
            && let (Ok(s), Ok(ep)) = (s_match.as_str().parse::<u32>(), ep_match.as_str().parse::<u32>()) {
                seasons_found.push(s);
                episodes_found.push(ep);
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Episode);
                }
            }
    }

    // 12c. Season/Episode X Range (1x02-04 or 1x02-e04)
    for cap in patterns::season_ep_x_range().captures_iter(filename) {
        if let (Some(s_match), Some(ep_start_match), Some(ep_end_match)) = (cap.get(1), cap.get(2), cap.get(3))
            && let (Ok(s), Ok(ep_start), Ok(ep_end)) = (
                s_match.as_str().parse::<u32>(),
                ep_start_match.as_str().parse::<u32>(),
                ep_end_match.as_str().parse::<u32>(),
            ) {
                seasons_found.push(s);
                for ep in ep_start..=ep_end {
                    episodes_found.push(ep);
                }
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Episode);
                }
            }
    }

    // 12d. Season/Episode X Single (1x02)
    for cap in patterns::season_ep_x_single().captures_iter(filename) {
        if let (Some(s_match), Some(ep_match)) = (cap.get(1), cap.get(2))
            && let (Ok(s), Ok(ep)) = (s_match.as_str().parse::<u32>(), ep_match.as_str().parse::<u32>()) {
                seasons_found.push(s);
                episodes_found.push(ep);
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Episode);
                }
            }
    }

    // 12e. Season Ranges (Season 1 - 3 or S01-S03)
    for cap in patterns::season_range().captures_iter(filename) {
        if let (Some(s_start_match), Some(s_end_match)) = (cap.get(1), cap.get(2))
            && let (Ok(s_start), Ok(s_end)) = (s_start_match.as_str().parse::<u32>(), s_end_match.as_str().parse::<u32>()) {
                for s in s_start..=s_end {
                    seasons_found.push(s);
                }
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Season);
                }
            }
    }
    for cap in patterns::season_range_short().captures_iter(filename) {
        if let (Some(s_start_match), Some(s_end_match)) = (cap.get(1), cap.get(2))
            && let (Ok(s_start), Ok(s_end)) = (s_start_match.as_str().parse::<u32>(), s_end_match.as_str().parse::<u32>()) {
                for s in s_start..=s_end {
                    seasons_found.push(s);
                }
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Season);
                }
            }
    }

    // 12f. Season Words (Season 1)
    for cap in patterns::season_word().captures_iter(filename) {
        if let Some(s_match) = cap.get(1)
            && let Ok(s) = s_match.as_str().parse::<u32>() {
                seasons_found.push(s);
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Season);
                }
            }
    }

    // 12g. Season Only (S01)
    for cap in patterns::season_only().captures_iter(filename) {
        if let Some(s_match) = cap.get(1)
            && let Ok(s) = s_match.as_str().parse::<u32>() {
                seasons_found.push(s);
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Season);
                }
            }
    }

    // 12h. Episode Ranges (Episode 01-03 or E01-E03)
    for cap in patterns::ep_range().captures_iter(filename) {
        if let (Some(ep_start_match), Some(ep_end_match)) = (cap.get(1), cap.get(2))
            && let (Ok(ep_start), Ok(ep_end)) = (ep_start_match.as_str().parse::<u32>(), ep_end_match.as_str().parse::<u32>()) {
                for ep in ep_start..=ep_end {
                    episodes_found.push(ep);
                }
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Episode);
                }
            }
    }
    for cap in patterns::ep_range_short().captures_iter(filename) {
        if let (Some(ep_start_match), Some(ep_end_match)) = (cap.get(1), cap.get(2))
            && let (Ok(ep_start), Ok(ep_end)) = (ep_start_match.as_str().parse::<u32>(), ep_end_match.as_str().parse::<u32>()) {
                for ep in ep_start..=ep_end {
                    episodes_found.push(ep);
                }
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Episode);
                }
            }
    }

    // 12i. Episode Words (Episode 1)
    for cap in patterns::episode_word().captures_iter(filename) {
        if let Some(ep_match) = cap.get(1)
            && let Ok(ep) = ep_match.as_str().parse::<u32>() {
                episodes_found.push(ep);
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Episode);
                }
            }
    }

    // 12j. Episode Only (E01)
    for cap in patterns::ep_only().captures_iter(filename) {
        if let Some(ep_match) = cap.get(1)
            && let Ok(ep) = ep_match.as_str().parse::<u32>() {
                episodes_found.push(ep);
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Episode);
                }
            }
    }

    // 12k. Anime Version Episode (01v2)
    for cap in patterns::anime_ep_v().captures_iter(filename) {
        if let Some(ep_match) = cap.get(1)
            && let Ok(ep) = ep_match.as_str().parse::<u32>() {
                episodes_found.push(ep);
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Episode);
                }
            }
    }

    // 12l. Anime / Lone Episode Number Heuristic
    // Scan tokens for untagged numbers (excluding common resolutions, years, and values >= 1000)
    let common_resolutions: HashSet<u32> = [360, 480, 576, 720, 900, 1080, 1440, 2160, 4320].iter().cloned().collect();
    let mut lone_episode_token_idx = None;

    for (idx, token) in tokens.iter().enumerate() {
        if token.category.is_none()
            && let Ok(val) = token.text.parse::<u32>() {
                // If it is < 1000 and not a common resolution/year
                if val < 1000 && !common_resolutions.contains(&val) {
                    let preceded_by_hyphen = if token.start > 0 {
                        let prev_part = &filename[..token.start];
                        prev_part.trim_end().ends_with('-')
                    } else {
                        false
                    };

                    let is_last_unclassified_number = {
                        let mut last = true;
                        for next_token in &tokens[idx + 1..] {
                            if next_token.category.is_none() && next_token.text.parse::<u32>().is_ok() {
                                last = false;
                                break;
                            }
                        }
                        last
                    };

                    if preceded_by_hyphen || is_last_unclassified_number {
                        lone_episode_token_idx = Some((idx, val));
                    }
                }
            }
    }

    if let Some((idx, ep_val)) = lone_episode_token_idx {
        episodes_found.push(ep_val);
        tokens[idx].category = Some(Category::Episode);
    }

    // Sort & Deduplicate seasons/episodes
    if !seasons_found.is_empty() {
        seasons_found.sort_unstable();
        seasons_found.dedup();
        info.season = Some(seasons_found);
    }
    if !episodes_found.is_empty() {
        episodes_found.sort_unstable();
        episodes_found.dedup();
        info.episode = Some(episodes_found);
    }

    // 13. Match Year
    let mut year_matches: Vec<_> = patterns::year().find_iter(filename).collect();
    if !year_matches.is_empty() {
        let first_token_start = tokens.first().map(|t| t.start).unwrap_or(0);
        let actual_year_match = if year_matches.len() > 1 && year_matches[0].start() == first_token_start {
            year_matches.remove(1)
        } else {
            year_matches.remove(0)
        };

        if let Ok(y) = actual_year_match.as_str().parse::<u32>() {
            info.year = Some(y);
            tag_range(&mut tokens, actual_year_match.start(), actual_year_match.end(), Category::Year);
        }
    }

    // 14. Extract Title
    let mut title_parts = Vec::new();
    let mut first_unbracketed_seen = false;

    for token in &tokens {
        if !first_unbracketed_seen {
            if token.bracket != Bracket::None {
                continue;
            }
            first_unbracketed_seen = true;
        }

        if let Some(cat) = token.category {
            match cat {
                Category::Website => {
                    continue;
                }
                Category::Year | Category::Resolution | Category::Source | Category::VideoCodec |
                Category::AudioCodec | Category::AudioChannels | Category::Season | Category::Episode |
                Category::Other => {
                    break;
                }
                _ => {}
            }
        }

        title_parts.push(token.text);
    }

    if !title_parts.is_empty() {
        info.title = title_parts.join(" ");
    } else {
        let fallback_words: Vec<&str> = tokens.iter()
            .filter(|t| t.bracket == Bracket::None && !t.is_tagged())
            .map(|t| t.text)
            .collect();
        if !fallback_words.is_empty() {
            info.title = fallback_words.join(" ");
        } else {
            info.title = filename.split('.').next().unwrap_or(filename).to_string();
        }
    }

    // 15. Extract Release Group
    let mut release_group_opt = None;
    if let Some(last_token) = tokens.last()
        && last_token.bracket == Bracket::None && !last_token.is_tagged() && last_token.start > 0 {
            let prev_char = filename[..last_token.start].chars().last();
            if prev_char == Some('-') {
                release_group_opt = Some(last_token.text.to_string());
            }
        }

    if release_group_opt.is_none()
        && let Some(first_token) = tokens.first()
            && first_token.bracket == Bracket::Square && !first_token.is_tagged()
                && let Some(open_idx) = filename[..first_token.start].rfind('[')
                    && let Some(close_idx) = filename[first_token.start..].find(']') {
                        let full_close_idx = first_token.start + close_idx;
                        let group_text = filename[open_idx + 1..full_close_idx].trim();
                        let txt_lower = group_text.to_lowercase();
                        if !txt_lower.contains("http") && !txt_lower.contains(".org") && !txt_lower.contains(".com") {
                            release_group_opt = Some(group_text.to_string());
                        }
                    }

    if release_group_opt.is_none() {
        for token in tokens.iter().rev() {
            if (token.bracket == Bracket::Square || token.bracket == Bracket::Parentheses) && !token.is_tagged() {
                let txt_lower = token.text.to_lowercase();
                if txt_lower.len() > 1 && !txt_lower.chars().all(|c| c.is_ascii_hexdigit()) && token.text.parse::<u32>().is_err() {
                    let open_char = if token.bracket == Bracket::Square { '[' } else { '(' };
                    let close_char = if token.bracket == Bracket::Square { ']' } else { ')' };
                    if let Some(open_idx) = filename[..token.start].rfind(open_char)
                        && let Some(close_idx) = filename[token.start..].find(close_char) {
                            let full_close_idx = token.start + close_idx;
                            let group_text = filename[open_idx + 1..full_close_idx].trim().to_string();
                            release_group_opt = Some(group_text);
                            break;
                        }
                }
            }
        }
    }

    info.release_group = release_group_opt;

    // 16. Extract Episode Title
    let mut ep_title_parts = Vec::new();
    if info.episode.is_some()
        && let Some(last_ep_token_idx) = tokens.iter().enumerate()
            .filter(|(_, t)| t.category == Some(Category::Episode))
            .map(|(idx, _)| idx)
            .next_back()
        {
            for token in &tokens[last_ep_token_idx + 1..] {
                if token.bracket != Bracket::None {
                    break;
                }
                if matches!(
                    token.category,
                    Some(
                        Category::Year | Category::Resolution | Category::Source | Category::VideoCodec |
                        Category::AudioCodec | Category::AudioChannels | Category::Other | Category::Website
                    )
                ) {
                    break;
                }
                
                ep_title_parts.push(token.text);
            }
        }
    
    if let Some(ref group) = info.release_group
        && let Some(last) = ep_title_parts.last()
            && last == group {
                ep_title_parts.pop();
            }
    
    if !ep_title_parts.is_empty() {
        info.episode_title = Some(ep_title_parts.join(" "));
    }

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix() {
        let info = parse("The.Matrix.1999.1080p.BluRay.x264-FGT.mkv");
        assert_eq!(info.title, "The Matrix");
        assert_eq!(info.year, Some(1999));
        assert_eq!(info.resolution, Some("1080p".to_string()));
        assert_eq!(info.source, Some("Blu-ray".to_string()));
        assert_eq!(info.video_codec, Some("H.264".to_string()));
        assert_eq!(info.release_group, Some("FGT".to_string()));
        assert_eq!(info.container, Some("mkv".to_string()));
        assert!(info.is_movie());
        assert!(!info.is_tv_show());
        assert_eq!(info.full_title(), "The Matrix (1999)");
    }

    #[test]
    fn test_av1_codec() {
        let info = parse("Dune.Part.Two.2024.2160p.UHD.BluRay.AV1.10bit.HDR.Atmos-FLUX.mkv");
        assert_eq!(info.title, "Dune Part Two");
        assert_eq!(info.year, Some(2024));
        assert_eq!(info.resolution, Some("2160p".to_string()));
        assert_eq!(info.video_codec, Some("AV1".to_string()));
        assert_eq!(info.release_group, Some("FLUX".to_string()));
    }

    #[test]
    fn test_streaming_service() {
        let info = parse("Stranger.Things.S04E01.1080p.NF.WEB-DL.DDP5.1.Atmos.H.264-NTb.mkv");
        assert_eq!(info.title, "Stranger Things");
        assert_eq!(info.season, Some(vec![4]));
        assert_eq!(info.episode, Some(vec![1]));
        assert_eq!(info.resolution, Some("1080p".to_string()));
        assert_eq!(info.source, Some("WEB-DL".to_string()));
        assert!(info.is_tv_show());
        assert!(info.other.unwrap().contains(&"Netflix".to_string()));
    }

    #[test]
    fn test_anime_subsplease() {
        let info = parse("[SubsPlease] Kono Subarashii Sekai ni Shukufuku wo! 3 - 01 (1080p) [F109283].mkv");
        assert_eq!(info.title, "Kono Subarashii Sekai ni Shukufuku wo! 3");
        assert_eq!(info.episode, Some(vec![1]));
        assert_eq!(info.resolution, Some("1080p".to_string()));
        assert_eq!(info.release_group, Some("SubsPlease".to_string()));
        assert_eq!(info.container, Some("mkv".to_string()));
        assert!(info.other.unwrap().contains(&"CRC32:F109283".to_string()));
    }

    #[test]
    fn test_game_of_thrones() {
        let info = parse("Game.of.Thrones.S01E01-E03.1080p.mkv");
        assert_eq!(info.title, "Game of Thrones");
        assert_eq!(info.season, Some(vec![1]));
        assert_eq!(info.episode, Some(vec![1, 2, 3]));
        assert_eq!(info.resolution, Some("1080p".to_string()));
    }

    #[test]
    fn test_friends_season_range() {
        let info = parse("Friends.S01-S03.720p.mkv");
        assert_eq!(info.title, "Friends");
        assert_eq!(info.season, Some(vec![1, 2, 3]));
        assert_eq!(info.resolution, Some("720p".to_string()));
    }

    #[test]
    fn test_1917_movie() {
        let info = parse("1917.2019.1080p.BluRay.x264.mkv");
        assert_eq!(info.title, "1917");
        assert_eq!(info.year, Some(2019));
        assert_eq!(info.resolution, Some("1080p".to_string()));
        assert_eq!(info.source, Some("Blu-ray".to_string()));
    }

    #[test]
    fn test_iron_man_3() {
        let info = parse("Iron.Man.3.2013.1080p.mkv");
        assert_eq!(info.title, "Iron Man 3");
        assert_eq!(info.year, Some(2013));
        assert_eq!(info.resolution, Some("1080p".to_string()));
    }

    #[test]
    fn test_simpsons() {
        let info = parse("The.Simpsons.S28E15.720p.HDTV.x264-AVS.mkv");
        assert_eq!(info.title, "The Simpsons");
        assert_eq!(info.season, Some(vec![28]));
        assert_eq!(info.episode, Some(vec![15]));
        assert_eq!(info.resolution, Some("720p".to_string()));
        assert_eq!(info.source, Some("HDTV".to_string()));
        assert_eq!(info.video_codec, Some("H.264".to_string()));
        assert_eq!(info.release_group, Some("AVS".to_string()));
    }

    #[test]
    fn test_interstellar() {
        let info = parse("Interstellar.2014.2160p.UHD.BluRay.x265.10bit.HDR.Atmos.7.1-TERMINAL.mkv");
        assert_eq!(info.title, "Interstellar");
        assert_eq!(info.year, Some(2014));
        assert_eq!(info.resolution, Some("2160p".to_string()));
        assert_eq!(info.source, Some("Blu-ray".to_string()));
        assert_eq!(info.video_codec, Some("H.265".to_string()));
        assert_eq!(info.audio_codec, Some("Dolby Atmos".to_string()));
        assert_eq!(info.audio_channels, Some("7.1".to_string()));
        assert_eq!(info.release_group, Some("TERMINAL".to_string()));
        
        let other = info.other.unwrap();
        assert!(other.contains(&"10-bit".to_string()));
        assert!(other.contains(&"HDR".to_string()));
    }

    #[test]
    fn test_agents_of_shield() {
        let info = parse("www.Torrenting.com - Marvels.Agents.of.S.H.I.E.L.D.S04E15.1080p.WEB-DL.DD5.1.H264-RARBG.mkv");
        assert_eq!(info.title, "Marvels Agents of S H I E L D");
        assert_eq!(info.season, Some(vec![4]));
        assert_eq!(info.episode, Some(vec![15]));
        assert_eq!(info.resolution, Some("1080p".to_string()));
        assert_eq!(info.source, Some("WEB-DL".to_string()));
        assert_eq!(info.video_codec, Some("H.264".to_string()));
        assert_eq!(info.audio_codec, Some("Dolby Digital".to_string()));
        assert_eq!(info.audio_channels, Some("5.1".to_string()));
        assert_eq!(info.release_group, Some("RARBG".to_string()));
        assert_eq!(info.website, Some("www.Torrenting.com".to_string()));
    }

    #[test]
    fn test_erai_raws() {
        let info = parse("[Erai-raws] Shingeki no Kyojin - The Final Season - 16 [1080p][HEVC 10bit][AAC][Multiple Subtitle] (Obfuscated).mkv");
        assert_eq!(info.title, "Shingeki no Kyojin The Final Season");
        assert_eq!(info.episode, Some(vec![16]));
        assert_eq!(info.resolution, Some("1080p".to_string()));
        assert_eq!(info.video_codec, Some("H.265".to_string()));
        assert_eq!(info.audio_codec, Some("AAC".to_string()));
        assert_eq!(info.release_group, Some("Erai-raws".to_string()));
        
        let other = info.other.unwrap();
        assert!(other.contains(&"10-bit".to_string()));
        assert!(other.contains(&"Obfuscated".to_string()));
    }

    #[test]
    fn test_episode_title() {
        let info = parse("Cyberpunk.Edgerunners.S01E02.Like.a.Boy.1080p.BluRay.Remux.Dual-Audio.DTS-HD.MA.5.1.H.264-CRUCiBLE.mkv");
        assert_eq!(info.title, "Cyberpunk Edgerunners");
        assert_eq!(info.season, Some(vec![1]));
        assert_eq!(info.episode, Some(vec![2]));
        assert_eq!(info.episode_title, Some("Like a Boy".to_string()));
        assert_eq!(info.resolution, Some("1080p".to_string()));
        assert_eq!(info.release_group, Some("CRUCiBLE".to_string()));
    }

    #[test]
    fn test_love_death_robots() {
        let info = parse("Love.Death.And.Robots.S02E01.Automated.Customer.Service.2160p.Web.mkv");
        assert_eq!(info.title, "Love Death And Robots");
        assert_eq!(info.season, Some(vec![2]));
        assert_eq!(info.episode, Some(vec![1]));
        assert_eq!(info.episode_title, Some("Automated Customer Service".to_string()));
    }

    #[test]
    fn test_no_delimiters() {
        let info = parse("Interstellar20142160pHEVCx265-RARBG.mkv");
        assert_eq!(info.title, "Interstellar");
        assert_eq!(info.year, Some(2014));
        assert_eq!(info.resolution, Some("2160p".to_string()));
        assert_eq!(info.video_codec, Some("H.265".to_string()));
        assert_eq!(info.release_group, Some("RARBG".to_string()));
    }
}
