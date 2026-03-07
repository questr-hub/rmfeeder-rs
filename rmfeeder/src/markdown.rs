use pulldown_cmark::{Options, Parser, html};

pub fn strip_yaml_frontmatter(content: &str) -> String {
    if !content.starts_with("---\n") {
        return content.to_string();
    }

    let rest = &content[4..];
    if let Some(end_idx) = rest.find("---\n") {
        return rest[(end_idx + 4)..].to_string();
    }

    content.to_string()
}

pub fn extract_first_h1(markdown: &str) -> Option<String> {
    for line in markdown.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ") {
            let title = title.trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
        }
    }
    None
}

pub fn strip_first_h1(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut removed = false;

    for chunk in markdown.split_inclusive('\n') {
        let line = chunk.strip_suffix('\n').unwrap_or(chunk);
        if !removed && line.trim_start().starts_with("# ") {
            removed = true;
            continue;
        }
        out.push_str(chunk);
    }

    if removed { out } else { markdown.to_string() }
}

pub fn markdown_to_html(input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(input, options);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

#[cfg(test)]
mod tests {
    use super::{extract_first_h1, markdown_to_html, strip_first_h1, strip_yaml_frontmatter};

    #[test]
    fn strips_leading_yaml_frontmatter_only() {
        let input = "---\ntitle: One\n---\n# Hello\nBody";
        let stripped = strip_yaml_frontmatter(input);
        assert_eq!(stripped, "# Hello\nBody");

        let no_closing = "---\ntitle: One\n# Hello\nBody";
        assert_eq!(strip_yaml_frontmatter(no_closing), no_closing);

        let not_leading = "Hello\n---\nvalue\n---\nBody";
        assert_eq!(strip_yaml_frontmatter(not_leading), not_leading);
    }

    #[test]
    fn extracts_first_h1_heading() {
        let input = "Text\n# First\n## Second";
        assert_eq!(extract_first_h1(input).as_deref(), Some("First"));
        assert_eq!(extract_first_h1("No headings"), None);
    }

    #[test]
    fn strips_only_the_first_h1_line() {
        let input = "# Title\n\nBody\n# Another";
        let stripped = strip_first_h1(input);
        assert_eq!(stripped, "\nBody\n# Another");

        let no_h1 = "## Subtitle\nBody";
        assert_eq!(strip_first_h1(no_h1), no_h1);
    }

    #[test]
    fn renders_markdown_features() {
        let html = markdown_to_html("| h |\n| - |\n| v |\n\n- [x] done\n\n~~gone~~");
        assert!(html.contains("<table>"));
        assert!(html.contains("type=\"checkbox\""));
        assert!(html.contains("<del>gone</del>"));
    }
}
