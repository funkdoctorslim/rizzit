use rizzit::{parse, Parser, Resolution, Source, TorrentInfo, VideoCodec};
use std::path::Path;

#[test]
fn test_from_str_trait() {
    let info: TorrentInfo = "Inception.2010.1080p.BluRay.x264.mkv".parse().unwrap();
    assert_eq!(info.title, "Inception");
    assert_eq!(info.year, Some(2010));
    assert_eq!(info.resolution, Some("1080p".to_string()));
}

#[test]
fn test_is_movie_and_tv() {
    let movie = TorrentInfo::parse("Oppenheimer.2023.2160p.UHD.mkv");
    assert!(movie.is_movie());
    assert!(!movie.is_tv_show());
    assert_eq!(movie.full_title(), "Oppenheimer (2023)");

    let show = TorrentInfo::parse("Breaking.Bad.S05E14.Ozymandias.1080p.mkv");
    assert!(!show.is_movie());
    assert!(show.is_tv_show());
    assert_eq!(show.full_title(), "Breaking Bad S05E14");
    assert_eq!(show.season_num(), Some(5));
    assert_eq!(show.episode_num(), Some(14));
}

#[test]
fn test_typed_enums_and_normalized_filename() {
    let info = parse("Cyberpunk.Edgerunners.S01E02.Like.a.Boy.1080p.BluRay.x264-CRUCiBLE.mkv");
    assert_eq!(info.typed_video_codec(), Some(VideoCodec::H264));
    assert_eq!(info.typed_resolution(), Some(Resolution::R1080p));
    assert_eq!(info.typed_source(), Some(Source::Bluray));
    assert_eq!(
        info.normalized_filename(),
        "Cyberpunk.Edgerunners.S01E02.1080p.Blu-ray.H.264-CRUCiBLE.mkv"
    );
}

#[test]
fn test_builder_parser_options() {
    let path = "The Office US/Season 02/Sample.mkv";
    
    // Default inherits parent directory
    let info_default = parse(path);
    assert_eq!(info_default.title, "The Office US");

    // Disabled directory inheritance
    let custom_parser = Parser::new().inherit_parent_dir(false);
    let info_custom = custom_parser.parse(path);
    assert_eq!(info_custom.title, "Sample");
}

#[test]
fn test_as_ref_path_and_string_inputs() {
    let string_input = String::from("The.Matrix.1999.1080p.mkv");
    let path_input = Path::new("The.Matrix.1999.1080p.mkv");

    let info1 = parse(&string_input);
    let info2 = parse(path_input.to_str().unwrap());

    assert_eq!(info1.title, "The Matrix");
    assert_eq!(info2.title, "The Matrix");
}

#[test]
fn test_clean_serde_json_without_nulls() {
    let info = parse("Inception.2010.1080p.mkv");
    let json = info.to_json().unwrap();

    assert!(json.contains("\"title\":\"Inception\""));
    assert!(json.contains("\"year\":2010"));
    assert!(json.contains("\"resolution\":\"1080p\""));
    assert!(!json.contains("null")); // Ensure Option::is_none fields are omitted!
}
