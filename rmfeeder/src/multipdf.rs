use std::error::Error;
use std::fs::write;
use std::process::Command;
use std::thread;
use std::time::Duration;

use crate::{escape_html, extractor, fetcher, summarize_html, temp_html_path, PageSize};
use reqwest::StatusCode;

const BASE_CSS: &str = include_str!("../styles.css");

pub fn generate_multi_pdf(
    urls: &[String],
    output_path: &str,
    delay_secs: u64,
    summarize: bool,
    pattern: &str,
    page_size: PageSize,
) -> Result<(), Box<dyn Error>> {
    let mut articles: Vec<(String, String)> = Vec::new();

    // -------- Fetch + extract articles --------
    for url in urls {
        eprintln!("Fetching {}", url);
        let normalized = match fetcher::normalize_url(url) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("Skipping {}: invalid URL: {}", url, e);
                continue;
            }
        };

        let html = match fetcher::fetch_html(&normalized) {
            Ok(body) => body,
            Err(e) => {
                if let Some(status) = e.status() {
                    if status == StatusCode::FORBIDDEN {
                        eprintln!("Skipping {}: got 403 Forbidden", url);
                    } else {
                        eprintln!("Skipping {}: HTTP {}", url, status);
                    }
                } else {
                    eprintln!("Skipping {}: request error: {}", url, e);
                }
                continue;
            }
        };

        let article = match extractor::extract_article(&html, Some(&normalized)) {
            Some(value) => value,
            None => {
                eprintln!("Skipping {}: extraction failed", url);
                continue;
            }
        };

        let title = article.title;
        let content_html = if summarize {
            match summarize_html(&article.content.to_string(), &normalized, pattern) {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("Skipping {}: summary failed: {}", url, e);
                    continue;
                }
            }
        } else {
            article.content.to_string()
        };
        articles.push((title, content_html));

        if delay_secs > 0 {
            thread::sleep(Duration::from_secs(delay_secs));
        }
    }

    generate_pdf_bundle(
        &articles,
        output_path,
        "rmfeeder ::<br>Reading Bundle",
        "Collected Articles",
        page_size,
    )
}

pub fn generate_pdf_bundle(
    articles: &[(String, String)],
    output_path: &str,
    cover_title: &str,
    cover_subtitle: &str,
    page_size: PageSize,
) -> Result<(), Box<dyn Error>> {
    if articles.is_empty() {
        return Err("No articles fetched".into());
    }

    // -------- Build Cover Page --------
    let today = chrono::Local::now().format("%B %e, %Y").to_string();
    let safe_cover_title = escape_html(cover_title).replace("&lt;br&gt;", "<br>");
    let safe_cover_subtitle = escape_html(cover_subtitle);

    let cover_html = format!(
        "<section class=\"cover-page\">
            <h1 class=\"cover-title\">{title}</h1>
            <h2 class=\"cover-subtitle\">{subtitle}</h2>
            <p class=\"cover-date\">{date}</p>
        </section>",
        title = safe_cover_title,
        subtitle = safe_cover_subtitle,
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
{page_override_css}
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
        page_override_css = page_size.page_override_css(),
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
