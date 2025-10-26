# Model Providers

Taskter’s agent system is model‑agnostic. A provider layer adapts the neutral
agent loop to a specific LLM API. Providers convert between Taskter’s message
history and the provider’s wire format, and translate responses into either a
text completion or a tool call.

All provider requests and responses are mirrored to `.taskter/api_responses.log`
so that you can inspect the exact JSON being exchanged when debugging a new
integration.

## Built-in Providers

- Gemini (default): selected when `agent.model` starts with `gemini`.
  - Env var: `GEMINI_API_KEY`
  - Code: `src/providers/gemini.rs`
- OpenAI: selected when `agent.model` starts with `gpt-4`, `gpt-5`, `gpt-4o`, `gpt-4.1`, `o1`, `o3`, `o4`, or `omni`.
  - Env var: `OPENAI_API_KEY`
  - Code: `src/providers/openai.rs`
  - APIs:
    - Chat Completions: used for models like `gpt-4o` and `gpt-4o-mini`. Tools are passed as `{"type":"function","function":{...}}` and responses carry `choices[0].message.tool_calls[]`.
    - Responses API: used for `gpt-4.1`, `gpt-5`, `o-series`, and Omni models. Input is an item list; tool calls arrive as `{"type":"function_call", name, arguments, call_id}` in `output[]` and you must append both the `function_call` and a `function_call_output` item with the same `call_id`.
  - Optional overrides:
    - `OPENAI_BASE_URL` to point at a proxy (`https://api.openai.com` by default)
    - `OPENAI_CHAT_ENDPOINT` / `OPENAI_RESPONSES_ENDPOINT` for full URL control
    - `OPENAI_REQUEST_STYLE=chat|responses` to force the request format
    - `OPENAI_RESPONSE_FORMAT` containing either a JSON blob (e.g. `{"type":"json_object"}`) or shorthand (`json_object`)
- Ollama: selected when `agent.model` starts with `ollama:`, `ollama/`, or `ollama-`.
  - Env var: `OLLAMA_BASE_URL` (defaults to `http://localhost:11434`)
  - Code: `src/providers/ollama.rs`
  - Uses the local `/api/chat` endpoint and mirrors the Chat Completions tool schema.

## Configure a Provider

- Choose a model string when creating/updating an agent (e.g. `gemini-2.5-pro`, `gpt-4.1`, `o1-mini`, or `ollama:llama3`).
- Set the provider explicitly when running CLI commands by passing `--provider gemini|openai|ollama`. To clear a stored provider, use `taskter agent update --provider none …`; new agent creation does not accept `none`. When no provider is stored Taskter falls back to model-name heuristics.
- Export the provider’s API key environment variable before running agents.
  - Gemini:
    ```bash
    export GEMINI_API_KEY=your_key_here
    ```
  - OpenAI:
    ```bash
    export OPENAI_API_KEY=your_key_here
    ```
  - Ollama does not require an API key. Optionally set `OLLAMA_BASE_URL` if your daemon listens somewhere other than `http://localhost:11434`.

If no valid API key is present, Taskter falls back to an offline simulation. No
real tool calls are made: agents that include the `send_email` tool return a
stubbed success comment, while all other agents are marked as failed so you can
spot the missing credentials.

## Add a New Provider

Implement the `ModelProvider` trait and register it in `select_provider`.

1) Create a file under `src/providers/`, e.g. `my_provider.rs`:

