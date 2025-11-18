use std::error::Error;
use std::fs::write;
use std::process::Command;

const BASE_CSS: &str = include_str!("../styles.css");

pub fn generate_pdf(
    title: &str,
    body_html: &str,
    output_path: &str,
) -> Result<(), Box<dyn Error>> {

    let tmp_html = "/tmp/rmfeeder_tmp.html";

    // Full HTML document for WeasyPrint — no page CSS here.
    let full_html = format!(
r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>{title}</title>
<style>
/* Entire styling is controlled by styles.css */
{base_css}
</style>
</head>
<body>
<main class="article-content">
{body}
</main>
</body>
</html>
"#,
        title = title,
        base_css = BASE_CSS,
        body = body_html
    );

    write(tmp_html, full_html)?;

    // Call WeasyPrint directly
    let status = Command::new("weasyprint")
        .arg(tmp_html)
        .arg(output_path)
        .status()?;

    if !status.success() {
        return Err("WeasyPrint PDF generation failed".into());
    }

    Ok(())
}