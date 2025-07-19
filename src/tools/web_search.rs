use anyhow::{anyhow, Result};
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

    let encoded = urlencoding::encode(query);
    let url = format!(
        "https://duckduckgo.com/?q={}&format=json&no_redirect=1&no_html=1",
        encoded
    );

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    match rt.block_on(async { reqwest::get(&url).await?.text().await }) {
        Ok(text) => Ok(text),
        Err(e) => Ok(format!("Failed to perform search: {e}")),
    }
}
