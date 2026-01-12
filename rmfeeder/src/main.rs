use std::env;

use chrono::Local;
use rmfeeder::multipdf;
use rmfeeder::process_url_to_pdf_with_options;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let mut input_file: Option<String> = None;
    let mut output_path: Option<String> = None;
    let mut delay_secs: u64 = 0;
    let mut summarize = false;
    let mut pattern: Option<String> = None;
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
        } else if arg == "--delay" {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("Error: --delay requires a number");
                std::process::exit(1);
            });
            delay_secs = parse_delay(&value);
        } else if arg == "--summarize" {
            summarize = true;
        } else if arg == "--pattern" {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("Error: --pattern requires a name");
                std::process::exit(1);
            });
            pattern = Some(value);
            summarize = true;
        } else if let Some(value) = arg.strip_prefix("--output=") {
            output_path = Some(value.to_string());
        } else if let Some(value) = arg.strip_prefix("--file=") {
            input_file = Some(value.to_string());
        } else if let Some(value) = arg.strip_prefix("--delay=") {
            delay_secs = parse_delay(value);
        } else if let Some(value) = arg.strip_prefix("--pattern=") {
            pattern = Some(value.to_string());
            summarize = true;
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
        eprintln!(
            "Usage: rmfeeder [--output <file.pdf>] [--file <path>] [--delay N] [--summarize] [--pattern <name>] <url1> [url2] [url3] ..."
        );
        std::process::exit(1);
    }

    let output_path = output_path.unwrap_or_else(|| {
        format!("{}.pdf", Local::now().format("%Y-%m-%d-%H-%M-%S"))
    });
    let pattern = pattern.unwrap_or_else(|| "summarize".to_string());

    // Single article
    if urls.len() == 1 {
        let url = &urls[0];
        match process_url_to_pdf_with_options(url, &output_path, summarize, &pattern) {
            Ok(_) => println!("Wrote {}", output_path),
            Err(e) => eprintln!("Error: {}", e),
        }

        return;
    }

    // Multi-article mode (TOC + article sections)
    match multipdf::generate_multi_pdf(&urls, &output_path, delay_secs, summarize, &pattern) {
        Ok(_) => println!("Wrote {}", output_path),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn parse_delay(value: &str) -> u64 {
    value.parse::<u64>().unwrap_or_else(|_| {
        eprintln!("Error: --delay must be a non-negative number");
        std::process::exit(1);
    })
}
