pub mod extractor;
pub mod xhtml;
pub mod fetcher;

/// High-level pipeline:
///
/// Raw URL (possibly with parentheses or unsafe chars)
/// → normalize_url()
/// → fetch_html()
/// → extract_article()
/// → wrap()
///
/// This is the top-level API used by the CLI.
pub fn process_url(url: &str) -> String {
    //
    // 1. Normalize the URL (escape parentheses, spaces, unicode, etc.)
    //
    let normalized = match fetcher::normalize_url(url) {
        Ok(u) => u,
        Err(e) => {
            return xhtml::wrap(
                "rmfeeder – invalid URL",
                &format!("Invalid URL '{}': {}", url, e),
            )
        }
    };

    //
    // 2. Fetch the HTML from the normalized URL
    //
    let html = match fetcher::fetch_html(&normalized) {
        Ok(body) => body,
        Err(e) => {
            return xhtml::wrap(
                "rmfeeder – fetch error",
                &format!("Error fetching '{}': {}", normalized, e),
            )
        }
    };

    //
    // 3. Run Readability extraction (dom_smoothie)
    //
    match extractor::extract_article(&html, Some(&normalized)) {
        Some(article) => {
            let title = article.title;
            let content_html = article.content.to_string();
            xhtml::wrap(&title, &content_html)
        }

        //
        // 4. Fallback: show raw HTML if extraction fails
        //
        None => xhtml::wrap(
            "rmfeeder – extraction failure",
            &format!(
                "<p>Failed to extract readable article from:</p><pre>{}</pre>",
                normalized
            ),
        ),
    }
}