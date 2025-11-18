/// Wrap sanitized content into a simple HTML shell for preview
pub fn wrap(title: &str, body_html: &str) -> String {
    format!(
r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{}</title>
<link rel="stylesheet" href="styles.css">
</head>
<body>
<main class="article">
{}
</main>
</body>
</html>
"#,
        title, body_html
    )
}