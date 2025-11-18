use std::error::Error;
use std::fs::write;
use std::process::Command;

pub fn generate_epub(
    title: &str,
    body_html: &str,
    output_path: &str,
) -> Result<(), Box<dyn Error>> 
{
    // Temporary HTML file
    let tmp_html = "/tmp/rmfeeder_tmp.html";

    // Write extracted HTML
    write(tmp_html, body_html)?;

    // Run Pandoc to produce EPUB3
    let status = Command::new("pandoc")
        .arg(tmp_html)
        .arg("-o")
        .arg(output_path)
        .arg("--from=html")
        .arg("--to=epub3")
        .arg("--metadata")
        .arg(format!("title={}", title))
        .status()?;

    if !status.success() {
        return Err("Pandoc EPUB generation failed".into());
    }

    Ok(())
}