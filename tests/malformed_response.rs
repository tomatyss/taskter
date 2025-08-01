use serde_json::json;
use taskter::agent::ExecutionResult;

#[test]
fn missing_tool_name_returns_failure() {
    let function_call = json!({"args": {}});
    let result = function_call
        .get("name")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| ExecutionResult::Failure {
            comment: "Malformed API response: missing field `name`".to_string(),
        });
    assert!(matches!(result, Err(ExecutionResult::Failure { .. })));
}

#[test]
fn non_string_text_returns_failure() {
    let part = json!({"text": 123});
    let result = part
        .get("text")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| ExecutionResult::Failure {
            comment: "Malformed API response: missing field `text`".to_string(),
        });
    assert!(matches!(result, Err(ExecutionResult::Failure { .. })));
}
