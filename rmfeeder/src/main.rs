use std::env;

use chrono::Local;
use rmfeeder::multipdf;
use rmfeeder::process_url_to_pdf;

fn main() {
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
        } else if let Some(value) = arg.strip_prefix("--output=") {
            output_path = Some(value.to_string());
        } else {
            urls.push(arg);
        }
    }

    if urls.is_empty() {
        eprintln!("Usage: rmfeeder [--output <file.pdf>] <url1> [url2] [url3] ...");
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
