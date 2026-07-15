use crate::token::{Bracket, Category, Token, tokenize};
use crate::patterns;
use std::collections::HashSet;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TorrentInfo {
    pub title: String,
    pub year: Option<u32>,
    pub season: Option<Vec<u32>>,
    pub episode: Option<Vec<u32>>,
    pub episode_title: Option<String>,
    pub resolution: Option<String>,
    pub source: Option<String>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub audio_channels: Option<String>,
    pub release_group: Option<String>,
    pub language: Option<Vec<String>>,
    pub other: Option<Vec<String>>,
    pub container: Option<String>,
    pub website: Option<String>,
}

pub fn parse(filename: &str) -> TorrentInfo {
    // Split path to handle nested torrent structures (parent folders with metadata)
    let segments: Vec<&str> = filename.split(|c| c == '/' || c == '\\')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if segments.is_empty() {
        return TorrentInfo::default();
    }

    let last_segment = segments.last().unwrap();
    let mut file_info = parse_segment(last_segment);

    if segments.len() > 1 && is_generic_file(&file_info.title, file_info.container.as_deref()) {
        let parent_segment = segments[segments.len() - 2];
        let parent_info = parse_segment(parent_segment);
        
        // Merge parent metadata into file metadata
        if file_info.title.to_lowercase() == "sample" || file_info.title.is_empty() || is_generic_word(&file_info.title) {
            file_info.title = parent_info.title;
        }
        if file_info.year.is_none() {
            file_info.year = parent_info.year;
        }
        if file_info.season.is_none() {
            file_info.season = parent_info.season;
        }
        if file_info.episode.is_none() {
            file_info.episode = parent_info.episode;
        }
        if file_info.episode_title.is_none() {
            file_info.episode_title = parent_info.episode_title;
        }
        if file_info.resolution.is_none() {
            file_info.resolution = parent_info.resolution;
        }
        if file_info.source.is_none() {
            file_info.source = parent_info.source;
        }
        if file_info.video_codec.is_none() {
            file_info.video_codec = parent_info.video_codec;
        }
        if file_info.audio_codec.is_none() {
            file_info.audio_codec = parent_info.audio_codec;
        }
        if file_info.audio_channels.is_none() {
            file_info.audio_channels = parent_info.audio_channels;
        }
        if file_info.release_group.is_none() {
            file_info.release_group = parent_info.release_group;
        }
        if file_info.website.is_none() {
            file_info.website = parent_info.website;
        }
        if file_info.language.is_none() {
            file_info.language = parent_info.language;
        }
        
        if let Some(p_other) = parent_info.other {
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
    ];
    generic_words.contains(&w_lower.as_str())
}

fn parse_segment(raw_filename: &str) -> TorrentInfo {
    // Reconstruct filename with dots at split points to provide word boundaries for delimiterless metadata runs
    let mut split_points = std::collections::BTreeSet::new();
    let split_regexes = [
        regex::Regex::new(r"((?:19|20)\d{2})").unwrap(),
        regex::Regex::new(r"(?i)(2160[pi]?|1080[pi]?|720[pi]?|480[pi]?|4k|8k|uhd)").unwrap(),
        regex::Regex::new(r"(?i)(x[._-]?265|h[._-]?265|hevc|x[._-]?264|h[._-]?264|avc)").unwrap(),
        regex::Regex::new(r"(?i)(?:^|[^0-9])(5\.1|7\.1|2\.0)(?:[^0-9]|$)").unwrap(),
        regex::Regex::new(r"(?i)(bluray|webdl|webrip|hdtv|remux)").unwrap(),
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
        "rar", "zip", "7z", "torrent", "nfo",
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
            if mid >= start && mid < end {
                if token.category.is_none() {
                    token.category = Some(cat);
                }
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
        let normalized = if res_str.contains("4k") || res_str.contains("uhd") {
            "2160p".to_string()
        } else if res_str.contains("fhd") {
            "1080p".to_string()
        } else if res_str.contains("qhd") {
            "1440p".to_string()
        } else if res_str.contains("sd") {
            "480p".to_string()
        } else if let Some(caps) = patterns::resolution().captures(m.as_str()) {
            if let Some(height) = caps.get(3) {
                format!("{}p", height.as_str())
            } else if let (Some(w), Some(h)) = (caps.get(1), caps.get(2)) {
                format!("{}x{}", w.as_str(), h.as_str())
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
        let normalized = if codec_str.contains("265") || codec_str.contains("hevc") {
            "H.265".to_string()
        } else if codec_str.contains("264") || codec_str.contains("h264") || codec_str.contains("avc") {
            "H.264".to_string()
        } else if codec_str.contains("vp9") {
            "VP9".to_string()
        } else if codec_str.contains("vp8") {
            "VP8".to_string()
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
        let chan_str = m.as_str().to_lowercase().replace(' ', ".").replace('_', ".");
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

    // 9. Match Other Tags
    let mut others = Vec::new();
    for m in patterns::other_tags().find_iter(filename) {
        let tag_str = m.as_str().to_lowercase();
        let normalized = if tag_str.contains("hdr10+") {
            "HDR10+".to_string()
        } else if tag_str.contains("hdr10") {
            "HDR10".to_string()
        } else if tag_str.contains("hdr") {
            "HDR".to_string()
        } else if tag_str == "dv" || tag_str == "dovi" || tag_str.contains("dolbyvision") || tag_str.contains("dolby-vision") {
            "Dolby Vision".to_string()
        } else if tag_str.contains("10bit") || tag_str == "10b" || tag_str.contains("10-bit") {
            "10-bit".to_string()
        } else if tag_str == "3d" {
            "3D".to_string()
        } else if tag_str == "repack" {
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

    // 10. Match CRC
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

    // 11. Match Seasons and Episodes
    let mut seasons_found = Vec::new();
    let mut episodes_found = Vec::new();

    // 11a. Season/Episode Range (S01E02-E04 or S01E02-04)
    for cap in patterns::season_ep_range().captures_iter(filename) {
        if let (Some(s_match), Some(ep_start_match), Some(ep_end_match)) = (cap.get(1), cap.get(2), cap.get(3)) {
            if let (Ok(s), Ok(ep_start), Ok(ep_end)) = (
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
    }

    // 11b. Season/Episode Single (S01E02)
    for cap in patterns::season_ep().captures_iter(filename) {
        if let (Some(s_match), Some(ep_match)) = (cap.get(1), cap.get(2)) {
            if let (Ok(s), Ok(ep)) = (s_match.as_str().parse::<u32>(), ep_match.as_str().parse::<u32>()) {
                seasons_found.push(s);
                episodes_found.push(ep);
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Episode);
                }
            }
        }
    }

    // 11c. Season/Episode X Range (1x02-04 or 1x02-e04)
    for cap in patterns::season_ep_x_range().captures_iter(filename) {
        if let (Some(s_match), Some(ep_start_match), Some(ep_end_match)) = (cap.get(1), cap.get(2), cap.get(3)) {
            if let (Ok(s), Ok(ep_start), Ok(ep_end)) = (
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
    }

    // 11d. Season/Episode X Single (1x02)
    for cap in patterns::season_ep_x_single().captures_iter(filename) {
        if let (Some(s_match), Some(ep_match)) = (cap.get(1), cap.get(2)) {
            if let (Ok(s), Ok(ep)) = (s_match.as_str().parse::<u32>(), ep_match.as_str().parse::<u32>()) {
                seasons_found.push(s);
                episodes_found.push(ep);
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Episode);
                }
            }
        }
    }

    // 11d. Season Ranges (Season 1 - 3 or S01-S03)
    for cap in patterns::season_range().captures_iter(filename) {
        if let (Some(s_start_match), Some(s_end_match)) = (cap.get(1), cap.get(2)) {
            if let (Ok(s_start), Ok(s_end)) = (s_start_match.as_str().parse::<u32>(), s_end_match.as_str().parse::<u32>()) {
                for s in s_start..=s_end {
                    seasons_found.push(s);
                }
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Season);
                }
            }
        }
    }
    for cap in patterns::season_range_short().captures_iter(filename) {
        if let (Some(s_start_match), Some(s_end_match)) = (cap.get(1), cap.get(2)) {
            if let (Ok(s_start), Ok(s_end)) = (s_start_match.as_str().parse::<u32>(), s_end_match.as_str().parse::<u32>()) {
                for s in s_start..=s_end {
                    seasons_found.push(s);
                }
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Season);
                }
            }
        }
    }

    // 11e. Season Words (Season 1)
    for cap in patterns::season_word().captures_iter(filename) {
        if let Some(s_match) = cap.get(1) {
            if let Ok(s) = s_match.as_str().parse::<u32>() {
                seasons_found.push(s);
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Season);
                }
            }
        }
    }

    // 11f. Season Only (S01)
    for cap in patterns::season_only().captures_iter(filename) {
        if let Some(s_match) = cap.get(1) {
            if let Ok(s) = s_match.as_str().parse::<u32>() {
                seasons_found.push(s);
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Season);
                }
            }
        }
    }

    // 11g. Episode Ranges (Episode 01-03 or E01-E03)
    for cap in patterns::ep_range().captures_iter(filename) {
        if let (Some(ep_start_match), Some(ep_end_match)) = (cap.get(1), cap.get(2)) {
            if let (Ok(ep_start), Ok(ep_end)) = (ep_start_match.as_str().parse::<u32>(), ep_end_match.as_str().parse::<u32>()) {
                for ep in ep_start..=ep_end {
                    episodes_found.push(ep);
                }
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Episode);
                }
            }
        }
    }
    for cap in patterns::ep_range_short().captures_iter(filename) {
        if let (Some(ep_start_match), Some(ep_end_match)) = (cap.get(1), cap.get(2)) {
            if let (Ok(ep_start), Ok(ep_end)) = (ep_start_match.as_str().parse::<u32>(), ep_end_match.as_str().parse::<u32>()) {
                for ep in ep_start..=ep_end {
                    episodes_found.push(ep);
                }
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Episode);
                }
            }
        }
    }

    // 11h. Episode Words (Episode 1)
    for cap in patterns::episode_word().captures_iter(filename) {
        if let Some(ep_match) = cap.get(1) {
            if let Ok(ep) = ep_match.as_str().parse::<u32>() {
                episodes_found.push(ep);
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Episode);
                }
            }
        }
    }

    // 11i. Episode Only (E01)
    for cap in patterns::ep_only().captures_iter(filename) {
        if let Some(ep_match) = cap.get(1) {
            if let Ok(ep) = ep_match.as_str().parse::<u32>() {
                episodes_found.push(ep);
                if let Some(full) = cap.get(0) {
                    tag_range(&mut tokens, full.start(), full.end(), Category::Episode);
                }
            }
        }
    }

    // 11j. Anime / Lone Episode Number Heuristic
    // Scan tokens for untagged numbers (excluding common resolutions, years, and values >= 1000)
    let common_resolutions: HashSet<u32> = [360, 480, 576, 720, 900, 1080, 1440, 2160, 4320].iter().cloned().collect();
    let mut lone_episode_token_idx = None;

    for (idx, token) in tokens.iter().enumerate() {
        if token.category.is_none() {
            if let Ok(val) = token.text.parse::<u32>() {
                // If it is < 1000 and not a common resolution/year
                if val < 1000 && !common_resolutions.contains(&val) {
                    // Check context: preceded by hyphen or last unclassified number
                    let preceded_by_hyphen = if token.start > 0 {
                        let prev_part = &filename[..token.start];
                        prev_part.trim_end().ends_with('-')
                    } else {
                        false
                    };

                    // Let's also check if the next unclassified token is bracketed metadata
                    let is_last_unclassified_number = {
                        let mut last = true;
                        for next_token in &tokens[idx + 1..] {
                            if next_token.category.is_none() {
                                if next_token.text.parse::<u32>().is_ok() {
                                    last = false;
                                    break;
                                }
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

    // 12. Match Year
    let mut year_matches: Vec<_> = patterns::year().find_iter(filename).collect();
    if !year_matches.is_empty() {
        // Resolve multiple years:
        // If the first token starts with a year match, and there is another year match later,
        // the first year is likely part of the title (e.g. "1917 (2019)").
        let first_token_start = tokens.first().map(|t| t.start).unwrap_or(0);
        let actual_year_match = if year_matches.len() > 1 && year_matches[0].start() == first_token_start {
            year_matches.remove(1) // Keep the second year as the metadata year
        } else {
            year_matches.remove(0)
        };

        if let Ok(y) = actual_year_match.as_str().parse::<u32>() {
            info.year = Some(y);
            tag_range(&mut tokens, actual_year_match.start(), actual_year_match.end(), Category::Year);
        }
    }

    // 13. Extract Title
    // Title is the sequence of consecutive untagged tokens from the start (ignoring leading bracketed release group)
    let mut title_parts = Vec::new();
    let mut first_unbracketed_seen = false;

    for token in &tokens {
        // Skip leading square bracketed/parentheses tokens (usually release group or other tags)
        if !first_unbracketed_seen {
            if token.bracket != Bracket::None {
                continue;
            }
            first_unbracketed_seen = true;
        }

        // Stop collecting title tokens when we hit a tagged metadata token (except website/language if they don't block)
        if let Some(cat) = token.category {
            match cat {
                Category::Website => {
                    continue; // Skip website tokens completely from title
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
        // Fallback: collect first few unbracketed words
        let fallback_words: Vec<&str> = tokens.iter()
            .filter(|t| t.bracket == Bracket::None && !t.is_tagged())
            .map(|t| t.text)
            .collect();
        if !fallback_words.is_empty() {
            info.title = fallback_words.join(" ");
        } else {
            // Last resort: raw name minus extension
            info.title = filename.split('.').next().unwrap_or(filename).to_string();
        }
    }

    // 14. Extract Release Group
    // Rule A: Last token is preceded by '-' and is untagged.
    let mut release_group_opt = None;
    if let Some(last_token) = tokens.last() {
        if last_token.bracket == Bracket::None && !last_token.is_tagged() {
            if last_token.start > 0 {
                let prev_char = filename[..last_token.start].chars().last();
                if prev_char == Some('-') {
                    release_group_opt = Some(last_token.text.to_string());
                }
            }
        }
    }

    // Rule B: First token was inside square brackets, and is not tagged.
    if release_group_opt.is_none() {
        if let Some(first_token) = tokens.first() {
            if first_token.bracket == Bracket::Square && !first_token.is_tagged() {
                // Find the full bracketed content from the raw filename
                if let Some(open_idx) = filename[..first_token.start].rfind('[') {
                    if let Some(close_idx) = filename[first_token.start..].find(']') {
                        let full_close_idx = first_token.start + close_idx;
                        let group_text = filename[open_idx + 1..full_close_idx].trim();
                        let txt_lower = group_text.to_lowercase();
                        if !txt_lower.contains("http") && !txt_lower.contains(".org") && !txt_lower.contains(".com") {
                            release_group_opt = Some(group_text.to_string());
                        }
                    }
                }
            }
        }
    }

    // Rule C: Last bracketed token before extension/CRC is unclassified and not tagged.
    if release_group_opt.is_none() {
        for token in tokens.iter().rev() {
            if (token.bracket == Bracket::Square || token.bracket == Bracket::Parentheses) && !token.is_tagged() {
                let txt_lower = token.text.to_lowercase();
                // Avoid picking up year, crc or obvious non-groups
                if txt_lower.len() > 1 && !txt_lower.chars().all(|c| c.is_ascii_hexdigit()) && token.text.parse::<u32>().is_err() {
                    let open_char = if token.bracket == Bracket::Square { '[' } else { '(' };
                    let close_char = if token.bracket == Bracket::Square { ']' } else { ')' };
                    if let Some(open_idx) = filename[..token.start].rfind(open_char) {
                        if let Some(close_idx) = filename[token.start..].find(close_char) {
                            let full_close_idx = token.start + close_idx;
                            let group_text = filename[open_idx + 1..full_close_idx].trim().to_string();
                            release_group_opt = Some(group_text);
                            break;
                        }
                    }
                }
            }
        }
    }

    info.release_group = release_group_opt;

    // 15. Extract Episode Title
    // If we have an episode number, walk through tokens after the last episode token
    // and collect any contiguous unbracketed, untagged words before metadata/brackets.
    let mut ep_title_parts = Vec::new();
    if info.episode.is_some() {
        if let Some(last_ep_token_idx) = tokens.iter().enumerate()
            .filter(|(_, t)| t.category == Some(Category::Episode))
            .map(|(idx, _)| idx)
            .last() 
        {
            for token in &tokens[last_ep_token_idx + 1..] {
                if token.bracket != Bracket::None {
                    break;
                }
                if let Some(cat) = token.category {
                    match cat {
                        Category::Year | Category::Resolution | Category::Source | Category::VideoCodec |
                        Category::AudioCodec | Category::AudioChannels | Category::Other | Category::Website => {
                            break;
                        }
                        _ => {}
                    }
                }
                
                ep_title_parts.push(token.text);
            }
        }
    }
    
    // Filter out release group from episode title if it overlaps
    if let Some(ref group) = info.release_group {
        if let Some(last) = ep_title_parts.last() {
            if *last == group {
                ep_title_parts.pop();
            }
        }
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
