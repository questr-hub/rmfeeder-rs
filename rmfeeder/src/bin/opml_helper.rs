use std::collections::BTreeSet;
use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Write};

use feed_rs::parser;
use reqwest::blocking::Client;
use roxmltree::Document;

fn main() {
    let mut output_path: Option<String> = None;
    let mut limit: usize = 3;
    let mut opml_path: Option<String> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--output" {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("Error: --output requires a path");
                std::process::exit(1);
            });
            output_path = Some(value);
        } else if let Some(value) = arg.strip_prefix("--output=") {
            output_path = Some(value.to_string());
        } else if arg == "--limit" {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("Error: --limit requires a number");
                std::process::exit(1);
            });
            limit = parse_limit(&value);
        } else if let Some(value) = arg.strip_prefix("--limit=") {
            limit = parse_limit(value);
        } else if opml_path.is_none() {
            opml_path = Some(arg);
        } else {
            eprintln!("Error: unexpected argument: {}", arg);
            std::process::exit(1);
        }
    }

    let opml_path = opml_path.unwrap_or_else(|| {
        eprintln!("Usage: rmfeeder-opml [--limit N] [--output path] <feeds.opml>");
        std::process::exit(1);
    });

    let feed_urls = match load_opml_feed_urls(&opml_path) {
        Ok(urls) => urls,
        Err(e) => {
            eprintln!("Error: failed to parse OPML: {}", e);
            std::process::exit(1);
        }
    };

    if feed_urls.is_empty() {
        eprintln!("Error: no feed URLs found in {}", opml_path);
        std::process::exit(1);
    }

    let client = Client::builder()
        .user_agent("rmfeeder-opml/0.1 (+https://example.com)")
        .build()
        .unwrap_or_else(|e| {
            eprintln!("Error: failed to build HTTP client: {}", e);
            std::process::exit(1);
        });

    let mut out: Box<dyn Write> = match output_path {
        Some(path) => {
            let file = File::create(&path).unwrap_or_else(|e| {
                eprintln!("Error: failed to write {}: {}", path, e);
                std::process::exit(1);
            });
            Box::new(BufWriter::new(file))
        }
        None => Box::new(BufWriter::new(io::stdout())),
    };

    for feed_url in feed_urls {
        match fetch_feed_links(&client, &feed_url, limit) {
            Ok(links) => {
                for link in links {
                    if let Err(e) = writeln!(out, "{}", link) {
                        eprintln!("Error: failed to write output: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: {}: {}", feed_url, e);
            }
        }
    }
}

fn parse_limit(value: &str) -> usize {
    value.parse::<usize>().unwrap_or_else(|_| {
        eprintln!("Error: --limit must be a positive number");
        std::process::exit(1);
    })
}

fn load_opml_feed_urls(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let doc = Document::parse(&content)?;
    let mut urls = BTreeSet::new();

    for node in doc.descendants().filter(|n| n.has_tag_name("outline")) {
        if let Some(url) = node.attribute("xmlUrl") {
            urls.insert(url.to_string());
        }
    }

    Ok(urls.into_iter().collect())
}

fn fetch_feed_links(
    client: &Client,
    feed_url: &str,
    limit: usize,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let res = client.get(feed_url).send()?.error_for_status()?;
    let bytes = res.bytes()?;
    let feed = parser::parse(bytes.as_ref())?;

    let mut entries = feed.entries;
    entries.sort_by(|a, b| entry_timestamp(b).cmp(&entry_timestamp(a)));

    let mut out = Vec::new();
    for entry in entries.into_iter().take(limit) {
        if let Some(link) = pick_entry_link(&entry) {
            out.push(link);
        }
    }

    Ok(out)
}

fn entry_timestamp(entry: &feed_rs::model::Entry) -> i64 {
    let dt = entry
        .published
        .or(entry.updated)
        .map(|d| d.timestamp())
        .unwrap_or(0);
    dt
}

fn pick_entry_link(entry: &feed_rs::model::Entry) -> Option<String> {
    if let Some(link) = entry
        .links
        .iter()
        .find(|l| l.rel.as_deref() == Some("alternate"))
    {
        return Some(link.href.clone());
    }
    entry.links.first().map(|l| l.href.clone())
}
