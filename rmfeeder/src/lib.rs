pub mod extractor;
pub mod fetcher;
pub mod epub;
pub mod pdf;
pub mod xhtml;      // ← ADD THIS
pub mod multipdf;   // ← ALSO ADD THIS

use serde::Deserialize;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize)]
pub struct AppConfig {
    pub state_db_path: Option<String>,
    pub feeds_opml_path: Option<String>,
    pub urls_path: Option<String>,
    pub output_dir: Option<String>,
    pub limit: Option<usize>,
    pub delay: Option<u64>,
    pub summarize: Option<bool>,
    pub pattern: Option<String>,
    pub yt_limit: Option<usize>,
    pub yt_pattern: Option<String>,
    pub yt_delay: Option<u64>,
    pub yt_cookies_browser: Option<String>,
    pub yt_mark_watched_on_success: Option<bool>,
}

pub fn load_config() -> Result<Option<AppConfig>, Box<dyn std::error::Error>> {
    load_config_from_path("rmfeeder.toml")
}

pub fn load_config_from_path(path: &str) -> Result<Option<AppConfig>, Box<dyn std::error::Error>> {
    let path = expand_tilde_path(path);
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            let cfg: AppConfig = toml::from_str(&contents)?;
            Ok(Some(cfg))
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn expand_tilde_path(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return Path::new(&home).join(rest);
        }
    }
    PathBuf::from(path)
}

/// HTML preview (if you still want it)
pub fn process_url(url: &str) -> String {
    let normalized = match fetcher::normalize_url(url) {
        Ok(u) => u,
        Err(e) => return format!("Invalid URL '{}': {}", url, e),
    };

    let html = match fetcher::fetch_html(&normalized) {
        Ok(body) => body,
        Err(e) => return format!("Fetch error '{}': {}", normalized, e),
    };

    match extractor::extract_article(&html, Some(&normalized)) {
        Some(article) => article.content.to_string(),
        None => format!("Readability failed for {}", normalized),
    }
}

pub fn process_url_to_pdf(
    url: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    process_url_to_pdf_with_options(url, output_path, false, "summarize")
}

pub fn process_url_to_pdf_with_options(
    url: &str,
    output_path: &str,
    summarize: bool,
    pattern: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let normalized = fetcher::normalize_url(url)?;
    let html = fetcher::fetch_html(&normalized)?;

    if let Some(article) = extractor::extract_article(&html, Some(&normalized)) {
        let body_html = if summarize {
            summarize_html(&article.content.to_string(), &normalized, pattern)?
        } else {
            article.content.to_string()
        };
        pdf::generate_pdf(&article.title, &body_html, output_path)
    } else {
        Err("Readability extraction failed".into())
    }
}

pub fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

pub fn temp_html_path(prefix: &str) -> std::path::PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};

    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    let filename = format!("{prefix}_{pid}_{since_epoch}.html");
    std::env::temp_dir().join(filename)
}

pub fn summarize_html(
    content_html: &str,
    source_url: &str,
    pattern: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let summary = run_fabric(pattern, content_html)?;
    let summary_html = markdown_to_html(&summary);
    let safe_url = escape_html(source_url);
    let source_html = format!(
        "<p class=\"article-source\">Source: <a href=\"{url}\">{url}</a></p>",
        url = safe_url
    );
    Ok(format!("{}\n{}", source_html, summary_html))
}

fn markdown_to_html(input: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(input, options);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

fn run_fabric(pattern: &str, input: &str) -> Result<String, Box<dyn std::error::Error>> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("fabric-ai")
        .arg("-p")
        .arg(pattern)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("fabric failed: {}", stderr.trim()).into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
