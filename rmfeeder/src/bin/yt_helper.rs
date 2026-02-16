use std::collections::HashSet;
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use chrono::Local;
use pulldown_cmark::{html, Options, Parser};
use rmfeeder::{escape_html, expand_tilde_path, load_config_from_path, multipdf};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;

const WATCH_LATER_URL: &str = "https://www.youtube.com/playlist?list=WL";

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

    let mut mode_watch_later = false;
    let mut output_path: Option<String> = None;
    let mut limit: usize = config.as_ref().and_then(|c| c.yt_limit).unwrap_or(10);
    let mut pattern: String = config
        .as_ref()
        .and_then(|c| c.yt_pattern.clone())
        .unwrap_or_else(|| "youtube_summary".to_string());
    let mut delay_secs: u64 = config.as_ref().and_then(|c| c.yt_delay).unwrap_or(0);
    let mut cookies_browser: String = config
        .as_ref()
        .and_then(|c| c.yt_cookies_browser.clone())
        .unwrap_or_else(|| "chrome".to_string());
    let mut mark_watched_on_success: bool = config
        .as_ref()
        .and_then(|c| c.yt_mark_watched_on_success)
        .unwrap_or(true);
    let mut dry_run = false;
    let mut clear_state = false;
    let mut output_dir: Option<String> = config.as_ref().and_then(|c| c.output_dir.clone());
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
        } else if arg == "--watch-later" {
            mode_watch_later = true;
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
        } else if arg == "--pattern" {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("Error: --pattern requires a name");
                std::process::exit(1);
            });
            pattern = value;
        } else if let Some(value) = arg.strip_prefix("--pattern=") {
            pattern = value.to_string();
        } else if arg == "--delay" {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("Error: --delay requires a number");
                std::process::exit(1);
            });
            delay_secs = parse_delay(&value);
        } else if let Some(value) = arg.strip_prefix("--delay=") {
            delay_secs = parse_delay(value);
        } else if arg == "--cookies-from-browser" {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("Error: --cookies-from-browser requires a browser name");
                std::process::exit(1);
            });
            cookies_browser = value;
        } else if let Some(value) = arg.strip_prefix("--cookies-from-browser=") {
            cookies_browser = value.to_string();
        } else if arg == "--dry-run" {
            dry_run = true;
        } else if arg == "--clear-state" {
            clear_state = true;
        } else {
            eprintln!("Error: unexpected argument: {}", arg);
            print_usage_and_exit();
        }
    }

    if !mode_watch_later {
        print_usage_and_exit();
    }

    if dry_run {
        mark_watched_on_success = false;
    }

    if clear_state && dry_run {
        eprintln!("Error: --clear-state cannot be used with --dry-run");
        std::process::exit(1);
    }

    let output_path = output_path.unwrap_or_else(|| {
        let filename = format!("yt-watchlist-{}.pdf", Local::now().format("%Y-%m-%d-%H-%M-%S"));
        if let Some(dir) = output_dir.take() {
            Path::new(&dir).join(filename).to_string_lossy().to_string()
        } else {
            filename
        }
    });

    eprintln!("Fetching Watch Later list...");
    let videos = match fetch_watch_later(&cookies_browser) {
        Ok(videos) => videos,
        Err(e) => {
            eprintln!("Error: failed to fetch watch list: {}", e);
            std::process::exit(1);
        }
    };

    if videos.is_empty() {
        eprintln!("Error: no videos found in Watch Later");
        std::process::exit(1);
    }
    eprintln!("Mode: yt-watchlist-summary");
    eprintln!("Pattern: {}", pattern);

    let mut state = if dry_run {
        None
    } else {
        Some(init_state_db(clear_state, state_db_path.take()))
    };

    let mut attempted = 0usize;
    let mut included = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;
    let mut articles: Vec<(String, String)> = Vec::new();

    for video in videos {
        if included >= limit {
            break;
        }

        attempted += 1;
        eprintln!("Processing {}", video.url);

        let state_key = format!("yt::{}", video.url);
        if let Some(ref mut db) = state {
            match db.should_emit(&state_key) {
                Ok(false) => {
                    skipped += 1;
                    eprintln!("Skipping {}: already seen", video.url);
                    continue;
                }
                Ok(true) => {}
                Err(e) => {
                    eprintln!("Warning: state check failed for {}: {}", video.url, e);
                }
            }
        }

        let summary_markdown = match run_fabric_youtube(&video.url, &pattern) {
            Ok(text) => text,
            Err(e) => {
                failed += 1;
                eprintln!("Skipping {}: summary failed: {}", video.url, e);
                continue;
            }
        };

        let summary_html = markdown_to_html(&summary_markdown);
        let safe_url = escape_html(&video.url);
        let body_html = format!(
            "<p class=\"article-source\">Source: <a href=\"{url}\">{url}</a></p>\n{body}",
            url = safe_url,
            body = summary_html
        );

        articles.push((video.title.clone(), body_html));
        included += 1;

        if let Some(ref mut db) = state {
            if let Err(e) = db.mark_seen(&state_key) {
                eprintln!("Warning: failed to update state for {}: {}", video.url, e);
            }
        }

        if mark_watched_on_success {
            if let Err(e) = mark_watched(&cookies_browser, &video.url) {
                eprintln!("Warning: failed to mark watched {}: {}", video.url, e);
            }
        }

        if delay_secs > 0 {
            thread::sleep(Duration::from_secs(delay_secs));
        }
    }

    if articles.is_empty() {
        eprintln!("Error: no videos were included in output");
        eprintln!(
            "Summary: attempted={} included={} skipped={} failed={}",
            attempted, included, skipped, failed
        );
        std::process::exit(1);
    }

    if let Err(e) = multipdf::generate_pdf_bundle(
        &articles,
        &output_path,
        "rmfeeder ::",
        "YouTube Watchlist",
    ) {
        eprintln!("Error: failed to generate PDF: {}", e);
        std::process::exit(1);
    }

    eprintln!(
        "Summary: attempted={} included={} skipped={} failed={}",
        attempted, included, skipped, failed
    );
    println!("Wrote {}", output_path);
}

