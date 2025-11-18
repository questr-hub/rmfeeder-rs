use std::env;
use rmfeeder::{process_url_to_pdf};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: rmfeeder <url>");
        std::process::exit(1);
    }

    let url = &args[1];

    // Output file
    let output_path = "output.pdf";

    match process_url_to_pdf(url, output_path) {
        Ok(_) => println!("Wrote {}", output_path),
        Err(e) => eprintln!("Error: {}", e),
    }
}