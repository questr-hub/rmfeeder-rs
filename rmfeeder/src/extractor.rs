use dom_smoothie::{Article, Config, Readability};

/// Run Readability (via dom_smoothie) on the given HTML.
/// `url` is optional but recommended for resolving relative links.
pub fn extract_article(html: &str, url: Option<&str>) -> Option<Article> {
    // Reasonable default; tune later if needed.
    let cfg = Config {
        max_elements_to_parse: 9000,
        ..Default::default()
    };

    let mut readability = Readability::new(html, url, Some(cfg)).ok()?;
    let article = readability.parse().ok()?;
    Some(article)
}