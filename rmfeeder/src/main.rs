use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use chrono::Local;
use clap::{CommandFactory, Parser};
use reqwest::StatusCode;
use reqwest::blocking::Client;
use rmfeeder::multipdf;
use rmfeeder::{
    PageSize, default_config_path, default_feeds_opml_path, extractor, feeds, fetcher,
    list_targets_csv, load_config_from_path, markdown, process_url_to_pdf_with_options, state,
    summarize_content_html, summarize_html, youtube,
};

const HELP_USAGE: &str = "\
rmfeeder [OPTIONS] <url1> [url2 ...]
       rmfeeder [OPTIONS] --file <path>
       rmfeeder [OPTIONS] --feeds [--feeds-file <feeds.opml>]
       rmfeeder [OPTIONS] --yt-watchlist
       rmfeeder [OPTIONS] --markdown <path>
       rmfeeder [OPTIONS] --markdown-dir <path>
       rmfeeder [OPTIONS] --stdin
       rmfeeder --clear-state
       rmfeeder --list-targets";

const HELP_AFTER: &str = "\
Examples:
  rmfeeder --feeds --limit 5
  rmfeeder --yt-watchlist --yt-limit 20 --page-size rmpp
  rmfeeder --markdown notes.md --summarize --pattern extract_wisdom
  cat notes.md | rmfeeder --stdin --output note.pdf

Defaults:
  --config defaults to ~/.config/rmfeeder/rmfeeder.toml
  --feeds uses ~/.config/rmfeeder/feeds.opml (or XDG_CONFIG_HOME)";

#[derive(Debug, Parser)]
#[command(
    name = "rmfeeder",
    about = "Build article/notes bundles as device-friendly PDFs",
    long_about = None,
    disable_help_subcommand = true,
    next_line_help = true,
    override_usage = HELP_USAGE,
    after_help = HELP_AFTER
)]
struct CliArgs {
    #[arg(
        long,
        value_name = "path",
        help_heading = "Maintenance",
        help = "Config file path (default: ~/.config/rmfeeder/rmfeeder.toml)"
    )]
    config: Option<String>,

    #[arg(
        long,
        value_name = "file.pdf",
        help_heading = "Output & Rendering",
        help = "Output PDF path (overrides timestamp naming)"
    )]
    output: Option<String>,

    #[arg(
        long,
        value_name = "path",
        help_heading = "Source Input (choose exactly one)",
        help = "Read URLs from a file (blank lines and # comments ignored)"
    )]
    file: Option<String>,

    #[arg(
        long,
        help_heading = "Source Input (choose exactly one)",
        help = "Use OPML feeds from default or configured path"
    )]
    feeds: bool,

    #[arg(
        long,
        value_name = "feeds.opml",
        help_heading = "Source Input (choose exactly one)",
        help = "Use an explicit OPML file (implies --feeds)"
    )]
    feeds_file: Option<String>,

    #[arg(
        long,
        help_heading = "Source Input (choose exactly one)",
        help = "Pull from YouTube Watch Later"
    )]
    yt_watchlist: bool,

    #[arg(
        long,
        value_name = "path",
        help_heading = "Source Input (choose exactly one)",
        help = "Convert one markdown file to a single PDF entry"
    )]
    markdown: Option<String>,

    #[arg(
        long,
        value_name = "path",
        help_heading = "Source Input (choose exactly one)",
        help = "Convert a directory of markdown files into one bundle"
    )]
    markdown_dir: Option<String>,

    #[arg(
        long,
        help_heading = "Source Input (choose exactly one)",
        help = "Read markdown content from stdin"
    )]
    stdin: bool,

    #[arg(
        value_name = "url",
        help_heading = "Source Input (choose exactly one)",
        help = "One or more direct URLs"
    )]
    urls: Vec<String>,

    #[arg(
        long,
        value_name = "seconds",
        help_heading = "Output & Rendering",
        help = "Delay between fetches"
    )]
    delay: Option<u64>,

    #[arg(
        long,
        value_name = "name",
        help_heading = "Output & Rendering",
        help = "Target device/page profile (run --list-targets)"
    )]
    page_size: Option<String>,

    #[arg(
        long,
        help_heading = "Summarization",
        help = "Use fabric to summarize content before rendering"
    )]
    summarize: bool,

    #[arg(
        long,
        value_name = "name",
        help_heading = "Summarization",
        help = "fabric pattern to use (implies --summarize)"
    )]
    pattern: Option<String>,

    #[arg(
        long,
        value_name = "N",
        help_heading = "YouTube Options",
        help = "Limit only YouTube items"
    )]
    yt_limit: Option<usize>,

    #[arg(
        long,
        value_name = "name",
        help_heading = "YouTube Options",
        help = "YouTube summary pattern (default: youtube_summary)"
    )]
    yt_pattern: Option<String>,

    #[arg(
        long,
        value_name = "name",
        help_heading = "YouTube Options",
        help = "Browser/profile for auth cookies (default: chrome)"
    )]
    cookies_from_browser: Option<String>,

    #[arg(
        long,
        help_heading = "YouTube Options",
        help = "Do not mark processed videos as watched"
    )]
    no_mark_watched: bool,

    #[arg(
        long,
        value_name = "N",
        help_heading = "Output & Rendering",
        help = "Shared item limit (feeds, yt, markdown-dir)"
    )]
    limit: Option<usize>,

    #[arg(
        long,
        help_heading = "Maintenance",
        help = "Clear seen-item state DB and exit (when no source is selected)"
    )]
    clear_state: bool,

    #[arg(
        long,
        help_heading = "Maintenance",
        help = "Print page-size target table as CSV and exit"
    )]
    list_targets: bool,
}

