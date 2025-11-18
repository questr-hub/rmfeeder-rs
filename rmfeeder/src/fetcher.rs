use reqwest::blocking::Client;
use url::Url;
use std::error::Error;

pub fn normalize_url(input: &str) -> Result<String, Box<dyn Error>> {
    let parsed = Url::parse(input)?;
    Ok(parsed.into())
}

pub fn fetch_html(url: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::builder()
        .user_agent("rmfeeder/0.1 (+https://example.com)")
        .build()?;

    let res = client.get(url).send()?;
    let body = res.text()?;
    Ok(body)
}