use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde_json::Value;

use crate::agent::FunctionDeclaration;

const DECL_JSON: &str = include_str!("../../tools/web_search.json");

pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid web_search.json")
}

pub fn execute(args: &Value) -> Result<String> {
    let query = args["query"]
        .as_str()
        .ok_or_else(|| anyhow!("query missing"))?;
    let base = std::env::var("SEARCH_API_BASE_URL")
        .unwrap_or_else(|_| "https://api.duckduckgo.com/".to_string());
    let url = format!("{base}?q={query}&format=json&no_redirect=1&skip_disambig=1");
    let client = Client::builder().user_agent("taskter/0.1.0").build()?;
    let resp = client.get(&url).send().map_err(|e| anyhow!(e))?;
    if !resp.status().is_success() {
        return Err(anyhow!("request failed: {}", resp.status()));
    }
    let json: Value = resp.json().map_err(|e| anyhow!(e))?;
    let heading = json.get("Heading").and_then(|v| v.as_str()).unwrap_or("");
    let abstract_text = json
        .get("AbstractText")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if heading.is_empty() && abstract_text.is_empty() {
        return Ok("No results".to_string());
    }
    Ok(format!("{heading}: {abstract_text}"))
}
