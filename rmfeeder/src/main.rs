use std::env;

use rmfeeder::multipdf;
use rmfeeder::process_url_to_pdf;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: rmfeeder <url1> [url2] [url3] ...");
        std::process::exit(1);
    }

    // Single article
    if args.len() == 1 {
        let url = &args[0];
        match process_url_to_pdf(url, "output.pdf") {
            Ok(_) => println!("Wrote output.pdf"),
            Err(e) => eprintln!("Error: {}", e),
        }

        return;
    }

    // Multi-article mode (TOC + article sections)
    match multipdf::generate_multi_pdf(&args, "output.pdf") {
        Ok(_) => println!("Wrote output.pdf"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
