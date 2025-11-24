//! Minimal MCP (Model Context Protocol) server for Taskter.
//!
//! This implementation focuses on stdio transport and supports the core
//! methods needed for tooling-based assistants (initialize, ping, shutdown,
//! tools/list, tools/call). HTTP/SSE transports and resource surfaces can be
//! added incrementally on top of this module.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{
    self, AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader,
};

use crate::agent::FunctionDeclaration;
use crate::tools;

const JSONRPC: &str = "2.0";
const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

#[derive(Debug, Deserialize)]
struct RpcRequest {
    #[serde(default)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i64,
    message: String,
}

#[derive(Debug, Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

fn rpc_ok(id: Option<&Value>, result: Value) -> RpcResponse {
    RpcResponse {
        jsonrpc: JSONRPC,
        id: id.cloned(),
        result: Some(result),
        error: None,
    }
}

fn rpc_err(id: Option<&Value>, code: i64, message: impl Into<String>) -> RpcResponse {
    RpcResponse {
        jsonrpc: JSONRPC,
        id: id.cloned(),
        result: None,
        error: Some(RpcError {
            code,
            message: message.into(),
        }),
    }
}

fn mcp_tool_descriptors() -> Vec<Value> {
    tools::builtin_names()
        .into_iter()
        .filter_map(tools::builtin_declaration)
        .map(|decl| {
            // Avoid moving the whole struct more than once.
            let FunctionDeclaration {
                name,
                description,
                parameters,
            } = decl;

            // MCP expects an inputSchema; reuse the existing parameters as a
            // best-effort JSON Schema, falling back to a permissive object.
            let input_schema = match parameters {
                Value::Null => json!({ "type": "object" }),
                Value::Bool(_) | Value::Number(_) | Value::String(_) => {
                    json!({ "type": "object" })
                }
                other => other,
            };
            json!({
                "name": name,
                "description": description,
                "inputSchema": input_schema,
            })
        })
        .collect()
}

fn handle_initialize(req: &RpcRequest) -> RpcResponse {
    let result = json!({
        "protocolVersion": MCP_PROTOCOL_VERSION,
        "capabilities": {
            "tools": {},
        }
    });
    rpc_ok(req.id.as_ref(), result)
}

fn handle_ping(req: &RpcRequest) -> RpcResponse {
    rpc_ok(req.id.as_ref(), json!({}))
}

fn handle_tools_list(req: &RpcRequest) -> RpcResponse {
    rpc_ok(
        req.id.as_ref(),
        json!({
            "tools": mcp_tool_descriptors(),
        }),
    )
}

fn handle_tools_call(req: &RpcRequest) -> RpcResponse {
    let name = req
        .params
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string);
    let args = req
        .params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let Some(tool_name) = name else {
        return rpc_err(req.id.as_ref(), -32602, "Missing tool name");
    };

    let output = match tools::execute_tool(&tool_name, &args) {
        Ok(o) => o,
        Err(e) => {
            return rpc_err(
                req.id.as_ref(),
                -32000,
                format!("Tool `{tool_name}` failed: {e}"),
            )
        }
    };

    rpc_ok(
        req.id.as_ref(),
        json!({
            "content": [{
                "type": "text",
                "text": output,
            }]
        }),
    )
}

fn handle_shutdown(req: &RpcRequest) -> RpcResponse {
    rpc_ok(req.id.as_ref(), json!({}))
}

fn dispatch(req: &RpcRequest) -> (RpcResponse, bool) {
    match req.method.as_str() {
        "initialize" => (handle_initialize(req), false),
        "ping" => (handle_ping(req), false),
        "tools/list" => (handle_tools_list(req), false),
        "tools/call" => (handle_tools_call(req), false),
        "shutdown" => (handle_shutdown(req), true),
        other => (
            rpc_err(
                req.id.as_ref(),
                -32601,
                format!("Method `{other}` not implemented"),
            ),
            false,
        ),
    }
}

