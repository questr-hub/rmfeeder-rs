use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::thread;
use std::time::Duration;

use chrono::Local;
use reqwest::StatusCode;
use reqwest::blocking::Client;
use rmfeeder::multipdf;
use rmfeeder::{
    PageSize, default_config_path, default_feeds_opml_path, extractor, feeds, fetcher,
    load_config_from_path, process_url_to_pdf_with_options, state, summarize_html, youtube,
};

struct UrlCandidate {
    url: String,
    source: &'static str,
    use_seen_state: bool,
    toc_section: Option<String>,
}

fn main() {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    let config_path = extract_config_path(&raw_args)
        .unwrap_or_else(|| default_config_path().to_string_lossy().to_string());

    let config = match load_config_from_path(&config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error: failed to load config {}: {}", config_path, e);
            std::process::exit(1);
        }
    };

    let mut input_file: Option<String> = None;
    let mut output_path: Option<String> = None;
    let mut output_dir: Option<String> = config.as_ref().and_then(|c| c.output_dir.clone());
    let mut delay_secs: u64 = config.as_ref().and_then(|c| c.delay).unwrap_or(0);
    let mut summarize = config.as_ref().and_then(|c| c.summarize).unwrap_or(false);
    let mut pattern: String = config
        .as_ref()
        .and_then(|c| c.pattern.clone())
        .unwrap_or_else(|| "summarize".to_string());
    let mut page_size = config
        .as_ref()
        .and_then(|c| c.page_size.as_deref())
        .map(parse_page_size)
        .unwrap_or(PageSize::Letter);

    let mut feeds_enabled = false;
    let mut yt_watchlist_enabled = false;
    let mut clear_state = false;

    let mut feeds_limit: usize = config.as_ref().and_then(|c| c.limit).unwrap_or(3);
    let mut opml_path: Option<String> = config.as_ref().and_then(|c| c.feeds_opml_path.clone());

    let mut yt_limit: usize = config.as_ref().and_then(|c| c.yt_limit).unwrap_or(10);
    let mut yt_pattern: String = config
        .as_ref()
        .and_then(|c| c.yt_pattern.clone())
        .unwrap_or_else(|| "youtube_summary".to_string());
    let yt_delay: u64 = config.as_ref().and_then(|c| c.yt_delay).unwrap_or(0);
    let mut yt_cookies_browser: String = config
        .as_ref()
        .and_then(|c| c.yt_cookies_browser.clone())
        .unwrap_or_else(|| "chrome".to_string());
    let mut yt_mark_watched_on_success: bool = config
        .as_ref()
        .and_then(|c| c.yt_mark_watched_on_success)
        .unwrap_or(true);

    let mut state_db_path: Option<String> = config.as_ref().and_then(|c| c.state_db_path.clone());

    let mut direct_urls: Vec<String> = Vec::new();
    let mut explicit_input_requested = false;
    let mut file_input_used = false;

    let mut args = raw_args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            print_usage_and_exit(0);
        } else if arg == "--config" {
            if args.next().is_none() {
                eprintln!("Error: --config requires a path");
                std::process::exit(1);
            }
        } else if arg.starts_with("--config=") {
            continue;
        } else if arg == "--output" {
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
            explicit_input_requested = true;
        } else if arg == "--delay" {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("Error: --delay requires a number");
                std::process::exit(1);
            });
            delay_secs = parse_delay(&value);
        } else if arg == "--page-size" {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("Error: --page-size requires a value (letter|rm2)");
                std::process::exit(1);
            });
            page_size = parse_page_size(&value);
        } else if arg == "--summarize" {
            summarize = true;
        } else if arg == "--pattern" {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("Error: --pattern requires a name");
                std::process::exit(1);
            });
            pattern = value;
            summarize = true;
        } else if arg == "--feeds" {
            feeds_enabled = true;
            explicit_input_requested = true;
        } else if arg == "--yt-watchlist" {
            yt_watchlist_enabled = true;
            explicit_input_requested = true;
        } else if arg == "--clear-state" {
            clear_state = true;
        } else if arg == "--limit" {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("Error: --limit requires a number");
                std::process::exit(1);
            });
            let parsed = parse_limit(&value);
            feeds_limit = parsed;
            yt_limit = parsed;
        } else if arg == "--yt-limit" {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("Error: --yt-limit requires a number");
                std::process::exit(1);
            });
            yt_limit = parse_limit(&value);
        } else if arg == "--yt-pattern" {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("Error: --yt-pattern requires a name");
                std::process::exit(1);
            });
            yt_pattern = value;
        } else if arg == "--cookies-from-browser" {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("Error: --cookies-from-browser requires a browser name");
                std::process::exit(1);
            });
            yt_cookies_browser = value;
        } else if arg == "--no-mark-watched" {
            yt_mark_watched_on_success = false;
        } else if arg == "--feeds-file" {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("Error: --feeds-file requires a path");
                std::process::exit(1);
            });
            opml_path = Some(value);
        } else if let Some(value) = arg.strip_prefix("--output=") {
            output_path = Some(value.to_string());
        } else if let Some(value) = arg.strip_prefix("--file=") {
            input_file = Some(value.to_string());
            explicit_input_requested = true;
        } else if let Some(value) = arg.strip_prefix("--delay=") {
            delay_secs = parse_delay(value);
        } else if let Some(value) = arg.strip_prefix("--page-size=") {
            page_size = parse_page_size(value);
        } else if let Some(value) = arg.strip_prefix("--pattern=") {
            pattern = value.to_string();
            summarize = true;
        } else if let Some(value) = arg.strip_prefix("--limit=") {
            let parsed = parse_limit(value);
            feeds_limit = parsed;
            yt_limit = parsed;
        } else if let Some(value) = arg.strip_prefix("--yt-limit=") {
            yt_limit = parse_limit(value);
        } else if let Some(value) = arg.strip_prefix("--yt-pattern=") {
            yt_pattern = value.to_string();
        } else if let Some(value) = arg.strip_prefix("--cookies-from-browser=") {
            yt_cookies_browser = value.to_string();
        } else if let Some(value) = arg.strip_prefix("--feeds-file=") {
            opml_path = Some(value.to_string());
        } else if arg.starts_with('-') {
            eprintln!("Error: unexpected argument: {}", arg);
            print_usage_and_exit(1);
        } else {
            direct_urls.push(arg);
            explicit_input_requested = true;
        }
    }

    if clear_state && !explicit_input_requested {
        match state::init_state_db(true, state_db_path.take()) {
            Ok(_) => {
                println!("Cleared state DB");
                return;
            }
            Err(e) => {
                eprintln!("Error: failed to clear state DB: {}", e);
                std::process::exit(1);
            }
        }
    }

    let mut url_candidates: Vec<UrlCandidate> = direct_urls
        .iter()
        .map(|url| UrlCandidate {
            url: url.clone(),
            source: "arg",
            use_seen_state: false,
            toc_section: None,
        })
        .collect();

    if let Some(path) = input_file {
        file_input_used = true;
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
            url_candidates.push(UrlCandidate {
                url: trimmed.to_string(),
                source: "file",
                use_seen_state: false,
                toc_section: None,
            });
        }
    }

    let using_source_workflows = feeds_enabled || yt_watchlist_enabled;

    if url_candidates.is_empty() && !using_source_workflows {
        print_usage_and_exit(1);
    }

    if !using_source_workflows && direct_urls.len() == 1 && !file_input_used {
        let output_path = output_path.unwrap_or_else(|| {
            let prefix = if summarize {
                "single-summary"
            } else {
                "single"
            };
            render_output_path(prefix, output_dir.take())
        });

        eprintln!(
            "Mode: {}",
            if summarize {
                "single-summary"
            } else {
                "single"
            }
        );
        if summarize {
            eprintln!("Pattern: {}", pattern);
        }
        eprintln!("Page size: {}", page_size.as_str());

        let url = &direct_urls[0];
        match process_url_to_pdf_with_options(url, &output_path, summarize, &pattern, page_size) {
            Ok(_) => println!("Wrote {}", output_path),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    if summarize && yt_watchlist_enabled {
        eprintln!(
            "Note: --summarize does not apply to yt-watchlist items (already summary-driven)."
        );
    }

    let output_path = output_path.unwrap_or_else(|| {
        let prefix = if summarize || yt_watchlist_enabled {
            "bundle-summary"
        } else {
            "bundle"
        };
        render_output_path(prefix, output_dir.take())
    });

    let mut articles: Vec<multipdf::BundleArticle> = Vec::new();
    let mut attempted = 0usize;
    let mut included = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    let mut state = if clear_state || feeds_enabled || yt_watchlist_enabled {
        match state::init_state_db(clear_state, state_db_path.take()) {
            Ok(db) => Some(db),
            Err(e) => {
                eprintln!("Error: failed to initialize state DB: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    let mut seen_urls_in_run = HashSet::new();

    if feeds_enabled {
        let opml_path =
            opml_path.unwrap_or_else(|| default_feeds_opml_path().to_string_lossy().to_string());

        let feed_sources = match feeds::load_opml_feed_sources(&opml_path) {
            Ok(urls) => urls,
            Err(e) => {
                eprintln!("Error: failed to parse OPML {}: {}", opml_path, e);
                std::process::exit(1);
            }
        };

        if feed_sources.is_empty() {
            eprintln!("Warning: no feed URLs found in {}", opml_path);
        } else {
            let client = Client::builder()
                .user_agent("rmfeeder/0.1 (+https://example.com)")
                .build()
                .unwrap_or_else(|e| {
                    eprintln!("Error: failed to build HTTP client: {}", e);
                    std::process::exit(1);
                });

            for feed_source in feed_sources {
                match feeds::fetch_feed_links(&client, &feed_source.feed_url, feeds_limit) {
                    Ok(links) => {
                        for link in links {
                            url_candidates.push(UrlCandidate {
                                url: link,
                                source: "feeds",
                                use_seen_state: true,
                                toc_section: feed_source.section.clone(),
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: {}: {}", feed_source.feed_url, e);
                    }
                }
            }
        }
    }

    for candidate in url_candidates {
        if !seen_urls_in_run.insert(candidate.url.clone()) {
            continue;
        }

        attempted += 1;

        if candidate.use_seen_state {
            if let Some(db) = state.as_mut() {
                match db.should_emit(&candidate.url) {
                    Ok(false) => {
                        skipped += 1;
                        eprintln!(
                            "already seen, skipping item: {} [source={}]",
                            candidate.url, candidate.source
                        );
                        continue;
                    }
                    Ok(true) => {}
                    Err(e) => {
                        eprintln!("Warning: state check failed for {}: {}", candidate.url, e);
                    }
                }
            }
        }

        eprintln!("Fetching {}", candidate.url);
        let normalized = match fetcher::normalize_url(&candidate.url) {
            Ok(value) => value,
            Err(e) => {
                failed += 1;
                eprintln!("Skipping {}: invalid URL: {}", candidate.url, e);
                continue;
            }
        };

        let html = match fetcher::fetch_html(&normalized) {
            Ok(body) => body,
            Err(e) => {
                failed += 1;
                if let Some(status) = e.status() {
                    if status == StatusCode::FORBIDDEN {
                        eprintln!("Skipping {}: got 403 Forbidden", candidate.url);
                    } else {
                        eprintln!("Skipping {}: HTTP {}", candidate.url, status);
                    }
                } else {
                    eprintln!("Skipping {}: request error: {}", candidate.url, e);
                }
                continue;
            }
        };

        let article = match extractor::extract_article(&html, Some(&normalized)) {
            Some(value) => value,
            None => {
                failed += 1;
                eprintln!("Skipping {}: extraction failed", candidate.url);
                continue;
            }
        };

        let title = article.title;
        let content_html = if summarize {
            match summarize_html(&article.content.to_string(), &normalized, &pattern) {
                Ok(value) => value,
                Err(e) => {
                    failed += 1;
                    eprintln!("Skipping {}: summary failed: {}", candidate.url, e);
                    continue;
                }
            }
        } else {
            article.content.to_string()
        };

        articles.push(multipdf::BundleArticle {
            section: candidate.toc_section.clone(),
            title,
            content_html,
        });
        included += 1;

        if candidate.use_seen_state {
            if let Some(db) = state.as_mut() {
                if let Err(e) = db.mark_seen(&candidate.url) {
                    eprintln!(
                        "Warning: failed to update state for {}: {}",
                        candidate.url, e
                    );
                }
            }
        }

        if delay_secs > 0 {
            thread::sleep(Duration::from_secs(delay_secs));
        }
    }

    if yt_watchlist_enabled {
        eprintln!("Fetching Watch Later list...");
        let videos = match youtube::fetch_watch_later(&yt_cookies_browser) {
            Ok(videos) => videos,
            Err(e) => {
                eprintln!("Error: failed to fetch watch list: {}", e);
                std::process::exit(1);
            }
        };

        let mut yt_included = 0usize;
        for video in videos {
            if yt_included >= yt_limit {
                break;
            }

            attempted += 1;
            let state_key = format!("yt::{}", video.url);

            if let Some(db) = state.as_mut() {
                match db.should_emit(&state_key) {
                    Ok(false) => {
                        skipped += 1;
                        eprintln!(
                            "already seen, skipping item: {} [source=yt-watchlist]",
                            video.url
                        );
                        continue;
                    }
                    Ok(true) => {}
                    Err(e) => {
                        eprintln!("Warning: state check failed for {}: {}", video.url, e);
                    }
                }
            }

            eprintln!("Processing {}", video.url);
            let body_html = match youtube::summarize_watch_video(&video.url, &yt_pattern) {
                Ok(value) => value,
                Err(e) => {
                    failed += 1;
                    eprintln!("Skipping {}: summary failed: {}", video.url, e);
                    continue;
                }
            };

            articles.push(multipdf::BundleArticle {
                section: Some("YouTube Watchlist".to_string()),
                title: video.title,
                content_html: body_html,
            });
            included += 1;
            yt_included += 1;

            if let Some(db) = state.as_mut() {
                if let Err(e) = db.mark_seen(&state_key) {
                    eprintln!("Warning: failed to update state for {}: {}", video.url, e);
                }
            }

            if yt_mark_watched_on_success {
                if let Err(e) = youtube::mark_watched(&yt_cookies_browser, &video.url) {
                    eprintln!("Warning: failed to mark watched {}: {}", video.url, e);
                }
            }

            if yt_delay > 0 {
                thread::sleep(Duration::from_secs(yt_delay));
            }
        }
    }

    if articles.is_empty() {
        eprintln!("Error: no items were included in output");
        eprintln!(
            "Summary: attempted={} included={} skipped={} failed={}",
            attempted, included, skipped, failed
        );
        std::process::exit(1);
    }

    eprintln!("Mode: unified-bundle");
    if summarize {
        eprintln!("Pattern: {}", pattern);
    }
    if yt_watchlist_enabled {
        eprintln!("YouTube pattern: {}", yt_pattern);
    }
    eprintln!("Page size: {}", page_size.as_str());

    match multipdf::generate_pdf_bundle_with_sections(
        &articles,
        &output_path,
        "rmfeeder ::<br>Reading Bundle",
        "Collected Articles",
        page_size,
    ) {
        Ok(_) => {
            eprintln!(
                "Summary: attempted={} included={} skipped={} failed={}",
                attempted, included, skipped, failed
            );
            println!("Wrote {}", output_path);
        }
        Err(e) => {
            eprintln!("Error: failed to generate PDF: {}", e);
            std::process::exit(1);
        }
    }
}

fn render_output_path(prefix: &str, output_dir: Option<String>) -> String {
    let filename = format!(
        "{}-{}.pdf",
        prefix,
        Local::now().format("%Y-%m-%d-%H-%M-%S")
    );
    if let Some(dir) = output_dir {
        Path::new(&dir).join(filename).to_string_lossy().to_string()
    } else {
        filename
    }
}

fn parse_delay(value: &str) -> u64 {
    value.parse::<u64>().unwrap_or_else(|_| {
        eprintln!("Error: --delay must be a non-negative number");
        std::process::exit(1);
    })
}

fn parse_limit(value: &str) -> usize {
    value.parse::<usize>().unwrap_or_else(|_| {
        eprintln!("Error: --limit must be a positive number");
        std::process::exit(1);
    })
}

fn parse_page_size(value: &str) -> PageSize {
    PageSize::parse(value).unwrap_or_else(|| {
        eprintln!("Error: --page-size must be one of: letter, rm2");
        std::process::exit(1);
    })
}

fn print_usage_and_exit(code: i32) -> ! {
    eprintln!(
        "Usage: rmfeeder [--config <path>] [--output <file.pdf>] [--file <path>] [--delay N] [--page-size <letter|rm2>] [--summarize] [--pattern <name>] [--feeds] [--feeds-file <feeds.opml>] [--yt-watchlist] [--yt-limit N] [--yt-pattern <name>] [--cookies-from-browser <name>] [--no-mark-watched] [--clear-state] [--limit N] <url1> [url2] ..."
    );
    std::process::exit(code);
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
