use reqwest::blocking::Client;
use url::Url;
use std::error::Error;

pub fn normalize_url(input: &str) -> Result<String, Box<dyn Error>> {
    let parsed = Url::parse(input)?;
    Ok(parsed.into())
}

pub fn fetch_html(url: &str) -> Result<String, reqwest::Error> {
    let client = Client::builder()
        .user_agent("rmfeeder/0.1 (+https://example.com)")
        .build()?;

    let res = client.get(url).send()?.error_for_status()?;
    let body = res.text()?;
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::normalize_url;

    #[test]
    fn normalize_url_accepts_valid_http_urls() {
        let normalized = normalize_url("https://example.com/path?q=1").expect("valid URL");
        assert_eq!(normalized, "https://example.com/path?q=1");
    }

    #[test]
    fn normalize_url_rejects_invalid_urls() {
        assert!(normalize_url("not a url").is_err());
    }
}
