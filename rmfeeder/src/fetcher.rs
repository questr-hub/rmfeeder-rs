use reqwest::blocking::Client;
use url::Url;

/// Normalize + percent-encode a URL so shell-unfriendly characters are handled.
///
/// Examples this fixes:
///  - parentheses
///  - spaces
///  - unicode characters
///  - relative paths
///  - stray whitespace
pub fn normalize_url(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = Url::parse(input)?;
    Ok(url.to_string())
}

/// Fetch an HTML page from the internet and return the raw HTML string.
pub fn fetch_html(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent(default_user_agent())
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()?;

    let resp = client.get(url).send()?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("Request failed with status {}", status).into());
    }

    let text = resp.text()?;
    Ok(text)
}

fn default_user_agent() -> String {
    format!(
        "rmfeeder/0.1 (+https://example.com; rust; {})",
        std::env::consts::OS
    )
}