use std::collections::BTreeSet;

use feed_rs::parser;
use reqwest::blocking::Client;
use roxmltree::Document;

pub fn load_opml_feed_urls(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let doc = Document::parse(&content)?;
    let mut urls = BTreeSet::new();

    for node in doc.descendants().filter(|n| n.has_tag_name("outline")) {
        if let Some(url) = node.attribute("xmlUrl") {
            urls.insert(url.to_string());
        }
    }

    Ok(urls.into_iter().collect())
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
