use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write as _;

use crate::agent::Agent;

#[derive(Debug)]
pub enum ModelAction {
    ToolCall { name: String, args: Value, call_id: Option<String> },
    Text { content: String },
}

pub trait ModelProvider {
    fn name(&self) -> &'static str;
    fn api_key_env(&self) -> &'static str;
    fn build_history(&self, agent: &Agent, user_prompt: &str) -> Vec<Value>;
    fn append_tool_result(
        &self,
        agent: &Agent,
        history: &mut Vec<Value>,
        tool_name: &str,
        args: &Value,
        tool_response: &str,
        call_id: Option<&str>,
    );
    fn tools_payload(&self, agent: &Agent) -> Value;
    fn endpoint(&self, agent: &Agent) -> String;
    fn request_body(&self, agent: &Agent, history: &[Value], tools: &Value) -> Value;
    fn parse_response(&self, response_json: &Value) -> Result<ModelAction>;
    fn headers(&self, api_key: &str) -> Vec<(String, String)>;

    fn infer<'a>(
        &'a self,
        client: &'a Client,
        agent: &'a Agent,
        api_key: &'a str,
        history: &'a [Value],
    ) -> futures::future::BoxFuture<'a, Result<ModelAction>> where Self: Sync {
        use futures::FutureExt;
        async move {
            let tools = self.tools_payload(agent);
            let body = self.request_body(agent, history, &tools);
            let mut req = client.post(self.endpoint(agent));
            for (k, v) in self.headers(api_key) {
                req = req.header(k, v);
            }
            // Best-effort debug logging of request
            let _ = (|| -> std::io::Result<()> {
                let path = crate::config::responses_log_path();
                if !path.exists() {
                    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
                }
                let mut f = OpenOptions::new().create(true).append(true).open(path)?;
                writeln!(f, "REQUEST provider={} model={} agent={} json={}", self.name(), agent.model, agent.id, serde_json::to_string(&body).unwrap_or_default())?;
                Ok(())
            })();

            let response = req.json(&body).send().await?;
            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("status {}: {}", status, text);
            }
            let json = response.json::<Value>().await?;
            // Best-effort debug logging of raw responses
            let _ = (|| -> std::io::Result<()> {
                let path = crate::config::responses_log_path();
                if !path.exists() {
                    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
                }
                let mut f = OpenOptions::new().create(true).append(true).open(path)?;
                writeln!(f, "provider={} model={} agent={} json={}", self.name(), agent.model, agent.id, json)?;
                Ok(())
            })();
            self.parse_response(&json)
        }
        .boxed()
    }
}

pub mod gemini;
pub mod openai;

pub fn select_provider(agent: &Agent) -> Box<dyn ModelProvider + Send + Sync> {
    // Simple heuristic by model name; default to Gemini for backward compatibility.
    let model = agent.model.to_lowercase();
    if model.starts_with("gemini") {
        Box::new(gemini::GeminiProvider)
    } else if model.starts_with("gpt-") {
        Box::new(openai::OpenAIProvider)
    } else {
        // Default for now
        Box::new(gemini::GeminiProvider)
    }
}
