use std::process::{Command, Stdio};

use pulldown_cmark::{Options, Parser, html};
use serde::Deserialize;

use crate::escape_html;

const WATCH_LATER_URL: &str = "https://www.youtube.com/playlist?list=WL";

#[derive(Debug, Clone)]
pub struct YtVideo {
    pub title: String,
    pub url: String,
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

pub fn fetch_watch_later(
    cookies_browser: &str,
) -> Result<Vec<YtVideo>, Box<dyn std::error::Error>> {
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

pub fn summarize_watch_video(
    url: &str,
    pattern: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let summary_markdown = run_fabric_youtube(url, pattern)?;
    let summary_html = markdown_to_html(&summary_markdown);
    let safe_url = escape_html(url);
    Ok(format!(
        "<p class=\"article-source\">Source: <a href=\"{url}\">{url}</a></p>\n{body}",
        url = safe_url,
        body = summary_html
    ))
}

pub fn mark_watched(cookies_browser: &str, url: &str) -> Result<(), Box<dyn std::error::Error>> {
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
