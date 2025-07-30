use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;

use crate::agent::FunctionDeclaration;
use crate::tools::Tool;

const DECL_JSON: &str = include_str!("../../tools/web_search.json");

pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid web_search.json")
}

async fn search_online(query: &str) -> Result<String> {
    let endpoint = std::env::var("SEARCH_API_ENDPOINT")
        .unwrap_or_else(|_| "https://api.duckduckgo.com".to_string());
    let url = reqwest::Url::parse_with_params(&endpoint, &[("q", query), ("format", "json")])?;
    let resp = reqwest::get(url).await?;
    let json: Value = resp.json().await?;
    if let Some(text) = json["AbstractText"].as_str() {
        if !text.is_empty() {
            return Ok(text.to_string());
        }
    }
    if let Some(arr) = json["RelatedTopics"].as_array() {
        if let Some(first) = arr.iter().find_map(|t| t["Text"].as_str()) {
            return Ok(first.to_string());
        }
    }
    Ok("No results found".to_string())
}

/// Performs a simple web search using DuckDuckGo.
///
/// # Errors
///
/// Returns an error if the `query` argument is missing or if the HTTP request
/// fails.
pub fn execute(args: &Value) -> Result<String> {
    let query = args["query"]
        .as_str()
        .ok_or_else(|| anyhow!("query missing"))?;
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(search_online(query))
}

pub fn register(map: &mut HashMap<&'static str, Tool>) {
    map.insert(
        "web_search",
        Tool {
            declaration: declaration(),
            execute,
        },
    );
}
