use rizzit::parse;
use std::fs;

#[test]
fn audit_python_names_dataset() {
    let dataset_path = "/home/slim/Dev/python/testing/names.txt";
    let content = match fs::read_to_string(dataset_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let trimmed = content.trim();
    let inner = if trimmed.starts_with('[') && trimmed.ends_with(']') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

    let names: Vec<String> = inner
        .split("', '")
        .map(|s| s.trim_matches(|c| c == '\'' || c == '"').to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let mut missed_resolution_candidates = Vec::new();
    let mut missed_codec_candidates = Vec::new();
    let mut missed_year_candidates = Vec::new();

    for name in &names {
        let info = parse(name);

        // Check if resolution was missed even though string has resolution-like tokens
        if info.resolution.is_none() {
            let lower = name.to_lowercase();
            if lower.contains("1080") || lower.contains("720") || lower.contains("480") || lower.contains("2160") || lower.contains("4k") {
                missed_resolution_candidates.push(name.clone());
            }
        }

        // Check if codec was missed even though string has codec-like tokens
        if info.video_codec.is_none() {
            let lower = name.to_lowercase();
            if lower.contains("264") || lower.contains("265") || lower.contains("hevc") || lower.contains("avc") || lower.contains("av1") || lower.contains("xvid") {
                missed_codec_candidates.push(name.clone());
            }
        }

        // Check if year was missed even though string has 4-digit year-like numbers
        if info.year.is_none() {
            for word in name.split(|c: char| !c.is_numeric()) {
                if word.len() == 4 {
                    if let Ok(y) = word.parse::<u32>() {
                        if (1920..=2026).contains(&y) {
                            // Check if this year wasn't part of an episode number (like 0747 or 1080)
                            if y != 1080 && y != 2160 && !name.contains(&format!(" - {:04}", y)) && !name.contains(&format!(" - {}", y)) {
                                missed_year_candidates.push((name.clone(), y));
                            }
                        }
                    }
                }
            }
        }
    }

    println!("\n=======================================================");
    println!("AUDIT RESULTS (Out of {} filenames)", names.len());
    println!("=======================================================");
    println!("Filenames missing resolution where '1080/720/480/4k' appears: {}", missed_resolution_candidates.len());
    println!("Filenames missing codec where '264/265/hevc/avc' appears:      {}", missed_codec_candidates.len());
    println!("Potential missed years:                                      {}", missed_year_candidates.len());
    println!("=======================================================\n");

    if !missed_resolution_candidates.is_empty() {
        println!("Sample Missed Resolution Candidates:");
        for name in missed_resolution_candidates.iter().take(5) {
            println!("  - {}", name);
        }
    }

    if !missed_codec_candidates.is_empty() {
        println!("\nSample Missed Codec Candidates:");
        for name in missed_codec_candidates.iter().take(5) {
            println!("  - {}", name);
        }
    }

    if !missed_year_candidates.is_empty() {
        println!("\nSample Missed Year Candidates:");
        for (name, y) in missed_year_candidates.iter().take(5) {
            println!("  - {} (found year candidate {})", name, y);
        }
    }
}
