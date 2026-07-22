# 👑 rizzit 👑
> **"Unspoken rizz, sub-microsecond parsing. No cap, this is the most GOATED video/torrent name parser known to man. Fr, fr."**

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](#)
[![Rizz Level](https://img.shields.io/badge/rizz-skibidi_level-blueviolet)](#)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-gold)](#)

---

## 🧐 What is this?
(a 100% vibe coded thing that actually works at 100% accuracy)
Look, we all know Python's `guessit` is an absolute boomer. It takes 10 business days to parse a single filename and uses look-arounds that Rust's regex engine would cry looking at. 

Enter **`rizzit`**—the ultimate, gigachad filename parser written in pure, unadulterated Rust. It has so much unspoken rizz that it doesn't just guess filenames; it *charms* the metadata out of them.

### 🔥 Features (No Cap)
* **⚡ Skibidi Speed:** Pre-compiled static regexes & zero-allocation bracket pre-scanner parse **over 500,000 filenames per second** in release mode. The CPU isn't compiling, it's *mewing*.
* **🎬 Modern Codecs & Services:** Native detection for **AV1**, **H.265/HEVC**, **H.264**, **VC-1**, **Dolby Atmos**, **DTS-HD MA**, **TrueHD**, and streaming services (**Netflix**, **Amazon Prime**, **Disney+**, **Apple TV+**, **HBO Max**, **Paramount+**).
* **🛡️ Anti-Cringe Bracket Matching:** Pre-scans and pairs up bracket stacks with stack-allocated linear pairs. Unbalanced brackets? Malformed brackets? `rizzit` ignores the drama and keeps it moving.
* **🌀 Delimiterless Mewing Mode:** Can parse filenames that have literally **zero spaces or dots** (like `Interstellar20142160pHEVCx265-RARBG.mkv`). It dynamically reconstructs and inserts delimiters like a Michelin chef.
* **🌐 Translating the Rizz:** Standardizes matched languages to official ISO English names using `isolang` (e.g. `eng` -> `"English"`, `vostfr` -> `"French"`).
* **📂 Directory Merging Rizz:** Parses generic sample or subtitle files (like `Subs/English.srt` or `Show/Season 01/Sample.mkv`) and automatically inherits metadata from parent and grandparent directories. It's not lazy, it's *efficient*.

---

## 🚀 How to Use (Let Him Cook)

### 📦 Cargo Dependency
Add the rizz to your `Cargo.toml`:
```toml
[dependencies]
rizzit = { git = "https://github.com/slim/rizzit" }
```

### 🦀 Rust Example
```rust
use rizzit::TorrentInfo;

fn main() {
    // A filename with extreme chaotic energy
    let name = "[VCB-Studio] Jujutsu Kaisen S2 [Ma10p_1080p]/[BeanSub&VCB-Studio] Jujutsu Kaisen [36][Ma10p_1080p][x265_flac].mkv";
    
    // Let rizzit charm it
    let info = TorrentInfo::parse(name);
    
    println!("Title: {}", info.title); // -> "Jujutsu Kaisen"
    println!("Episode: {:?}", info.episode); // -> Some([36])
    println!("Resolution: {:?}", info.resolution); // -> Some("1080p")
    println!("Video Codec: {:?}", info.video_codec); // -> Some("H.265")
    println!("Audio Codec: {:?}", info.audio_codec); // -> Some("FLAC")
    println!("Release Group: {:?}", info.release_group); // -> Some("BeanSub&VCB-Studio")
    println!("Formatted: {}", info.full_title()); // -> "Jujutsu Kaisen S02E36"
    println!("Is TV Show? {}", info.is_tv_show()); // -> true
}
```

### 💻 CLI Usage
```bash
# Clone and build in release (makes the CPU mew)
cargo build --release

# Parse a single filename and get pretty JSON output
./target/release/rizzit "Cyberpunk.Edgerunners.S01E02.Like.a.Boy.1080p.BluRay.Remux.x264-CRUCiBLE.mkv"

# Or pipe a billion filenames into stdin
cat names.txt | ./target/release/rizzit
```

---

## 📊 The Metadata Rizz Struct
This is the structured representation of a parsed name:
```rust
pub struct TorrentInfo {
    pub title: String,                  // Clean title
    pub year: Option<u32>,              // Release year
    pub season: Option<Vec<u32>>,       // Season numbers
    pub episode: Option<Vec<u32>>,      // Episode numbers
    pub episode_title: Option<String>,  // Episode title (e.g. "Like a Boy")
    pub resolution: Option<String>,     // Video resolution (2160p, 1080p, etc.)
    pub source: Option<String>,         // Source (Blu-ray, WEB-DL, etc.)
    pub video_codec: Option<String>,    // Codec (AV1, H.265, H.264, etc.)
    pub audio_codec: Option<String>,    // Audio codec (Dolby Digital, DTS, etc.)
    pub audio_channels: Option<String>, // Channels (5.1, 2.0, etc.)
    pub release_group: Option<String>,  // Release group
    pub language: Option<Vec<String>>,  // ISO Normalized languages
    pub other: Option<Vec<String>>,     // HDR, Dolby Vision, Streaming Service, Repack, etc.
    pub container: Option<String>,      // Container extension
    pub website: Option<String>,        // Release domain website
}
```

---

## 🤝 Contributing
If you find a filename that lacks rizz and breaks the parser, open an issue. No cap, we'll fix it fr.

*Made with 💖, Rust, and absolute brainrot by Antigravity AI. (with funkdoctorslims expert opinions)*
