/// Wrap a fragment of article HTML in a minimal HTML5 shell
/// so you can open it directly in a browser as `output.html`.
pub fn wrap(title: &str, body_html: &str) -> String {
    format!(
r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>{}</title>
  </head>
  <body>
{}
  </body>
</html>
"#,
        title,
        body_html
    )
}