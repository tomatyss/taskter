use serde_json::json;

use taskter::agent::{Agent, FunctionDeclaration};
use taskter::providers::{openai::OpenAIProvider, select_provider, ModelAction, ModelProvider};

fn base_agent(model: &str) -> Agent {
    Agent {
        id: 42,
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
        schedule: None,
        repeat: false,
    }
}

#[test]
fn select_provider_picks_openai_for_gpt_models() {
    let agent = base_agent("gpt-4.1");
    let p = select_provider(&agent);
    assert_eq!(p.name(), "openai");
}

#[test]
fn openai_chat_parses_tool_call_and_text() {
    let provider = OpenAIProvider;
    // Chat Completions: tool call path
    let v = json!({
        "choices": [
            {"message": {"tool_calls": [
                {"id": "call_123", "type": "function", "function": {"name": "run_bash", "arguments": "{\"command\":\"echo hi\"}"}}
            ]}}
        ]
    });
    let action = provider.parse_response(&v).expect("tool call parsed");
    match action {
        ModelAction::ToolCall { name, args, call_id } => {
            assert_eq!(name, "run_bash");
            assert_eq!(args["command"], "echo hi");
            assert_eq!(call_id.as_deref(), Some("call_123"));
        }
        _ => panic!("expected tool call"),
    }

    // Chat Completions: text path
    let v = json!({
        "choices": [
            {"message": {"content": "done"}}
        ]
    });
    let action = provider.parse_response(&v).expect("text parsed");
    match action {
        ModelAction::Text { content } => assert_eq!(content, "done"),
        _ => panic!("expected text"),
    }
}

#[test]
fn openai_responses_parses_function_call_and_message() {
    let provider = OpenAIProvider;
    // Responses: function_call item
    let v = json!({
        "output": [
            {"type": "reasoning", "summary": []},
            {"type": "function_call", "id": "fc_1", "call_id": "call_1", "name": "run_bash", "arguments": "{\"command\":\"echo hello\"}"}
        ]
    });
    let action = provider.parse_response(&v).expect("function_call parsed");
    match action {
        ModelAction::ToolCall { name, args, call_id } => {
            assert_eq!(name, "run_bash");
            assert_eq!(args["command"], "echo hello");
            assert_eq!(call_id.as_deref(), Some("call_1"));
        }
        _ => panic!("expected tool call"),
    }

    // Responses: message with output_text
    let v = json!({
        "output": [
            {"type": "message", "role": "assistant", "content": [
                {"type": "output_text", "text": "ok"}
            ]}
        ]
    });
    let action = provider.parse_response(&v).expect("text parsed");
    match action {
        ModelAction::Text { content } => assert_eq!(content, "ok"),
        _ => panic!("expected text"),
    }
}

#[test]
fn append_tool_result_shapes_are_correct() {
    let provider = OpenAIProvider;

    // Chat Completions history mutation
    let agent_chat = base_agent("gpt-4.1");
    let mut hist_chat = Vec::new();
    provider.append_tool_result(
        &agent_chat,
        &mut hist_chat,
        "run_bash",
        &json!({"command":"ls"}),
        "ok",
        Some("call_abc"),
    );
    assert_eq!(hist_chat.len(), 2);
    assert_eq!(hist_chat[0]["role"], "assistant");
    assert_eq!(hist_chat[0]["tool_calls"][0]["id"], "call_abc");
    assert_eq!(hist_chat[0]["tool_calls"][0]["function"]["name"], "run_bash");
    assert_eq!(hist_chat[1]["role"], "tool");
    assert_eq!(hist_chat[1]["tool_call_id"], "call_abc");

    // Responses history mutation
    let agent_resp = base_agent("gpt-5");
    let mut hist_resp = Vec::new();
    provider.append_tool_result(
        &agent_resp,
        &mut hist_resp,
        "run_bash",
        &json!({"command":"echo hi"}),
        "hello from tool",
        Some("call_xyz"),
    );
    assert_eq!(hist_resp.len(), 2);
    assert_eq!(hist_resp[0]["type"], "function_call");
    assert_eq!(hist_resp[0]["call_id"], "call_xyz");
    assert_eq!(hist_resp[0]["name"], "run_bash");
    assert_eq!(hist_resp[1]["type"], "function_call_output");
    assert_eq!(hist_resp[1]["call_id"], "call_xyz");
    assert_eq!(hist_resp[1]["output"], "hello from tool");
}