async fn handle_line(line: &str) -> (RpcResponse, bool) {
    let parsed: RpcRequest = match serde_json::from_str(line) {
        Ok(req) => req,
        Err(err) => {
            return (rpc_err(None, -32700, format!("Invalid JSON: {err}")), false);
        }
    };

    if !parsed.jsonrpc.is_empty() && parsed.jsonrpc != JSONRPC {
        return (
            rpc_err(
                parsed.id.as_ref(),
                -32600,
                format!("Unsupported jsonrpc version `{}`", parsed.jsonrpc),
            ),
            false,
        );
    }

    dispatch(&parsed)
}

async fn read_content_length<R: AsyncBufRead + Unpin>(reader: &mut R) -> Result<Option<usize>> {
    let mut content_length: Option<usize> = None;
    let mut line = String::new();

    loop {
        line.clear();
        let read = reader
            .read_line(&mut line)
            .await
            .context("reading MCP header line")?;

        if read == 0 {
            // EOF
            return Ok(None);
        }

        let trimmed = line.trim_end_matches(&['\r', '\n'][..]);
        if trimmed.is_empty() {
            break;
        }

        if let Some((key, value)) = trimmed.split_once(':') {
            if key.trim().eq_ignore_ascii_case("content-length") {
                let len = value
                    .trim()
                    .parse::<usize>()
                    .context("parsing Content-Length")?;
                content_length = Some(len);
            }
        }
    }

    content_length
        .map(Some)
        .ok_or_else(|| anyhow!("Missing Content-Length header"))
}

async fn serve_stream<R, W>(mut reader: R, mut writer: W) -> Result<()>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWrite + Unpin,
{
    while let Some(len) = read_content_length(&mut reader).await? {
        let mut body = vec![0u8; len];
        reader
            .read_exact(&mut body)
            .await
            .context("reading MCP body")?;
        let body_str = std::str::from_utf8(&body).context("MCP body not valid UTF-8")?;

        let (response, should_shutdown) = handle_line(body_str).await;
        let serialized = serde_json::to_string(&response).context("serializing MCP response")?;
        let header = format!("Content-Length: {}\r\n\r\n", serialized.len());

        writer
            .write_all(header.as_bytes())
            .await
            .context("write response header")?;
        writer
            .write_all(serialized.as_bytes())
            .await
            .context("write response body")?;
        writer.flush().await.context("flush MCP response")?;

        if should_shutdown {
            break;
        }
    }

    Ok(())
}

/// Serve MCP over stdio using MCP's `Content-Length` framing.
pub async fn serve_stdio() -> Result<()> {
    let reader = BufReader::new(io::stdin());
    let writer = io::stdout();
    serve_stream(reader, writer).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{duplex, AsyncReadExt};

    #[tokio::test]
    async fn tools_list_contains_builtin() {
        let req = RpcRequest {
            jsonrpc: JSONRPC.to_string(),
            id: Some(json!(1)),
            method: "tools/list".into(),
            params: json!({}),
        };
        let (resp, _) = dispatch(&req);
        let tools = resp
            .result
            .as_ref()
            .and_then(|v| v.get("tools"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(!tools.is_empty(), "expected at least one tool");
    }

    #[tokio::test]
    async fn content_length_round_trip() {
        let (client, server) = duplex(4096);
        let (server_read, server_write) = tokio::io::split(server);
        let server_reader = BufReader::new(server_read);
        let (mut client_reader, mut client_writer) = tokio::io::split(client);

        let server_task =
            tokio::spawn(async move { serve_stream(server_reader, server_write).await });

        let request_body = r#"{"jsonrpc":"2.0","id":1,"method":"ping","params":{}}"#;
        let request = format!(
            "Content-Length: {}\r\n\r\n{}",
            request_body.len(),
            request_body
        );
        client_writer.write_all(request.as_bytes()).await.unwrap();
        client_writer.shutdown().await.unwrap();

        let mut response_raw = Vec::new();
        client_reader.read_to_end(&mut response_raw).await.unwrap();
        let response = String::from_utf8(response_raw).unwrap();

        assert!(
            response.contains("Content-Length:"),
            "response missing Content-Length header"
        );
        assert!(
            response.contains(r#""result":{}"#),
            "ping response should contain empty result object"
        );

        server_task.await.unwrap().unwrap();
    }
}