```rust
use serde_json::{json, Value};
use anyhow::Result;
use crate::agent::Agent;
use super::{ModelAction, ModelProvider};

pub struct OpenAIProvider;

impl ModelProvider for OpenAIProvider {
    fn name(&self) -> &'static str { "openai" }
    fn api_key_env(&self) -> &'static str { "OPENAI_API_KEY" }

    fn build_history(&self, agent: &Agent, user_prompt: &str) -> Vec<Value> {
        vec![json!({
            "role": "user",
            "content": [
                {"type": "text", "text": format!("System: {}\\nUser: {}", agent.system_prompt, user_prompt)}
            ]
        })]
    }

    fn append_tool_result(&self, history: &mut Vec<Value>, tool: &str, _args: &Value, tool_response: &str) {
        history.push(json!({
            "role": "tool",
            "content": [{
                "type": "tool_result",
                "name": tool,
                "content": tool_response
            }]
        }));
    }

    fn tools_payload(&self, agent: &Agent) -> Value {
        // Map our FunctionDeclaration to OpenAI tools schema
        json!(agent.tools.iter().map(|t| {
            json!({
                "type": "function",
                "name": t.name,
                "description": t.description,
                "parameters": t.parameters,
                "strict": true
            })
        }).collect::<Vec<_>>())
    }

    fn endpoint(&self, _agent: &Agent) -> String {
        "https://api.openai.com/v1/responses".to_string()
    }

    fn request_body(&self, history: &[Value], tools: &Value) -> Value {
        json!({
            "model": "gpt-4.1",
            "input": history,
            "tools": tools
        })
    }

    fn parse_response(&self, v: &Value) -> Result<ModelAction> {
        if let Some(tc) = v["output"][0].get("tool_calls").and_then(|x| x.get(0)) {
            let name = tc["function"]["name"].as_str().unwrap_or_default().to_string();
            let args = tc["function"]["arguments"].clone();
            return Ok(ModelAction::ToolCall { name, args });
        }
        // Fallback to text content
        let text = v["output_text"].as_str().unwrap_or("").to_string();
        Ok(ModelAction::Text { content: text })
    }

    fn headers(&self, api_key: &str) -> Vec<(String, String)> {
        vec![
            ("Authorization".into(), format!("Bearer {}", api_key)),
            ("Content-Type".into(), "application/json".into()),
        ]
    }
}
```

2) Register it in `select_provider` inside `src/providers/mod.rs`:

```rust
pub fn select_provider(agent: &Agent) -> Box<dyn ModelProvider + Send + Sync> {
    let model = agent.model.to_lowercase();
    if model.starts_with("gemini") {
        Box::new(gemini::GeminiProvider)
    } else if model.starts_with("gpt-") {
        Box::new(openai::OpenAIProvider)
    } else {
        Box::new(gemini::GeminiProvider)
    }
}
```

3) Set the API key and choose a matching model:

```bash
export OPENAI_API_KEY=your_key
# Example agent
taskter agent add --prompt "Be helpful" --tools run_bash --model my-model --provider openai
```

### Notes

- The agent loop is neutral: it asks a provider for one step, executes a tool
  if requested, and appends the result via the provider to maintain the correct
  message format.
- Providers must ensure tools are represented in the target API’s expected
  schema and that responses are robustly parsed into `ModelAction`.
- See `src/providers/gemini.rs` and `src/providers/openai.rs` as complete reference implementations.

### OpenAI Responses: Tool Calling Flow

The Responses API differs from Chat Completions. A typical multi‑turn flow:

1. Send input as a list (start with user):
   ```json
   [
     {"role":"user", "content":[{"type":"input_text","text":"Use run_bash to echo hello"}]}
   ]
   ```
2. Model returns an `output` array which can include `{"type":"function_call", "name":"run_bash", "arguments":"{\"command\":\"echo hi\"}", "call_id":"call_123"}`.
3. Execute the tool, then append to your input list:
   ```json
   {"type":"function_call","call_id":"call_123","name":"run_bash","arguments":"{\"command\":\"echo hi\"}"},
   {"type":"function_call_output","call_id":"call_123","output":"hello"}
   ```
4. Call the Responses API again with the expanded `input` and the same `tools`. The model will produce a final `message` with `output_text`.

Taskter automates these steps inside the provider, including the call_id wiring and multi‑turn loop.
