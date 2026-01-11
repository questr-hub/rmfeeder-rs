pub mod extractor;
pub mod fetcher;
pub mod epub;
pub mod pdf;
pub mod xhtml;      // ← ADD THIS
pub mod multipdf;   // ← ALSO ADD THIS

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
    let normalized = fetcher::normalize_url(url)?;
    let html = fetcher::fetch_html(&normalized)?;

    if let Some(article) = extractor::extract_article(&html, Some(&normalized)) {
        pdf::generate_pdf(&article.title, &article.content.to_string(), output_path)
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
