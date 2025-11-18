use dom_smoothie::{Article, Config, Readability};

pub fn extract_article(html: &str, url: Option<&str>) -> Option<Article> {
    let cfg = Config::default();
    let mut rdr = Readability::new(html, url, Some(cfg)).ok()?;
    rdr.parse().ok()
}