fn print_usage_and_exit() -> ! {
    eprintln!(
        "Usage: yt_helper --watch-later [--config <path>] [--output <file.pdf>] [--limit N] [--pattern <name>] [--delay N] [--cookies-from-browser <name>] [--dry-run] [--clear-state]"
    );
    eprintln!(
        "  --dry-run: Generate PDF without side effects (no local state read/write, no mark-watched updates)."
    );
    eprintln!(
        "  Note: Watch Later filtering is local-state based; remove items manually in YouTube UI when desired."
    );
    std::process::exit(1);
}

fn parse_limit(value: &str) -> usize {
    value.parse::<usize>().unwrap_or_else(|_| {
        eprintln!("Error: --limit must be a positive number");
        std::process::exit(1);
    })
}

fn parse_delay(value: &str) -> u64 {
    value.parse::<u64>().unwrap_or_else(|_| {
        eprintln!("Error: --delay must be a non-negative number");
        std::process::exit(1);
    })
}

#[derive(Debug, Clone)]
struct YtVideo {
    title: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct YtPlaylist {
    entries: Vec<YtEntry>,
}

#[derive(Debug, Deserialize)]
struct YtEntry {
    title: Option<String>,
    webpage_url: Option<String>,
    url: Option<String>,
    id: Option<String>,
}

fn fetch_watch_later(cookies_browser: &str) -> Result<Vec<YtVideo>, Box<dyn std::error::Error>> {
    let output = Command::new("yt-dlp")
        .arg("--cookies-from-browser")
        .arg(cookies_browser)
        .arg("--flat-playlist")
        .arg("--dump-single-json")
        .arg(WATCH_LATER_URL)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp failed: {}", stderr.trim()).into());
    }

    let payload = String::from_utf8_lossy(&output.stdout);
    let playlist: YtPlaylist = serde_json::from_str(&payload)?;

    let mut out = Vec::new();
    for entry in playlist.entries {
        let url = resolve_video_url(&entry);
        let title = entry.title.unwrap_or_else(|| "Untitled Video".to_string());
        if let Some(url) = url {
            out.push(YtVideo { title, url });
        }
    }
    Ok(out)
}

fn resolve_video_url(entry: &YtEntry) -> Option<String> {
    if let Some(url) = &entry.webpage_url {
        return Some(url.clone());
    }
    if let Some(url) = &entry.url {
        if url.starts_with("http://") || url.starts_with("https://") {
            return Some(url.clone());
        }
        return Some(format!("https://www.youtube.com/watch?v={}", url));
    }
    entry
        .id
        .as_ref()
        .map(|id| format!("https://www.youtube.com/watch?v={}", id))
}

fn run_fabric_youtube(url: &str, pattern: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("fabric-ai")
        .arg("-y")
        .arg(url)
        .arg("--pattern")
        .arg(pattern)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("fabric-ai failed: {}", stderr.trim()).into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn markdown_to_html(input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(input, options);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

fn mark_watched(cookies_browser: &str, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("yt-dlp")
        .arg("--cookies-from-browser")
        .arg(cookies_browser)
        .arg("--mark-watched")
        .arg("--skip-download")
        .arg("--no-warnings")
        .arg("--quiet")
        .arg(url)
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp mark-watched failed: {}", stderr.trim()).into());
    }
    Ok(())
}

struct StateDb {
    conn: Connection,
    seen_in_run: HashSet<String>,
}

impl StateDb {
    fn should_emit(&mut self, key: &str) -> rusqlite::Result<bool> {
        if self.seen_in_run.contains(key) {
            return Ok(false);
        }

        let exists = self
            .conn
            .query_row("SELECT 1 FROM seen WHERE url = ?1 LIMIT 1", [key], |_| Ok(()))
            .optional()?
            .is_some();
        Ok(!exists)
    }

    fn mark_seen(&mut self, key: &str) -> rusqlite::Result<()> {
        if self.seen_in_run.contains(key) {
            return Ok(());
        }
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT OR IGNORE INTO seen (url, seen_at) VALUES (?1, ?2)",
            params![key, now],
        )?;
        self.seen_in_run.insert(key.to_string());
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

fn default_state_path() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
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
