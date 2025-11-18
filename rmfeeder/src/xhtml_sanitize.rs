use kuchiki::traits::*;
use kuchiki::parse_html;

/// Remove nodes entirely
fn remove_by_selector(document: &kuchiki::NodeRef, selector: &str) {
    if let Ok(nodes) = document.select(selector) {
        for node in nodes {
            node.as_node().detach();
        }
    }
}

/// Remove the tag but keep its children
fn unwrap_selector(document: &kuchiki::NodeRef, selector: &str) {
    if let Ok(nodes) = document.select(selector) {
        for matched in nodes {
            let node = matched.as_node();

            // Move children before this node
            for child in node.children() {
                node.insert_before(child);
            }

            node.detach();
        }
    }
}

pub fn sanitize_for_xhtml(html: &str) -> String {
    let document = parse_html().one(html);

    // 🚫 Remove problematic elements
    for selector in &[
        "img",
        "picture",
        "figure",
        "wbr",
        "script",
        "style",
        "meta",
        "link",
    ] {
        remove_by_selector(&document, selector);
    }

    // 🚫 REMOVE ALL LINKS — THIS IS THE CRITICAL FIX
    unwrap_selector(&document, "a");

    // 🚫 Unwrap HTML5 structural tags (flatten them)
    for selector in &["main", "header", "footer", "section", "article", "nav"] {
        unwrap_selector(&document, selector);
    }

    // Serialize DOM back to XHTML-ish HTML
    let mut out = Vec::new();
    document.serialize(&mut out).unwrap();

    String::from_utf8(out).unwrap()
}