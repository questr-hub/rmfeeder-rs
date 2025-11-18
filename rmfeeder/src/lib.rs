pub mod extractor;
pub mod fetcher;
pub mod pdf;

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