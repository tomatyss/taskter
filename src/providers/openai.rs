use anyhow::Result;
use serde_json::{json, Value};
use std::env;

use super::{ModelAction, ModelProvider};
use crate::agent::Agent;

pub struct OpenAIProvider;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RequestStyle {
    ChatCompletions,
    Responses,
}

fn with_additional_properties_false(mut params: Value) -> Value {
    if let Some(obj) = params.as_object_mut() {
        let is_object = obj
            .get("type")
            .and_then(|v| v.as_str())
            .map(|t| t == "object")
            .unwrap_or(false);
        if is_object {
            // Set additionalProperties: false if missing
            if !obj.contains_key("additionalProperties") {
                obj.insert("additionalProperties".to_string(), json!(false));
            }
            // Ensure required contains all property keys (strict mode requirement)
            if let Some(props) = obj.get("properties").and_then(|p| p.as_object()) {
                let mut all_keys: Vec<String> = props.keys().cloned().collect();
                all_keys.sort();
                let mut required = obj
                    .get("required")
                    .and_then(|r| r.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                for k in &all_keys {
                    if !required.iter().any(|r| r == k) {
                        required.push(k.clone());
                    }
                }
                obj.insert("required".to_string(), json!(required));
            }
        }
    }
    params
}

impl ModelProvider for OpenAIProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn api_key_env(&self) -> &'static str {
        "OPENAI_API_KEY"
    }

    fn build_history(&self, agent: &Agent, user_prompt: &str) -> Vec<Value> {
        match self.request_style(agent) {
            RequestStyle::Responses => vec![json!({
                "role": "user",
                "content": [ {"type": "input_text", "text": user_prompt } ]
            })],
            RequestStyle::ChatCompletions => vec![
                json!({"role": "system", "content": agent.system_prompt}),
                json!({"role": "user", "content": user_prompt}),
            ],
        }
    }

    fn append_tool_result(
        &self,
        agent: &Agent,
        history: &mut Vec<Value>,
        tool_name: &str,
        args: &Value,
        tool_response: &str,
        call_id: Option<&str>,
    ) {
        let id = call_id.unwrap_or("tool_call_1");
        match self.request_style(agent) {
            RequestStyle::Responses => {
                // For Responses API, emulate appending model output items to input
                // (function_call) followed by our function_call_output.
                let args_string = match args {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                history.push(json!({
                    "type": "function_call",
                    "call_id": id,
                    "name": tool_name,
                    "arguments": args_string
                }));
                history.push(json!({
                    "type": "function_call_output",
                    "call_id": id,
                    "output": tool_response
                }));
            }
            RequestStyle::ChatCompletions => {
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
        }
    }

    fn tools_payload(&self, agent: &Agent) -> Value {
        // Map FunctionDeclaration to the OpenAI tools schema
        match self.request_style(agent) {
            RequestStyle::Responses => json!(agent
                .tools
                .iter()
                .map(|t| {
                    let params = with_additional_properties_false(t.parameters.clone());
                    json!({
                        "type": "function",
                        "name": t.name,
                        "description": t.description,
                        "parameters": params,
                        "strict": true
                    })
                })
                .collect::<Vec<_>>()),
            RequestStyle::ChatCompletions => json!(agent
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
                .collect::<Vec<_>>()),
        }
    }

    fn endpoint(&self, agent: &Agent) -> String {
        match self.request_style(agent) {
            RequestStyle::Responses => Self::responses_endpoint(),
            RequestStyle::ChatCompletions => Self::chat_endpoint(),
        }
    }

    fn request_body(&self, agent: &Agent, history: &[Value], tools: &Value) -> Value {
        match self.request_style(agent) {
            RequestStyle::Responses => {
                let mut body = json!({
                    "model": agent.model,
                    "instructions": agent.system_prompt,
                    "input": history,
                    "tool_choice": "auto"
                });
                if !tools.as_array().map(|a| a.is_empty()).unwrap_or(true) {
                    body["tools"] = tools.clone();
                }
                if let Some(fmt) = Self::response_format_override() {
                    body["response_format"] = fmt;
                }
                body
            }
            RequestStyle::ChatCompletions => {
                let mut body = json!({
                    "model": agent.model,
                    "messages": history,
                    "tools": tools,
                    "tool_choice": "auto"
                });
                if let Some(fmt) = Self::response_format_override() {
                    body["response_format"] = fmt;
                }
                body
            }
        }
    }

    fn parse_response(&self, v: &Value) -> Result<ModelAction> {
        // Responses parsing
        if let Some(output_items) = v.get("output").and_then(|o| o.as_array()) {
            for out in output_items {
                if out.get("type").and_then(|x| x.as_str()) == Some("function_call") {
                    let call_id = out
                        .get("call_id")
                        .and_then(|x| x.as_str())
                        .map(|s| s.to_string())
                        .or_else(|| {
                            out.get("id")
                                .and_then(|x| x.as_str())
                                .map(|s| s.to_string())
                        });
                    let name = out
                        .get("name")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string();
                    let args_val = out.get("arguments").cloned().unwrap_or_else(|| json!({}));
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
                if out.get("type").and_then(|x| x.as_str()) == Some("message") {
                    if let Some(content_arr) = out.get("content").and_then(|c| c.as_array()) {
                        for item in content_arr {
                            if item.get("type").and_then(|x| x.as_str()) == Some("tool_call") {
                                let call_id = item
                                    .get("id")
                                    .and_then(|x| x.as_str())
                                    .map(|s| s.to_string());
                                let name = item
                                    .get("name")
                                    .and_then(|x| x.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let args_val =
                                    item.get("arguments").cloned().unwrap_or_else(|| json!({}));
                                let args = match args_val {
                                    Value::String(s) => {
                                        serde_json::from_str::<Value>(&s).unwrap_or(json!({}))
                                    }
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
                            if item.get("type").and_then(|x| x.as_str()) == Some("output_text") {
                                if let Some(text) = item.get("text").and_then(|x| x.as_str()) {
                                    return Ok(ModelAction::Text {
                                        content: text.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            if let Some(text) = v.get("output_text").and_then(|x| x.as_str()) {
                return Ok(ModelAction::Text {
                    content: text.to_string(),
                });
            }
        }

        // Chat Completions parsing
        if let Some(choice) = v
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.get(0))
        {
            let message = &choice["message"];
            if let Some(tc_arr) = message.get("tool_calls").and_then(|x| x.as_array()) {
                if let Some(tc) = tc_arr.get(0) {
                    let call_id = tc.get("id").and_then(|x| x.as_str()).map(|s| s.to_string());
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
            if let Some(text) = message.get("content").and_then(|c| c.as_str()) {
                return Ok(ModelAction::Text {
                    content: text.to_string(),
                });
            }
        }

        anyhow::bail!("No tool call or text response from the model")
    }

    fn headers(&self, api_key: &str) -> Vec<(String, String)> {
        vec![
            ("Authorization".to_string(), format!("Bearer {}", api_key)),
            ("Content-Type".to_string(), "application/json".to_string()),
            // Model is provided in the body; keep headers minimal.
        ]
    }
}

impl OpenAIProvider {
    fn request_style(&self, agent: &Agent) -> RequestStyle {
        if let Some(override_style) = Self::request_style_override() {
            return override_style;
        }
        Self::inferred_request_style(&agent.model)
    }

    fn request_style_override() -> Option<RequestStyle> {
        let raw = env::var("OPENAI_REQUEST_STYLE").ok()?.to_lowercase();
        match raw.as_str() {
            "responses" | "responses_api" | "responses-api" => Some(RequestStyle::Responses),
            "chat" | "chat_completions" | "chat-completions" => Some(RequestStyle::ChatCompletions),
            _ => None,
        }
    }

    fn inferred_request_style(model: &str) -> RequestStyle {
        let lower = model.to_lowercase();
        let uses_responses = lower.starts_with("gpt-5")
            || lower.starts_with("gpt5")
            || lower.starts_with("gpt-4.1")
            || lower.starts_with("gpt4.1")
            || lower.starts_with("o1")
            || lower.starts_with("o3")
            || lower.starts_with("o4")
            || lower.starts_with("omni");
        if uses_responses {
            RequestStyle::Responses
        } else {
            RequestStyle::ChatCompletions
        }
    }

    fn responses_endpoint() -> String {
        if let Ok(url) = env::var("OPENAI_RESPONSES_ENDPOINT") {
            if !url.trim().is_empty() {
                return url;
            }
        }
        let base = env::var("OPENAI_BASE_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "https://api.openai.com".to_string());
        let trimmed = base.trim_end_matches('/');
        format!("{}/v1/responses", trimmed)
    }

    fn chat_endpoint() -> String {
        if let Ok(url) = env::var("OPENAI_CHAT_ENDPOINT") {
            if !url.trim().is_empty() {
                return url;
            }
        }
        let base = env::var("OPENAI_BASE_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "https://api.openai.com".to_string());
        let trimmed = base.trim_end_matches('/');
        format!("{}/v1/chat/completions", trimmed)
    }

    fn response_format_override() -> Option<Value> {
        let raw = env::var("OPENAI_RESPONSE_FORMAT").ok()?;
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }
        if trimmed.starts_with('{') {
            serde_json::from_str::<Value>(trimmed).ok()
        } else {
            Some(json!({ "type": trimmed }))
        }
    }
}
