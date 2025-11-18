use std::env;

mod pdf;
mod multipdf;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: rmfeeder <url1> [url2] [url3] ...");
        std::process::exit(1);
    }

    // Single article
    if args.len() == 1 {
        let url = &args[0];
        let title = "Article";  // Will be overridden by extractor anyway

        match pdf::generate_pdf(title, url, "output.pdf") {
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