struct UrlCandidate {
    url: String,
    source: &'static str,
    use_seen_state: bool,
    toc_section: Option<String>,
}

#[derive(Clone, Copy)]
struct SourceSelection {
    label: &'static str,
    kind: SourceKind,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum SourceKind {
    UrlArgs,
    UrlFile,
    Feeds,
    YtWatchlist,
    MarkdownFile,
    MarkdownDir,
    Stdin,
}

fn main() {
    let cli = CliArgs::parse();
    if cli.list_targets {
        print!("{}", list_targets_csv());
        return;
    }

    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(|| default_config_path().to_string_lossy().to_string());

    let config = match load_config_from_path(&config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error: failed to load config {}: {}", config_path, e);
            std::process::exit(1);
        }
    };

    let input_file: Option<String> = cli.file.clone();
    let output_path: Option<String> = cli.output.clone();
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

    let mut feeds_enabled = cli.feeds;
    let yt_watchlist_enabled = cli.yt_watchlist;
    let clear_state = cli.clear_state;

    let mut feeds_limit: usize = config.as_ref().and_then(|c| c.limit).unwrap_or(3);
    let mut markdown_limit: Option<usize> = config.as_ref().and_then(|c| c.limit);
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

    let direct_urls: Vec<String> = cli.urls.clone();
    let markdown_file: Option<String> = cli.markdown.clone();
    let markdown_dir: Option<String> = cli.markdown_dir.clone();
    let stdin_enabled = cli.stdin;
    let mut feeds_file_flag_used = false;
    if let Some(value) = cli.delay {
        delay_secs = value;
    }
    if let Some(value) = cli.page_size {
        page_size = parse_page_size(&value);
    }
    if cli.summarize {
        summarize = true;
    }
    if let Some(value) = cli.pattern {
        pattern = value;
        summarize = true;
    }
    if let Some(value) = cli.limit {
        feeds_limit = value;
        yt_limit = value;
        markdown_limit = Some(value);
    }
    if let Some(value) = cli.yt_limit {
        yt_limit = value;
    }
    if let Some(value) = cli.yt_pattern {
        yt_pattern = value;
    }
    if let Some(value) = cli.cookies_from_browser {
        yt_cookies_browser = value;
    }
    if cli.no_mark_watched {
        yt_mark_watched_on_success = false;
    }
    if let Some(value) = cli.feeds_file {
        opml_path = Some(value);
        feeds_enabled = true;
        feeds_file_flag_used = true;
    }

    let explicit_input_requested = !direct_urls.is_empty()
        || input_file.is_some()
        || feeds_enabled
        || yt_watchlist_enabled
        || markdown_file.is_some()
        || markdown_dir.is_some()
        || stdin_enabled;

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

