use std::collections::HashSet;

use feed_rs::parser;
use reqwest::blocking::Client;
use roxmltree::{Document, Node};

#[derive(Debug, Clone)]
pub struct FeedSource {
    pub feed_url: String,
    pub section: Option<String>,
}

pub fn load_opml_feed_urls(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    Ok(load_opml_feed_sources(path)?
        .into_iter()
        .map(|source| source.feed_url)
        .collect())
}

pub fn load_opml_feed_sources(path: &str) -> Result<Vec<FeedSource>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let doc = Document::parse(&content)?;

    let mut seen_feed_urls = HashSet::new();
    let mut sources = Vec::new();

    let root = doc.root_element();
    collect_sources(root, None, &mut seen_feed_urls, &mut sources);

    Ok(sources)
}

fn collect_sources(
    node: Node<'_, '_>,
    current_section: Option<&str>,
    seen_feed_urls: &mut HashSet<String>,
    sources: &mut Vec<FeedSource>,
) {
    if !node.has_tag_name("outline") {
        for child in node.children().filter(|child| child.is_element()) {
            collect_sources(child, current_section, seen_feed_urls, sources);
        }
        return;
    }

    let section_label = node
        .attribute("title")
        .or_else(|| node.attribute("text"))
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if let Some(feed_url) = node.attribute("xmlUrl").map(str::trim) {
        if !feed_url.is_empty() && seen_feed_urls.insert(feed_url.to_string()) {
            sources.push(FeedSource {
                feed_url: feed_url.to_string(),
                section: current_section.map(ToString::to_string),
            });
        }
    }

    let next_section = if node.attribute("xmlUrl").is_some() {
        current_section
    } else {
        section_label.or(current_section)
    };

    for child in node.children().filter(|child| child.is_element()) {
        collect_sources(child, next_section, seen_feed_urls, sources);
    }
}

pub fn fetch_feed_links(
    client: &Client,
    feed_url: &str,
    limit: usize,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let res = client.get(feed_url).send()?.error_for_status()?;
    let bytes = res.bytes()?;
    let feed = parser::parse(bytes.as_ref())?;

    let mut entries = feed.entries;
    entries.sort_by(|a, b| entry_timestamp(b).cmp(&entry_timestamp(a)));

    let mut out = Vec::new();
    for entry in entries.into_iter().take(limit) {
        if let Some(link) = pick_entry_link(&entry) {
            out.push(link);
        }
    }

    Ok(out)
}

fn entry_timestamp(entry: &feed_rs::model::Entry) -> i64 {
    entry
        .published
        .or(entry.updated)
        .map(|d| d.timestamp())
        .unwrap_or(0)
}

fn pick_entry_link(entry: &feed_rs::model::Entry) -> Option<String> {
    if let Some(link) = entry
        .links
        .iter()
        .find(|l| l.rel.as_deref() == Some("alternate"))
    {
        return Some(link.href.clone());
    }
    entry.links.first().map(|l| l.href.clone())
}

#[cfg(test)]
mod tests {
    use super::load_opml_feed_sources;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("rmfeeder-{name}-{nanos}.opml"))
    }

    #[test]
    fn parses_sections_and_dedupes_feed_urls() {
        let opml = r#"<?xml version="1.0" encoding="UTF-8"?>
<opml version="2.0">
  <body>
    <outline text="Tech">
      <outline text="Feed A" xmlUrl="https://example.com/a.xml"/>
      <outline text="Nested">
        <outline text="Feed B" xmlUrl="https://example.com/b.xml"/>
      </outline>
    </outline>
    <outline text="Duplicate" xmlUrl="https://example.com/a.xml"/>
    <outline text="No Section Feed" xmlUrl="https://example.com/c.xml"/>
  </body>
</opml>"#;

        let path = temp_path("sources");
        std::fs::write(&path, opml).expect("write OPML fixture");

        let sources = load_opml_feed_sources(path.to_str().expect("utf8 path"))
            .expect("parse OPML sources");

        std::fs::remove_file(&path).ok();

        assert_eq!(sources.len(), 3);
        assert_eq!(sources[0].feed_url, "https://example.com/a.xml");
        assert_eq!(sources[0].section.as_deref(), Some("Tech"));
        assert_eq!(sources[1].feed_url, "https://example.com/b.xml");
        assert_eq!(sources[1].section.as_deref(), Some("Nested"));
        assert_eq!(sources[2].feed_url, "https://example.com/c.xml");
        assert_eq!(sources[2].section, None);
    }
}
