use std::error::Error;
use std::fs::write;
use std::process::Command;

use crate::extractor;
use crate::fetcher;
use crate::xhtml;

const BASE_CSS: &str = include_str!("../styles.css");

pub fn generate_multi_pdf(
    urls: &[String],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {

    let tmp_html = "/tmp/rmfeeder_multi.html";

    let mut toc_items = String::new();
    let mut article_sections = String::new();

    for (i, url) in urls.iter().enumerate() {
        let id = format!("article-{}", i + 1);

        let normalized = fetcher::normalize_url(url)?;
        let html = fetcher::fetch_html(&normalized)?;
        let article = extractor::extract_article(&html, Some(&normalized))
            .ok_or("Extraction failed")?;

        let title = article.title;
        let body = article.content.to_string();

        // Add to TOC
        toc_items.push_str(&format!(
            r#"<li><a href="#{id}">{title}</a></li>"#
        ));

        // Add article block
        article_sections.push_str(&format!(
r#"<section id="{id}" class="article-block">
    <h1>{title}</h1>
    {body}
    <a class="back-home" href="#toc">📄 Back to TOC</a>
</section>
"#));
    }

    // Final HTML
    let full_html = format!(
r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
{base_css}
</style>
</head>

<body>

<section id="toc" class="toc-page">
  <h1 class="toc-title">Contents</h1>
  <ul class="toc-list">
    {toc_items}
  </ul>
</section>

{article_sections}

</body>
</html>
"#,
        base_css = BASE_CSS,
        toc_items = toc_items,
        article_sections = article_sections
    );

    write(tmp_html, full_html)?;

    let status = Command::new("weasyprint")
        .arg(tmp_html)
        .arg(output_path)
        .status()?;

    if !status.success() {
        return Err("WeasyPrint PDF generation failed".into());
    }

    Ok(())
}