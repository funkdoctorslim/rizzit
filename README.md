# rizzit 🎬

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](#)
[![License](https://img.shields.io/badge/license-Unlicense-blue)](#)
[![Crate](https://img.shields.io/badge/crates.io-rizzit-orange)](#)

> A fast, vibe-coded media and torrent filename parser written in pure Rust.

`rizzit` extracts structured metadata (title, release year, season/episode numbers, resolution, video/audio codecs, audio channels, release groups, and quality tags) from raw video, torrent, and anime file paths.

---

## 🔥 Key Features

- ⚡ **High Performance**: Parses over **120,000 filenames per second** per thread in release mode.
- 🍿 **Media Specific**: Purpose-built for movies, TV series, anime releases, and media collections.
- 🎬 **Modern Codec & Service Detection**: Built-in support for **AV1**, **H.265/HEVC**, **H.264**, **VC-1**, **Dolby Atmos**, **DTS-HD MA**, **TrueHD**, and streaming services (**Netflix**, **Amazon Prime**, **Disney+**, **Apple TV+**, **HBO Max**, **Paramount+**).
- 🌀 **Delimiterless Mode**: Parses filenames with zero spaces or dots (e.g. `Interstellar20142160pHEVCx265-RARBG.mkv`) by reconstructing word boundaries dynamically.
- 📂 **Directory Hierarchy Inheritance**: Automatically inherits metadata from parent and grandparent directories for generic sample or subtitle files (e.g., `Show Name / Season 01 / Sample.mkv`).
- 🌐 **ISO Language Normalization**: Converts language tags (e.g. `eng`, `fre`, `vostfr`) into standard ISO English names using `isolang`.
- 🛠️ **Strongly-Typed Enums**: Provides `VideoCodec`, `Resolution`, and `Source` enums for type-safe pattern matching without string allocations.

---

## 📦 Installation

Add `rizzit` to your `Cargo.toml`:

```toml
[dependencies]
rizzit = { git = "https://github.com/funkdoctorslim/rizzit" }
```

---

## 💻 Examples

### 1. Basic Movie & TV Show Parsing

```rust
use rizzit::TorrentInfo;

fn main() {
    let name = "Cyberpunk.Edgerunners.S01E02.Like.a.Boy.1080p.BluRay.Remux.x264-CRUCiBLE.mkv";
    let info = TorrentInfo::parse(name);

    println!("Title:         {}", info.title);           // "Cyberpunk Edgerunners"
    println!("Full Title:    {}", info.full_title());      // "Cyberpunk Edgerunners S01E02"
    println!("Season:        {:?}", info.season);          // Some([1])
    println!("Episode:       {:?}", info.episode);         // Some([2])
    println!("Episode Title: {:?}", info.episode_title);  // Some("Like a Boy")
    println!("Resolution:    {:?}", info.resolution);     // Some("1080p")
    println!("Source:        {:?}", info.source);         // Some("Blu-ray")
    println!("Codec:         {:?}", info.video_codec);     // Some("H.264")
    println!("Group:         {:?}", info.release_group);  // Some("CRUCiBLE")
}
```

### 2. Anime & Subtitle Tags

```rust
use rizzit::TorrentInfo;

fn main() {
    let name = "[SubsPlease] Kono Subarashii Sekai ni Shukufuku wo! 3 - 01 (1080p) [F109283].mkv";
    let info = TorrentInfo::parse(name);

    println!("Title:         {}", info.title);          // "Kono Subarashii Sekai ni Shukufuku wo! 3"
    println!("Episode:       {:?}", info.episode);        // Some([1])
    println!("Group:         {:?}", info.release_group); // Some("SubsPlease")
    println!("Is Anime?      {}", info.is_anime());      // true
    println!("CRC32 / Tags:  {:?}", info.other);         // Some(["CRC32:F109283"])
}
```

### 3. Delimiterless Filenames (No Spaces or Dots)

```rust
use rizzit::parse;

fn main() {
    let raw = "Interstellar20142160pHEVCx265-RARBG.mkv";
    let info = parse(raw);

    assert_eq!(info.title, "Interstellar");
    assert_eq!(info.year, Some(2014));
    assert_eq!(info.resolution, Some("2160p".to_string()));
    assert_eq!(info.video_codec, Some("H.265".to_string()));
    assert_eq!(info.release_group, Some("RARBG".to_string()));
}
```

### 4. Directory Metadata Inheritance

```rust
use rizzit::parse;

fn main() {
    // Generic file inside a structured directory path
    let path = "The Office US/Season 02/Sample.mkv";
    let info = parse(path);

    println!("Inherited Title:  {}", info.title);    // "The Office US"
    println!("Inherited Season: {:?}", info.season);  // Some([2])
}
```

### 5. Strongly-Typed Enums & Normalized Filename Output

```rust
use rizzit::{parse, VideoCodec, Resolution, Source};

fn main() {
    let info = parse("Stranger.Things.S04E01.2160p.NF.WEB-DL.AV1.DV.HDR.Atmos-FLUX.mkv");

    // Match typed enums directly
    if info.typed_video_codec() == Some(VideoCodec::AV1) {
        println!("Encoder: AV1 modern codec");
    }

    if info.typed_source() == Some(Source::WebDl) {
        println!("Source: WEB-DL");
    }

    // Format a standard scene filename
    println!("Normalized: {}", info.normalized_filename());
    // -> "Stranger.Things.S04E01.2160p.WEB-DL.AV1-FLUX.mkv"
}
```

### 6. Customizing Parser Settings

```rust
use rizzit::Parser;

fn main() {
    // Configure parser behavior with custom options
    let custom_parser = Parser::new()
        .inherit_parent_dir(false)  // Disable parent folder inheritance
        .delimiterless_mode(true);

    let info = custom_parser.parse("The Office US/Season 02/Sample.mkv");
    println!("Title: {}", info.title); // "Sample"
}
```

---

## 💻 CLI Usage

`rizzit` includes a lightweight command-line executable:

```bash
# Build in release mode
cargo build --release

# Parse a single filename (prints pretty-printed JSON)
./target/release/rizzit "Cyberpunk.Edgerunners.S01E02.1080p.BluRay.x264-CRUCiBLE.mkv"

# Stream filenames from stdin (prints compact JSON per line)
cat filenames.txt | ./target/release/rizzit
```

---

## 📊 TorrentInfo Struct

```rust
pub struct TorrentInfo {
    pub title: String,                  // Clean media title
    pub year: Option<u32>,              // Release year (e.g. 2024)
    pub season: Option<Vec<u32>>,       // Season numbers (e.g. [1])
    pub episode: Option<Vec<u32>>,      // Episode numbers (e.g. [2])
    pub episode_title: Option<String>,  // Episode title (e.g. "Like a Boy")
    pub resolution: Option<String>,     // Video resolution (2160p, 1080p, 720p, etc.)
    pub source: Option<String>,         // Source (Blu-ray, WEB-DL, WEBRip, etc.)
    pub video_codec: Option<String>,    // Video codec (AV1, H.265, H.264, etc.)
    pub audio_codec: Option<String>,    // Audio codec (Dolby Atmos, DTS-HD, FLAC, etc.)
    pub audio_channels: Option<String>, // Audio channels (7.1, 5.1, 2.0, etc.)
    pub release_group: Option<String>,  // Release group (e.g. RARBG, SubsPlease)
    pub language: Option<Vec<String>>,  // ISO-normalized languages (e.g. ["English"])
    pub other: Option<Vec<String>>,     // HDR, Dolby Vision, Netflix, Repack, etc.
    pub container: Option<String>,      // Container extension (mkv, mp4, etc.)
    pub website: Option<String>,        // Release website domain
}
```

---

## 📄 License

This is free and unencumbered software released into the public domain under the [Unlicense](UNLICENSE).
