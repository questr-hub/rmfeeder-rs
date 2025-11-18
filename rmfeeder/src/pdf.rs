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

    // Today’s date for the cover page
    let today = chrono::Local::now().format("%B %e, %Y").to_string();

    // Build HTML with a cover page, article header, and your CSS
    let full_html = format!(
r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>{title}</title>
<style>
{base_css}
</style>
</head>

<body>

<!-- ===== COVER PAGE ===== -->
<section class="cover-page">
  <div class="cover-title">{title}</div>
  <div class="cover-subtitle">rmfeeder Article</div>
  <div class="cover-date">{today}</div>
</section>

<!-- ===== ARTICLE CONTENT ===== -->
<main class="article-content">

  <header class="article-header">
    <h1 class="article-title">{title}</h1>
  </header>

  {body}

</main>

</body>
</html>
"#,
        title = title,
        base_css = BASE_CSS,
        today = today,
        body = body_html
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