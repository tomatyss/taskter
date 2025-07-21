use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::agent::FunctionDeclaration;
use crate::store::{self, KeyResult, Okr};
use crate::tools::Tool;
use std::collections::HashMap;

const DECL_JSON: &str = include_str!("../../tools/add_okr.json");

/// Returns the function declaration for this tool.
pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid add_okr.json")
}

/// Adds a new OKR to `.taskter/okrs.json`.
pub fn execute(args: &Value) -> Result<String> {
    let objective = args["objective"]
        .as_str()
        .ok_or_else(|| anyhow!("objective missing"))?;
    let key_results = args["key_results"]
        .as_array()
        .ok_or_else(|| anyhow!("key_results missing"))?;

    let mut okrs = store::load_okrs()?;
    let mut kr_list = Vec::new();
    for kr in key_results {
        if let Some(name) = kr.as_str() {
            kr_list.push(KeyResult {
                name: name.to_string(),
                progress: 0.0,
            });
        }
    }

    okrs.push(Okr {
        objective: objective.to_string(),
        key_results: kr_list,
    });
    store::save_okrs(&okrs)?;
    Ok(format!("Added OKR '{objective}'"))
}

/// Registers the tool in the provided map.
pub fn register(map: &mut HashMap<&'static str, Tool>) {
    map.insert(
        "add_okr",
        Tool {
            declaration: declaration(),
            execute,
        },
    );
}
