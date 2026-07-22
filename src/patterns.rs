use std::sync::OnceLock;
use regex::Regex;

macro_rules! define_regex {
    ($name:ident, $pat:expr) => {
        pub fn $name() -> &'static Regex {
            static RE: OnceLock<Regex> = OnceLock::new();
            RE.get_or_init(|| Regex::new($pat).expect(concat!("Failed to compile regex: ", stringify!($name))))
        }
    };
}

define_regex!(year, r"(?i)\b(18|19|20)\d{2}\b");
define_regex!(resolution, r"(?i)\b(?:(\d{3,4})x(\d{3,4})|(\d{3,4})[pi]|4k|8k|uhd|fhd|qhd|sd)\b");
define_regex!(video_codec, r"(?i)\b(av1|av01|svt-av1|x[._-]?265|h[._-]?265|hevc|x[._-]?264|h[._-]?264|avc|avchd|vp9|vp8|vc-?1|xvid|divx|mpeg4|mpeg-?4|mpeg2|mpeg-?2)\b");
define_regex!(audio_codec, r"(?i)\b(atmos|truehd|dts-hd\s*ma|dts-hd\s*hra|dts-hd|dts-ma|dts-x|dts-es|dts|ddp|dd\+|eac3|e-ac3|ac3|ac-3|dd\d*|dolby|aac-lc|aac|flac|opus|pcm|lpcm|alac|mp3|lame|wma)\b");
define_regex!(audio_channels, r"(?i)\b(?:dd|ch)?(7[._\s]1|5[._\s]1|2[._\s]0|1[._\s]0|8ch|6ch|2ch|1ch)\b|\b(stereo|mono)\b");
define_regex!(source, r"(?i)\b(bluray|blu-ray|bdrip|brrip|bd25|bd50|bd|web-dl|webdl|webrip|web-rip|web|hdtv|hd-tv|dsr|satrip|dvdrip|dvd-rip|dvd5|dvd9|dvd|dvdr|tvrip|pdtv|sdtv|vhs|camrip|cam|ts|telesync|screener|scr|tc|telecine|remux)\b");
define_regex!(language, r"(?i)\b(english|eng|french|fra|fre|vostfr|vf|spanish|spa|german|ger|deu|italian|ita|japanese|jpn|korean|kor|chinese|chi|zho|chs|cht|russian|rus|portuguese|por|polish|pol|dutch|nld|swedish|swe|turkish|tur|multi|dual|bilingual)\b");
define_regex!(streaming_service, r"(?i)\b(nf|netflix|amzn|amazon|dsnp|disney\+?|atvp|apple\s*tv\+?|hmax|hbomax|hbo|pmod|paramount\+?|stan|cr|crunchyroll)\b");
define_regex!(website, r"(?i)\b(?:www\.[a-z0-9-]+(?:\.[a-z]{2,})+|[a-z0-9-]+(?:\.[a-z]{2,})+(?:\.to|\.co|\.tv|\.io|\.com|\.net|\.org))\b");
define_regex!(crc, r"(?i)\b[0-9a-f]{7,8}\b");

define_regex!(season_ep_range, r"(?i)\bs(\d{1,2})[._-]?e(\d{1,3})(?:[._-]+(?:e|ex|ep)?|[._-]*e|ex|ep|-)\s*(\d{1,3})\b");
define_regex!(season_ep, r"(?i)\bs(\d{1,2})[._-]?e(\d{1,3})\b");
define_regex!(season_ep_multi, r"(?i)\bs(\d{1,2})((?:[._-]?e\d{1,3}){2,})\b");
define_regex!(season_ep_x_range, r"(?i)\b(\d{1,2})x(\d{1,3})(?:[._-]*[xep]+|[._-]+)\s*(\d{1,3})\b");
define_regex!(season_ep_x_single, r"(?i)\b(\d{1,2})x(\d{1,3})\b");
define_regex!(season_range, r"(?i)\b(?:seasons?|saison|sezon)\s*(\d{1,2})\s*[-~to]\s*(\d{1,2})\b");
define_regex!(season_range_short, r"(?i)\bs(\d{1,2})\s*[-~to]\s*s?(\d{1,2})\b");
define_regex!(season_word, r"(?i)\b(?:season|saison|staffel|sezon|series?)\s*(\d{1,2})\b");
define_regex!(season_only, r"(?i)\bs(\d{1,2})\b");
define_regex!(ep_range, r"(?i)\b(?:episodes?|eps?|capitulos?)\s*(\d{1,3})\s*[-~to]\s*(\d{1,3})\b");
define_regex!(ep_range_short, r"(?i)\be(\d{1,3})\s*[-~to]\s*e?(\d{1,3})\b");
define_regex!(episode_word, r"(?i)\b(?:episode|ep|eps|capitulo)\s*(\d{1,3})\b");
define_regex!(ep_only, r"(?i)\be(\d{1,3})\b");
define_regex!(anime_ep_v, r"(?i)\b0*(\d{1,4})v\d+\b");

define_regex!(other_tags, r"(?i)\b(hdr10\+|hdr10|hdr|hlg|sdr|dv|dovi|dolbyvision|dolby-vision|10bit|10b|10-bit|8bit|8b|8-bit|3d|repack2|repack|proper|real|unrated|extended|complete|remastered|directors-cut|dc|imax|subbed|dubbed|hardsub|obfuscated|scrambled|hybrid|readnfo)\b");

// Static split regexes for delimiterless mode to avoid recompiling inside parse_segment
define_regex!(split_year, r"((?:19|20)\d{2})");
define_regex!(split_res, r"(?i)(2160[pi]?|1080[pi]?|720[pi]?|480[pi]?|4k|8k|uhd)");
define_regex!(split_codec, r"(?i)(av1|x[._-]?265|h[._-]?265|hevc|x[._-]?264|h[._-]?264|avc)");
define_regex!(split_channels, r"(?i)(?:^|[^0-9])(5\.1|7\.1|2\.0)(?:[^0-9]|$)");
define_regex!(split_source, r"(?i)(bluray|webdl|webrip|hdtv|remux)");
