use std::error::Error;
use std::io::Write;
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

const FABRIC_PATTERN: &str = "rmfeeder_categorize";

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
    let payload = serde_json::to_string(inputs)?;

    let mut child = Command::new("fabric-ai")
        .arg("--pattern")
        .arg(FABRIC_PATTERN)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        stdin.write_all(payload.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("fabric-ai categorize failed: {}", stderr.trim()).into());
    }

    let response_text = String::from_utf8_lossy(&output.stdout);
    parse_categorize_response(response_text.trim())
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