    let mut selected_sources: Vec<SourceSelection> = Vec::new();
    if !direct_urls.is_empty() {
        selected_sources.push(SourceSelection {
            label: "URL args",
            kind: SourceKind::UrlArgs,
        });
    }
    if input_file.is_some() {
        selected_sources.push(SourceSelection {
            label: "--file",
            kind: SourceKind::UrlFile,
        });
    }
    if feeds_enabled {
        selected_sources.push(SourceSelection {
            label: if feeds_file_flag_used {
                "--feeds-file"
            } else {
                "--feeds"
            },
            kind: SourceKind::Feeds,
        });
    }
    if yt_watchlist_enabled {
        selected_sources.push(SourceSelection {
            label: "--yt-watchlist",
            kind: SourceKind::YtWatchlist,
        });
    }
    if markdown_file.is_some() {
        selected_sources.push(SourceSelection {
            label: "--markdown",
            kind: SourceKind::MarkdownFile,
        });
    }
    if markdown_dir.is_some() {
        selected_sources.push(SourceSelection {
            label: "--markdown-dir",
            kind: SourceKind::MarkdownDir,
        });
    }
    if stdin_enabled {
        selected_sources.push(SourceSelection {
            label: "--stdin",
            kind: SourceKind::Stdin,
        });
    }

    if selected_sources.len() > 1 {
        let first = selected_sources[0].label;
        let second = selected_sources[1].label;
        eprintln!(
            "error: conflicting source flags: {} and {} cannot be used together",
            first, second
        );
        std::process::exit(1);
    }

    let selected_source_kind = selected_sources.first().map(|s| s.kind);

    if let Some(path) = markdown_file {
        let output_path = output_path.unwrap_or_else(|| {
            let prefix = if summarize {
                "single-summary"
            } else {
                "single"
            };
            render_output_path(prefix, output_dir.take())
        });
        run_markdown_file_mode(&path, &output_path, summarize, &pattern, page_size);
        return;
    }

    if let Some(path) = markdown_dir {
        let output_path = output_path.unwrap_or_else(|| {
            let prefix = if summarize {
                "bundle-summary"
            } else {
                "bundle"
            };
            render_output_path(prefix, output_dir.take())
        });
        run_markdown_dir_mode(
            &path,
            &output_path,
            summarize,
            &pattern,
            markdown_limit,
            page_size,
        );
        return;
    }

