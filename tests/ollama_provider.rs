use serde_json::json;

use taskter::agent::{Agent, FunctionDeclaration};
use taskter::providers::{ollama::OllamaProvider, select_provider, ModelAction, ModelProvider};

fn base_agent(model: &str) -> Agent {
    Agent {
        id: 99,
        system_prompt: "You are helpful.".to_string(),
        tools: vec![FunctionDeclaration {
            name: "run_bash".to_string(),
            description: Some("Execute a bash command and return its output".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {"command": {"type": "string"}},
                "required": ["command"],
                "additionalProperties": false
            }),
        }],
        model: model.to_string(),
        provider: Some("ollama".into()),
        schedule: None,
        repeat: false,
    }
}

#[test]
fn select_provider_picks_ollama() {
    let agent = base_agent("ollama:llama3.1");
    let provider = select_provider(&agent);
    assert_eq!(provider.name(), "ollama");
}

#[test]
fn ollama_history_includes_system_and_user() {
    let provider = OllamaProvider;
    let agent = base_agent("ollama:llama3");
    let history = provider.build_history(&agent, "Hello");
    assert_eq!(history.len(), 2);
    assert_eq!(history[0]["role"], "system");
    assert_eq!(history[1]["role"], "user");
}

#[test]
fn ollama_append_tool_result_shapes_are_correct() {
    let provider = OllamaProvider;
    let agent = base_agent("ollama/llama3");
    let mut history = Vec::new();
    provider.append_tool_result(
        &agent,
        &mut history,
        "run_bash",
        &json!({"command": "ls"}),
        "output",
        Some("call_42"),
    );
    assert_eq!(history.len(), 2);
    assert_eq!(history[0]["role"], "assistant");
    assert_eq!(history[0]["tool_calls"][0]["id"], "call_42");
    assert_eq!(history[1]["role"], "tool");
    assert_eq!(history[1]["tool_call_id"], "call_42");
}

#[test]
fn ollama_parse_response_tool_call_and_text() {
    let provider = OllamaProvider;

    let tool_call = json!({
        "message": {
            "role": "assistant",
            "tool_calls": [{
                "id": "call_007",
                "type": "function",
                "function": {
                    "name": "run_bash",
                    "arguments": "{\"command\":\"echo hi\"}"
                }
            }]
        }
    });
    let action = provider
        .parse_response(&tool_call)
        .expect("tool call parsed");
    match action {
        ModelAction::ToolCall {
            name,
            args,
            call_id,
        } => {
            assert_eq!(name, "run_bash");
            assert_eq!(args["command"], "echo hi");
            assert_eq!(call_id.as_deref(), Some("call_007"));
        }
        ModelAction::Text { .. } => panic!("expected tool call"),
    }

    let text_resp = json!({
        "message": {
            "role": "assistant",
            "content": "done"
        }
    });
    let action = provider.parse_response(&text_resp).expect("text parsed");
    match action {
        ModelAction::Text { content } => assert_eq!(content, "done"),
        ModelAction::ToolCall { .. } => panic!("expected text"),
    }
}

#[test]
fn ollama_request_body_includes_tools() {
    let provider = OllamaProvider;
    let agent = base_agent("ollama-phi3");
    let history = provider.build_history(&agent, "Ping");
    let tools = provider.tools_payload(&agent);
    let body = provider.request_body(&agent, &history, &tools);
    assert_eq!(body["model"], "phi3");
    assert_eq!(body["messages"].as_array().unwrap().len(), 2);
    assert!(body["tools"].as_array().is_some());
}
