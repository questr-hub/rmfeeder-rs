// Include your original styles.css
const BASE_CSS: &str = include_str!("../styles.css");

/// CSS enhancements for browser preview (NOT for EPUB).
/// These wrap the article in a readable centered column
/// without altering your PDF/EPUB typography.
const BROWSER_CSS: &str = r#"
html, body {
  background: #fefefe;
}

main.article {
  max-width: 700px;       /* classic readable line length */
  margin: 2.5rem auto;    /* center the article */
  padding: 0 1.25rem;     /* keep text off the edges */
  box-sizing: border-box;
}

img {
  border-radius: 4px;
}
"#;

pub fn wrap(title: &str, body_html: &str) -> String {
    format!(
r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>{title}</title>
    <style>
{base}
{browser}
    </style>
  </head>
  <body>
    <main class="article">
{body}
    </main>
  </body>
</html>
"#,
        title = title,
        base = BASE_CSS,
        browser = BROWSER_CSS,
        body = body_html,
    )
}