    if stdin_enabled {
        let output_path = output_path.unwrap_or_else(|| {
            let prefix = if summarize {
                "single-summary"
            } else {
                "single"
            };
            render_output_path(prefix, output_dir.take())
        });
        run_stdin_mode(&output_path, summarize, &pattern, page_size);
        return;
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

    if selected_source_kind == Some(SourceKind::UrlArgs) && direct_urls.len() == 1 {
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

        if candidate.use_seen_state
            && let Some(db) = state.as_mut()
        {
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
            match summarize_html(article.content.as_ref(), &normalized, &pattern) {
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

        if candidate.use_seen_state
            && let Some(db) = state.as_mut()
            && let Err(e) = db.mark_seen(&candidate.url)
        {
            eprintln!(
                "Warning: failed to update state for {}: {}",
                candidate.url, e
            );
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

            if let Some(db) = state.as_mut()
                && let Err(e) = db.mark_seen(&state_key)
            {
                eprintln!("Warning: failed to update state for {}: {}", video.url, e);
            }

            if yt_mark_watched_on_success
                && let Err(e) = youtube::mark_watched(&yt_cookies_browser, &video.url)
            {
                eprintln!("Warning: failed to mark watched {}: {}", video.url, e);
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

fn run_markdown_file_mode(
    path: &str,
    output_path: &str,
    summarize: bool,
    pattern: &str,
    page_size: PageSize,
) {
    let path_buf = PathBuf::from(path);
    if !path_buf.is_file() {
        eprintln!("error: file not found: {}", path);
        std::process::exit(1);
    }

    let article =
        markdown_file_to_bundle_article(&path_buf, summarize, pattern).unwrap_or_else(|e| {
            eprintln!("error: failed to read {}: {}", path, e);
            std::process::exit(1);
        });

    let cover_subtitle = format!("Source: {} • Entries: 1", path_buf.to_string_lossy());
    let articles = vec![article];
    match multipdf::generate_pdf_bundle_with_render_options(
        &articles,
        output_path,
        &articles[0].title,
        &cover_subtitle,
        page_size,
        false,
        false,
    ) {
        Ok(_) => println!("Wrote {}", output_path),
        Err(e) => {
            eprintln!("Error: failed to generate PDF: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_markdown_dir_mode(
    path: &str,
    output_path: &str,
    summarize: bool,
    pattern: &str,
    limit: Option<usize>,
    page_size: PageSize,
) {
    let dir_path = PathBuf::from(path);
    if !dir_path.is_dir() {
        eprintln!("error: not a directory: {}", path);
        std::process::exit(1);
    }

    let mut markdown_files = list_markdown_files_flat(&dir_path).unwrap_or_else(|e| {
        eprintln!("error: failed to read directory {}: {}", path, e);
        std::process::exit(1);
    });
    markdown_files.sort_by_key(|p| {
        p.file_name()
            .map(|n| n.to_string_lossy().to_ascii_lowercase())
    });
    if let Some(limit) = limit {
        markdown_files.truncate(limit);
    }

    if markdown_files.is_empty() {
        eprintln!("error: no markdown files found in {}", path);
        std::process::exit(1);
    }

    let mut articles = Vec::with_capacity(markdown_files.len());
    for file_path in markdown_files {
        let article = markdown_file_to_bundle_article(&file_path, summarize, pattern)
            .unwrap_or_else(|e| {
                eprintln!(
                    "error: failed to read {}: {}",
                    file_path.to_string_lossy(),
                    e
                );
                std::process::exit(1);
            });
        articles.push(article);
    }

    let bundle_title = dir_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "markdown-bundle".to_string());
    let cover_subtitle = format!(
        "Source: {} • Entries: {}",
        dir_path.to_string_lossy(),
        articles.len()
    );
    match multipdf::generate_pdf_bundle_with_sections(
        &articles,
        output_path,
        &bundle_title,
        &cover_subtitle,
        page_size,
    ) {
        Ok(_) => println!("Wrote {}", output_path),
        Err(e) => {
            eprintln!("Error: failed to generate PDF: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_stdin_mode(output_path: &str, summarize: bool, pattern: &str, page_size: PageSize) {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .unwrap_or_else(|e| {
            eprintln!("error: failed to read stdin: {}", e);
            std::process::exit(1);
        });
    if input.trim().is_empty() {
        eprintln!("error: stdin produced no content");
        std::process::exit(1);
    }

    let article = markdown_content_to_bundle_article(&input, "stdin-bundle", summarize, pattern)
        .unwrap_or_else(|e| {
            eprintln!("error: failed to process stdin: {}", e);
            std::process::exit(1);
        });
    let cover_subtitle = "Source: <stdin> • Entries: 1";
    let articles = vec![article];
    match multipdf::generate_pdf_bundle_with_render_options(
        &articles,
        output_path,
        &articles[0].title,
        cover_subtitle,
        page_size,
        false,
        false,
    ) {
        Ok(_) => println!("Wrote {}", output_path),
        Err(e) => {
            eprintln!("Error: failed to generate PDF: {}", e);
            std::process::exit(1);
        }
    }
}

fn list_markdown_files_flat(dir_path: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(extension) = path.extension().and_then(|s| s.to_str()) else {
            continue;
        };
        if extension.eq_ignore_ascii_case("md") {
            out.push(path);
        }
    }
    Ok(out)
}

fn markdown_file_to_bundle_article(
    path: &Path,
    summarize: bool,
    pattern: &str,
) -> Result<multipdf::BundleArticle, Box<dyn std::error::Error>> {
    let raw_content = fs::read_to_string(path)?;
    let fallback_title = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "untitled".to_string());
    markdown_content_to_bundle_article(&raw_content, &fallback_title, summarize, pattern)
}

fn markdown_content_to_bundle_article(
    raw_markdown: &str,
    fallback_title: &str,
    summarize: bool,
    pattern: &str,
) -> Result<multipdf::BundleArticle, Box<dyn std::error::Error>> {
    let without_frontmatter = markdown::strip_yaml_frontmatter(raw_markdown);
    let title = markdown::extract_first_h1(&without_frontmatter)
        .unwrap_or_else(|| fallback_title.to_string());
    let body_markdown = markdown::strip_first_h1(&without_frontmatter);

    let rendered_html = markdown::markdown_to_html(&body_markdown);
    let content_html = if summarize {
        summarize_content_html(&rendered_html, pattern)?
    } else {
        rendered_html
    };

    Ok(multipdf::BundleArticle {
        section: None,
        title,
        content_html,
    })
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

fn parse_page_size(value: &str) -> PageSize {
    PageSize::parse(value).unwrap_or_else(|| {
        eprintln!(
            "Error: --page-size must be one of: {}",
            PageSize::VALUE_LIST
        );
        std::process::exit(1);
    })
}

fn print_usage_and_exit(code: i32) -> ! {
    let mut command = CliArgs::command();
    if code == 0 {
        let mut out = std::io::stdout();
        let _ = command.write_long_help(&mut out);
        let _ = writeln!(&mut out);
    } else {
        let mut err = std::io::stderr();
        let _ = command.write_long_help(&mut err);
        let _ = writeln!(&mut err);
    }
    std::process::exit(code);
}
