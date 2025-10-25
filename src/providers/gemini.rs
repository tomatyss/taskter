use anyhow::Result;
use serde_json::{json, Value};

use super::{ModelAction, ModelProvider};
use crate::agent::Agent;

pub struct GeminiProvider;

impl ModelProvider for GeminiProvider {
    fn name(&self) -> &'static str {
        "gemini"
    }

    fn api_key_env(&self) -> &'static str {
        "GEMINI_API_KEY"
    }

    fn build_history(&self, agent: &Agent, user_prompt: &str) -> Vec<Value> {
        vec![json!({
            "role": "user",
            "parts": [{"text": format!("System: {}\nUser: {}", agent.system_prompt, user_prompt)}]
        })]
    }

    fn append_tool_result(
        &self,
        _agent: &Agent,
        history: &mut Vec<Value>,
        tool_name: &str,
        args: &Value,
        tool_response: &str,
        _call_id: Option<&str>,
    ) {
        history.push(json!({
            "role": "model",
            "parts": [{"functionCall": {"name": tool_name, "args": args}}]
        }));
        history.push(json!({
            "role": "tool",
            "parts": [{"functionResponse": {"name": tool_name, "response": {"content": tool_response}}}]
        }));
    }

    fn tools_payload(&self, agent: &Agent) -> Value {
        json!({"functionDeclarations": agent.tools})
    }

    fn endpoint(&self, agent: &Agent) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            agent.model
        )
    }

    fn request_body(&self, _agent: &Agent, history: &[Value], tools: &Value) -> Value {
        json!({
            "contents": history,
            "tools": [tools]
        })
    }

    fn parse_response(&self, response_json: &Value) -> Result<ModelAction> {
        let candidate = &response_json["candidates"][0];
        let part = &candidate["content"]["parts"][0];

        if let Some(function_call) = part.get("functionCall") {
            let tool_name = function_call
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Malformed API response: missing field `name`"))?;
            let args = function_call
                .get("args")
                .cloned()
                .unwrap_or_else(|| json!({}));
            return Ok(ModelAction::ToolCall {
                name: tool_name.to_string(),
                args,
                call_id: None,
            });
        }

        if let Some(text) = part.get("text").and_then(Value::as_str) {
            return Ok(ModelAction::Text {
                content: text.to_string(),
            });
        }

        anyhow::bail!("No tool call or text response from the model")
    }

    fn headers(&self, api_key: &str) -> Vec<(String, String)> {
        vec![
            ("x-goog-api-key".to_string(), api_key.to_string()),
            ("Content-Type".to_string(), "application/json".to_string()),
        ]
    }
}
