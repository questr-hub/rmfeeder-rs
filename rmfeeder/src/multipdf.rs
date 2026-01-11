use std::error::Error;
use std::fs::write;
use std::process::Command;

use crate::{escape_html, extractor, fetcher, temp_html_path};

const BASE_CSS: &str = include_str!("../styles.css");

pub fn generate_multi_pdf(urls: &[String], output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut articles: Vec<(String, String)> = Vec::new();

    // -------- Fetch + extract articles --------
    for url in urls {
        let normalized = fetcher::normalize_url(url)?;
        let html = fetcher::fetch_html(&normalized)?;
        let article = extractor::extract_article(&html, Some(&normalized))
            .ok_or("Extraction failed")?;

        let title = article.title;
        let content_html = article.content.to_string();
        articles.push((title, content_html));
    }

    // -------- Build Cover Page --------
    let today = chrono::Local::now().format("%B %e, %Y").to_string();

    let cover_html = format!(
        "<section class=\"cover-page\">
            <h1 class=\"cover-title\">rmfeeder — Reading Bundle</h1>
            <h2 class=\"cover-subtitle\">Collected Articles</h2>
            <p class=\"cover-date\">{date}</p>
        </section>",
        date = today.trim()
    );

    // -------- Build TOC --------
    let mut toc_items = String::new();
    for (idx, (title, _)) in articles.iter().enumerate() {
        let id = format!("article-{}", idx + 1);
        let safe_title = escape_html(title);
        toc_items.push_str(&format!(
            "<li><a href=\"#{}\">{}</a></li>\n",
            id, safe_title
        ));
    }

    let toc_html = format!(
        "<section class=\"toc-page\">
            <h1 class=\"toc-title\">Contents</h1>
            <ul class=\"toc-list\">
            {items}
            </ul>
        </section>",
        items = toc_items
    );

    // -------- Build Article Blocks --------
    let mut article_blocks = String::new();
    for (idx, (title, content_html)) in articles.iter().enumerate() {
        let id = format!("article-{}", idx + 1);
        let safe_title = escape_html(title);

        article_blocks.push_str(&format!(
            "<section id=\"{id}\" class=\"article-block\">
                <h1>{title}</h1>
                {body}
                <p><a class=\"back-home\" href=\"#toc\">📄 Back to TOC</a></p>
            </section>\n",
            id = id,
            title = safe_title,
            body = content_html
        ));
    }

    // -------- Combine HTML --------
    let full_html = format!(
        "<!DOCTYPE html>
<html>
<head>
<meta charset=\"utf-8\">
<title>rmfeeder – Multi Article</title>
<style>
{base_css}
</style>
</head>
<body>
{cover}
<a id=\"toc\"></a>
{toc}
{articles}
</body>
</html>",
        base_css = BASE_CSS,
        cover = cover_html,
        toc = toc_html,
        articles = article_blocks
    );

    let tmp_html = temp_html_path("rmfeeder_multi_tmp");
    write(&tmp_html, full_html)?;

    // -------- Generate PDF via WeasyPrint --------
    let status = Command::new("weasyprint")
        .arg(&tmp_html)
        .arg(output_path)
        .status()?;

    let _ = std::fs::remove_file(&tmp_html);

    if !status.success() {
        return Err("WeasyPrint PDF generation failed".into());
    }

    Ok(())
}
