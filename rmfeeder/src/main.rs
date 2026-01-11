use std::env;

use chrono::Local;
use rmfeeder::multipdf;
use rmfeeder::process_url_to_pdf;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let mut input_file: Option<String> = None;
    let mut output_path: Option<String> = None;
    let mut urls: Vec<String> = Vec::new();
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        if arg == "--output" {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("Error: --output requires a filename");
                std::process::exit(1);
            });
            output_path = Some(value);
        } else if arg == "--file" {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("Error: --file requires a path");
                std::process::exit(1);
            });
            input_file = Some(value);
        } else if let Some(value) = arg.strip_prefix("--output=") {
            output_path = Some(value.to_string());
        } else if let Some(value) = arg.strip_prefix("--file=") {
            input_file = Some(value.to_string());
        } else {
            urls.push(arg);
        }
    }

    if let Some(path) = input_file {
        let file = File::open(&path).unwrap_or_else(|e| {
            eprintln!("Error: failed to open {}: {}", path, e);
            std::process::exit(1);
        });
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap_or_else(|e| {
                eprintln!("Error: failed to read {}: {}", path, e);
                std::process::exit(1);
            });
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            urls.push(trimmed.to_string());
        }
    }

    if urls.is_empty() {
        eprintln!("Usage: rmfeeder [--output <file.pdf>] [--file <path>] <url1> [url2] [url3] ...");
        std::process::exit(1);
    }

    let output_path = output_path.unwrap_or_else(|| {
        format!("{}.pdf", Local::now().format("%Y-%m-%d-%H-%M-%S"))
    });

    // Single article
    if urls.len() == 1 {
        let url = &urls[0];
        match process_url_to_pdf(url, &output_path) {
            Ok(_) => println!("Wrote {}", output_path),
            Err(e) => eprintln!("Error: {}", e),
        }

        return;
    }

    // Multi-article mode (TOC + article sections)
    match multipdf::generate_multi_pdf(&urls, &output_path) {
        Ok(_) => println!("Wrote {}", output_path),
        Err(e) => eprintln!("Error: {}", e),
    }
}
