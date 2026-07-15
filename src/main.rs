use std::env;
use std::io::{self, BufRead};
use rizzit::TorrentInfo;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        // Parse the provided argument
        let filename = &args[1];
        let info = TorrentInfo::parse(filename);
        match info.to_json_pretty() {
            Ok(json) => println!("{}", json),
            Err(e) => {
                eprintln!("Error serializing to JSON: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Check if stdin is a TTY. If it is, print usage. Otherwise, process stdin line-by-line.
        // Wait, under linux we can check if it's a tty or just read lines. Let's read lines:
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let mut line = String::new();
        
        // Try reading first line to see if there's input
        if let Ok(bytes) = reader.read_line(&mut line) {
            if bytes == 0 {
                // Stdin is empty, print usage
                print_usage();
                return;
            }
            // Parse first line
            let info = TorrentInfo::parse(line.trim());
            match info.to_json() {
                Ok(json) => println!("{}", json),
                Err(e) => eprintln!("Error: {}", e),
            }
            line.clear();
            
            // Parse remaining lines
            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                let info = TorrentInfo::parse(line.trim());
                match info.to_json() {
                    Ok(json) => println!("{}", json),
                    Err(e) => eprintln!("Error: {}", e),
                }
                line.clear();
            }
        } else {
            print_usage();
        }
    }
}

fn print_usage() {
    println!("rizzit: The GOATED torrent name parser in Rust");
    println!("Usage:");
    println!("  rizzit \"<filename>\"   Parse a single filename and print pretty JSON");
    println!("  echo \"<filename>\" | rizzit   Parse lines from stdin and print JSON");
}
