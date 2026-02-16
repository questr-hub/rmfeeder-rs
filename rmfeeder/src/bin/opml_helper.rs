use std::collections::{BTreeSet, HashSet};
use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use feed_rs::parser;
use rmfeeder::{expand_tilde_path, load_config_from_path};
use reqwest::blocking::Client;
use roxmltree::Document;
use rusqlite::{params, Connection, OptionalExtension};

fn main() {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    let config_path = extract_config_path(&raw_args).unwrap_or_else(|| "rmfeeder.toml".to_string());

    let config = match load_config_from_path(&config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error: failed to load config {}: {}", config_path, e);
            std::process::exit(1);
        }
    };

    let mut output_path: Option<String> = config.as_ref().and_then(|c| c.urls_path.clone());
    let mut limit: usize = config.as_ref().and_then(|c| c.limit).unwrap_or(3);
    let mut use_state = true;
    let mut clear_state = false;
    let mut opml_path: Option<String> = config.as_ref().and_then(|c| c.feeds_opml_path.clone());
    let mut state_db_path: Option<String> = config.as_ref().and_then(|c| c.state_db_path.clone());

    let mut args = raw_args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--config" {
            if args.next().is_none() {
                eprintln!("Error: --config requires a path");
                std::process::exit(1);
            }
        } else if arg.starts_with("--config=") {
            continue;
        } else if arg == "--output" {
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
        } else if arg == "--no-state" {
            use_state = false;
        } else if arg == "--clear-state" {
            clear_state = true;
        } else if opml_path.is_none() {
            opml_path = Some(arg);
        } else {
            eprintln!("Error: unexpected argument: {}", arg);
            std::process::exit(1);
        }
    }

    let opml_path = opml_path.unwrap_or_else(|| {
        eprintln!(
            "Usage: rmfeeder-opml [--config <path>] [--limit N] [--output path] [--no-state] [--clear-state] <feeds.opml>"
        );
        std::process::exit(1);
    });

    if !use_state && clear_state {
        eprintln!("Error: --no-state cannot be used with --clear-state");
        std::process::exit(1);
    }

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

    let mut state = if use_state {
        Some(init_state_db(clear_state, state_db_path.take()))
    } else {
        None
    };

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
                    if let Some(ref mut db) = state {
                        match db.should_emit(&link) {
                            Ok(true) => {}
                            Ok(false) => continue,
                            Err(e) => {
                                eprintln!("Warning: state check failed: {}", e);
                            }
                        }
                    }

                    if let Err(e) = writeln!(out, "{}", link) {
                        eprintln!("Error: failed to write output: {}", e);
                        std::process::exit(1);
                    }

                    if let Some(ref mut db) = state {
                        if let Err(e) = db.mark_seen(&link) {
                            eprintln!("Warning: failed to update state: {}", e);
                        }
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

struct StateDb {
    conn: Connection,
    seen_in_run: HashSet<String>,
}

impl StateDb {
    fn should_emit(&mut self, url: &str) -> rusqlite::Result<bool> {
        if self.seen_in_run.contains(url) {
            return Ok(false);
        }

        let exists = self
            .conn
            .query_row("SELECT 1 FROM seen WHERE url = ?1 LIMIT 1", [url], |_| {
                Ok(())
            })
            .optional()?
            .is_some();

        Ok(!exists)
    }

    fn mark_seen(&mut self, url: &str) -> rusqlite::Result<()> {
        if self.seen_in_run.contains(url) {
            return Ok(());
        }

        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT OR IGNORE INTO seen (url, seen_at) VALUES (?1, ?2)",
            params![url, now],
        )?;
        self.seen_in_run.insert(url.to_string());
        Ok(())
    }
}

fn init_state_db(clear_state: bool, custom_path: Option<String>) -> StateDb {
    let path = match custom_path {
        Some(path) => expand_tilde_path(&path),
        None => default_state_path().unwrap_or_else(|e| {
            eprintln!("Error: failed to resolve state path: {}", e);
            std::process::exit(1);
        }),
    };

    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("Error: failed to create state directory {}: {}", parent.display(), e);
            std::process::exit(1);
        }
    }

    let conn = Connection::open(&path).unwrap_or_else(|e| {
        eprintln!("Error: failed to open state DB {}: {}", path.display(), e);
        std::process::exit(1);
    });

    if let Err(e) = conn.execute(
        "CREATE TABLE IF NOT EXISTS seen (url TEXT PRIMARY KEY, seen_at INTEGER NOT NULL)",
        [],
    ) {
        eprintln!("Error: failed to initialize state DB: {}", e);
        std::process::exit(1);
    }

    if clear_state {
        if let Err(e) = conn.execute("DELETE FROM seen", []) {
            eprintln!("Error: failed to clear state DB: {}", e);
            std::process::exit(1);
        }
    }

    StateDb {
        conn,
        seen_in_run: HashSet::new(),
    }
}

fn default_state_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = env::var("HOME")?;
    Ok(Path::new(&home)
        .join(".local")
        .join("share")
        .join("rmfeeder")
        .join("rmfeeder_state.sqlite"))
}

fn extract_config_path(args: &[String]) -> Option<String> {
    let mut i = 0usize;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--config" {
            if i + 1 >= args.len() {
                eprintln!("Error: --config requires a path");
                std::process::exit(1);
            }
            return Some(args[i + 1].clone());
        }
        if let Some(value) = arg.strip_prefix("--config=") {
            return Some(value.to_string());
        }
        i += 1;
    }
    None
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
