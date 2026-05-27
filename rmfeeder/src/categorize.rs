use std::env;
use std::error::Error;

use serde::{Deserialize, Serialize};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const MODEL: &str = "claude-haiku-4-5-20251001";

#[derive(Debug, Clone, Serialize)]
pub struct CategorizeInput {
    pub index: usize,
    pub title: String,
    pub channel: Option<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CategoryGroup {
    pub name: String,
    pub ordered_items: Vec<usize>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CategorizeResult {
    pub categories: Vec<CategoryGroup>,
    #[serde(default)]
    pub other: Vec<usize>,
}

pub fn categorize(inputs: &[CategorizeInput]) -> Result<CategorizeResult, Box<dyn Error>> {
    let api_key = env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY environment variable not set")?;

    let items_json = serde_json::to_string(inputs)?;

    let system = "You are a content organizer. Given a list of video summaries, group them into \
        thematic categories for a reading bundle. Return ONLY valid JSON with no explanation, \
        no markdown fencing, no extra text. Format: \
        {\"categories\":[{\"name\":\"Category Name\",\"ordered_items\":[indices]}],\"other\":[indices]} \
        Rules: \
        - 3-6 categories with descriptive names \
        - Order items within each category for logical reading flow \
        - Items that don't fit any theme go in the top-level 'other' array \
        - Every input index must appear exactly once across all categories and other \
        - Return pure JSON only";

    let user_content = format!(
        "Categorize these {} videos into thematic groups:\n{}",
        inputs.len(),
        items_json
    );

    let request_body = serde_json::json!({
        "model": MODEL,
        "max_tokens": 4096,
        "system": system,
        "messages": [
            {"role": "user", "content": user_content}
        ]
    });

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", &api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("content-type", "application/json")
        .json(&request_body)
        .send()?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(format!("API request failed with status {}: {}", status, body).into());
    }

    let response_json: serde_json::Value = response.json()?;
    let content = response_json["content"][0]["text"]
        .as_str()
        .ok_or("unexpected response shape from API")?;

    parse_categorize_response(content)
}

fn parse_categorize_response(text: &str) -> Result<CategorizeResult, Box<dyn Error>> {
    let json_str = extract_json(text);
    let result: CategorizeResult = serde_json::from_str(json_str)?;
    Ok(result)
}

// Strips markdown code fences if present, returning the inner JSON string.
fn extract_json(text: &str) -> &str {
    let trimmed = text.trim();
    if let Some(inner) = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
    {
        if let Some(end) = inner.rfind("```") {
            return inner[..end].trim();
        }
    }
    trimmed
}

#[cfg(test)]
mod tests {
    use super::{CategorizeResult, CategoryGroup, extract_json, parse_categorize_response};

    #[test]
    fn parse_valid_response() {
        let json = r#"{"categories":[{"name":"AI","ordered_items":[0,2]},{"name":"Tech","ordered_items":[1]}],"other":[3]}"#;
        let result = parse_categorize_response(json).unwrap();
        assert_eq!(result.categories.len(), 2);
        assert_eq!(result.categories[0].name, "AI");
        assert_eq!(result.categories[0].ordered_items, vec![0, 2]);
        assert_eq!(result.categories[1].name, "Tech");
        assert_eq!(result.categories[1].ordered_items, vec![1]);
        assert_eq!(result.other, vec![3]);
    }

    #[test]
    fn parse_missing_other_field_defaults_to_empty() {
        let json = r#"{"categories":[{"name":"AI","ordered_items":[0]}]}"#;
        let result = parse_categorize_response(json).unwrap();
        assert_eq!(result.other, Vec::<usize>::new());
    }

    #[test]
    fn parse_malformed_json_returns_err() {
        let result = parse_categorize_response("{not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn parse_empty_other_array() {
        let json = r#"{"categories":[{"name":"All","ordered_items":[0,1]}],"other":[]}"#;
        let result = parse_categorize_response(json).unwrap();
        assert_eq!(result.other, Vec::<usize>::new());
    }

    #[test]
    fn extract_json_strips_markdown_fences() {
        let fenced = "```json\n{\"a\":1}\n```";
        assert_eq!(extract_json(fenced), "{\"a\":1}");
    }

    #[test]
    fn extract_json_strips_bare_code_fences() {
        let fenced = "```\n{\"a\":1}\n```";
        assert_eq!(extract_json(fenced), "{\"a\":1}");
    }

    #[test]
    fn extract_json_passthrough_plain_json() {
        let plain = r#"{"a":1}"#;
        assert_eq!(extract_json(plain), plain);
    }

    #[test]
    fn categorize_result_equality() {
        let a = CategorizeResult {
            categories: vec![CategoryGroup {
                name: "AI".to_string(),
                ordered_items: vec![0, 1],
            }],
            other: vec![2],
        };
        let b = a.clone();
        assert_eq!(a, b);
    }
}
