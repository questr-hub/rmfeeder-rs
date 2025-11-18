use std::env;
use std::fs;

use rmfeeder::process_url;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rmfeeder <url>");
        std::process::exit(1);
    }

    let url = &args[1];

    let output = process_url(url);

    fs::write("output.html", output)
        .expect("Failed to write output.html");

    println!("Wrote output.html");
}