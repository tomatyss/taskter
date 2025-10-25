use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use super::{ModelAction, ModelProvider};
use crate::agent::Agent;

pub struct OllamaProvider;

impl OllamaProvider {
    fn normalized_model_name(agent: &Agent) -> &str {
        let model = agent.model.trim();
        model
            .strip_prefix("ollama:")
            .or_else(|| model.strip_prefix("ollama/"))
            .or_else(|| model.strip_prefix("ollama-"))
            .unwrap_or(model)
    }

    fn base_url() -> String {
        std::env::var("OLLAMA_BASE_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "http://localhost:11434".to_string())
    }

    fn endpoint_url() -> String {
        let base = Self::base_url();
        let trimmed = base.trim_end_matches('/');
        format!("{trimmed}/api/chat")
    }
}

impl ModelProvider for OllamaProvider {
    fn name(&self) -> &'static str {
        "ollama"
    }

    fn api_key_env(&self) -> &'static str {
        "OLLAMA_API_KEY"
    }

    fn requires_api_key(&self) -> bool {
        false
    }

    fn build_history(&self, agent: &Agent, user_prompt: &str) -> Vec<Value> {
        vec![
            json!({"role": "system", "content": agent.system_prompt}),
            json!({"role": "user", "content": user_prompt}),
        ]
    }

    fn append_tool_result(
        &self,
        _agent: &Agent,
        history: &mut Vec<Value>,
        tool_name: &str,
        args: &Value,
        tool_response: &str,
        call_id: Option<&str>,
    ) {
        let id = call_id.unwrap_or("tool_call_1");
        let args_string = match args {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        history.push(json!({
            "role": "assistant",
            "tool_calls": [
                {
                    "id": id,
                    "type": "function",
                    "function": {
                        "name": tool_name,
                        "arguments": args_string
                    }
                }
            ]
        }));
        history.push(json!({
            "role": "tool",
            "tool_call_id": id,
            "name": tool_name,
            "content": tool_response
        }));
    }

    fn tools_payload(&self, agent: &Agent) -> Value {
        json!(agent
            .tools
            .iter()
            .map(|t| {
                json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters
                    }
                })
            })
            .collect::<Vec<_>>())
    }

    fn endpoint(&self, _agent: &Agent) -> String {
        Self::endpoint_url()
    }

    fn request_body(&self, agent: &Agent, history: &[Value], tools: &Value) -> Value {
        let mut body = json!({
            "model": Self::normalized_model_name(agent),
            "messages": history,
            "stream": false
        });
        if !tools.as_array().map(|a| a.is_empty()).unwrap_or(true) {
            body["tools"] = tools.clone();
        }
        body
    }

    fn parse_response(&self, response_json: &Value) -> Result<ModelAction> {
        if let Some(message) = response_json.get("message") {
            if let Some(tool_calls) = message.get("tool_calls").and_then(|t| t.as_array()) {
                if let Some(tc) = tool_calls.first() {
                    let call_id = tc.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let name = tc
                        .get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("")
                        .to_string();
                    let args_val = tc
                        .get("function")
                        .and_then(|f| f.get("arguments"))
                        .cloned()
                        .unwrap_or_else(|| json!({}));
                    let args = match args_val {
                        Value::String(s) => serde_json::from_str::<Value>(&s).unwrap_or(json!({})),
                        other => other,
                    };
                    if !name.is_empty() {
                        return Ok(ModelAction::ToolCall {
                            name,
                            args,
                            call_id,
                        });
                    }
                }
            }
            if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                return Ok(ModelAction::Text {
                    content: content.to_string(),
                });
            }
            if let Some(content_arr) = message.get("content").and_then(|c| c.as_array()) {
                // Some builds return content as array of segments
                let mut combined = String::new();
                for segment in content_arr {
                    if let Some(text) = segment.get("text").and_then(|t| t.as_str()) {
                        combined.push_str(text);
                    }
                }
                if !combined.is_empty() {
                    return Ok(ModelAction::Text { content: combined });
                }
            }
        }
        if let Some(response_text) = response_json.get("response").and_then(|r| r.as_str()) {
            return Ok(ModelAction::Text {
                content: response_text.to_string(),
            });
        }

        Err(anyhow!("No tool call or text response from the model"))
    }

    fn headers(&self, _api_key: &str) -> Vec<(String, String)> {
        vec![("Content-Type".to_string(), "application/json".to_string())]
    }
}
