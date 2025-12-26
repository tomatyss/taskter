//! Minimal MCP (Model Context Protocol) server for Taskter.
//!
//! This implementation focuses on stdio transport and supports the core
//! methods needed for tooling-based assistants (initialize, ping, shutdown,
//! tools/list, tools/call). HTTP/SSE transports and resource surfaces can be
//! added incrementally on top of this module.

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::io::Write;
use tokio::io::{
    self, AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader,
};

use crate::agent::FunctionDeclaration;
use crate::tools;

const JSONRPC: &str = "2.0";
const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

#[derive(Debug)]
struct RpcRequest {
    jsonrpc: String,
    id: Value,
    has_id: bool,
    method: String,
    params: Value,
}

impl RpcRequest {
    fn response_id(&self) -> Option<&Value> {
        if self.has_id {
            Some(&self.id)
        } else {
            None
        }
    }
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

fn trace_enabled() -> bool {
    std::env::var_os("TASKTER_MCP_TRACE").is_some()
}

fn trace_path() -> std::path::PathBuf {
    if let Ok(path) = std::env::var("TASKTER_MCP_TRACE_FILE") {
        if !path.trim().is_empty() {
            return std::path::PathBuf::from(path);
        }
    }
    std::env::temp_dir().join("taskter_mcp_trace.log")
}

fn trace_stderr_enabled() -> bool {
    match std::env::var("TASKTER_MCP_TRACE_STDERR") {
        Ok(value) => {
            let trimmed = value.trim();
            !(trimmed.is_empty()
                || trimmed.eq_ignore_ascii_case("0")
                || trimmed.eq_ignore_ascii_case("false")
                || trimmed.eq_ignore_ascii_case("off"))
        }
        Err(_) => false,
    }
}

struct TraceLogger {
    sink: Option<Box<dyn Write + Send>>,
}

impl TraceLogger {
    fn disabled() -> Self {
        Self { sink: None }
    }

    fn new() -> Self {
        if !trace_enabled() {
            return Self::disabled();
        }

        let path = trace_path();
        let sink = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map(|file| Box::new(std::io::BufWriter::new(file)) as Box<dyn Write + Send>)
            .or_else(|_| {
                if trace_stderr_enabled() {
                    Ok(Box::new(std::io::stderr()) as Box<dyn Write + Send>)
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "trace disabled",
                    ))
                }
            })
            .ok();

        Self { sink }
    }

    fn log(&mut self, message: impl AsRef<str>) {
        if let Some(sink) = self.sink.as_mut() {
            let _ = writeln!(sink, "{}", message.as_ref());
            let _ = sink.flush();
        }
    }

    fn enabled(&self) -> bool {
        self.sink.is_some()
    }
}

fn is_notification(has_id: bool) -> bool {
    !has_id
}

fn handle_initialize(req: &RpcRequest) -> RpcResponse {
    let requested = req.params.get("protocolVersion").and_then(Value::as_str);
    let protocol_version = requested.unwrap_or(MCP_PROTOCOL_VERSION);

    let result = json!({
        "protocolVersion": protocol_version,
        "capabilities": {
            "tools": {},
        },
        "serverInfo": {
            "name": "taskter",
            "version": env!("CARGO_PKG_VERSION"),
        },
    });
    rpc_ok(req.response_id(), result)
}

fn handle_ping(req: &RpcRequest) -> RpcResponse {
    rpc_ok(req.response_id(), json!({}))
}

fn handle_tools_list(req: &RpcRequest) -> RpcResponse {
    rpc_ok(
        req.response_id(),
        json!({
            "tools": mcp_tool_descriptors(),
        }),
    )
}

async fn handle_tools_call(req: &RpcRequest) -> RpcResponse {
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
        return rpc_err(req.response_id(), -32602, "Missing tool name");
    };

    let tool_name_clone = tool_name.clone();
    let args_clone = args.clone();
    let output = match tokio::task::spawn_blocking(move || {
        tools::execute_tool(&tool_name_clone, &args_clone)
    })
    .await
    {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => {
            return rpc_err(
                req.response_id(),
                -32000,
                format!("Tool `{tool_name}` failed: {e}"),
            )
        }
        Err(e) => {
            return rpc_err(
                req.response_id(),
                -32000,
                format!("Tool `{tool_name}` panicked: {e}"),
            )
        }
    };

    rpc_ok(
        req.response_id(),
        json!({
            "content": [{
                "type": "text",
                "text": output,
            }]
        }),
    )
}

fn handle_shutdown(req: &RpcRequest) -> RpcResponse {
    rpc_ok(req.response_id(), json!({}))
}

async fn dispatch(req: &RpcRequest) -> (RpcResponse, bool) {
    match req.method.as_str() {
        "initialize" => (handle_initialize(req), false),
        "ping" => (handle_ping(req), false),
        "tools/list" => (handle_tools_list(req), false),
        "tools/call" => (handle_tools_call(req).await, false),
        "shutdown" => (handle_shutdown(req), true),
        other => (
            rpc_err(
                req.response_id(),
                -32601,
                format!("Method `{other}` not implemented"),
            ),
            false,
        ),
    }
}

fn parse_request(line: &str) -> Result<RpcRequest> {
    let value: Value = serde_json::from_str(line).context("Invalid JSON")?;
    let obj = value
        .as_object()
        .context("MCP request must be a JSON object")?;
    let jsonrpc = obj
        .get("jsonrpc")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let method = obj
        .get("method")
        .and_then(Value::as_str)
        .context("Missing method")?
        .to_string();
    let mut has_id = obj.contains_key("id");
    let mut id = obj.get("id").cloned().unwrap_or(Value::Null);
    if !has_id && method == "initialize" {
        has_id = true;
        id = Value::Null;
    }
    let params = obj.get("params").cloned().unwrap_or(Value::Null);
    Ok(RpcRequest {
        jsonrpc,
        id,
        has_id,
        method,
        params,
    })
}

async fn handle_line(line: &str) -> (Option<RpcResponse>, bool) {
    let parsed = match parse_request(line) {
        Ok(req) => req,
        Err(err) => {
            return (Some(rpc_err(None, -32700, format!("Invalid JSON: {err}"))), false);
        }
    };

    if !parsed.jsonrpc.is_empty() && parsed.jsonrpc != JSONRPC {
        let response = rpc_err(
            parsed.response_id(),
            -32600,
            format!("Unsupported jsonrpc version `{}`", parsed.jsonrpc),
        );
        return (
            if is_notification(parsed.has_id) {
                None
            } else {
                Some(response)
            },
            false,
        );
    }

    let (response, should_shutdown) = dispatch(&parsed).await;
    (
        if is_notification(parsed.has_id) {
            None
        } else {
            Some(response)
        },
        should_shutdown,
    )
}

fn looks_like_json(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('{') || trimmed.starts_with('[')
}

async fn read_message<R: AsyncBufRead + Unpin>(
    reader: &mut R,
) -> Result<Option<(Vec<String>, Vec<u8>)>> {
    let mut content_length: Option<usize> = None;
    let mut line = String::new();
    let mut headers = Vec::new();

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
            continue;
        }

        if looks_like_json(trimmed) {
            return Ok(Some((Vec::new(), trimmed.as_bytes().to_vec())));
        }

        headers.push(trimmed.to_string());
        if let Some((key, value)) = trimmed.split_once(':') {
            if key.trim().eq_ignore_ascii_case("content-length") {
                let len = value
                    .trim()
                    .parse::<usize>()
                    .context("parsing Content-Length")?;
                content_length = Some(len);
            }
        }
        break;
    }

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

        headers.push(trimmed.to_string());
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

    let Some(len) = content_length else {
        return Err(anyhow!("Missing Content-Length header"));
    };

    let mut body = vec![0u8; len];
    reader
        .read_exact(&mut body)
        .await
        .context("reading MCP body")?;
    Ok(Some((headers, body)))
}

async fn serve_stream<R, W>(mut reader: R, mut writer: W) -> Result<()>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut trace = TraceLogger::new();
    if trace.enabled() {
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("<unknown>"));
        trace.log(format!(
            "MCP server started (pid={}, cwd={:?})",
            std::process::id(),
            cwd
        ));
    }

    loop {
        let (headers, body) = match read_message(&mut reader).await {
            Ok(Some(value)) => value,
            Ok(None) => break,
            Err(err) => {
                if trace.enabled() {
                    trace.log(format!("MCP header error: {err:#}"));
                }
                return Err(err);
            }
        };
        let body_str = std::str::from_utf8(&body).context("MCP body not valid UTF-8")?;

        let response_as_line = headers.is_empty();
        let (response, should_shutdown) = handle_line(body_str).await;
        if trace.enabled() {
            if headers.is_empty() {
                trace.log("MCP <- headers: (none, line-delimited request)");
            } else {
                trace.log(format!("MCP <- headers: {:?}", headers));
            }
            trace.log(format!("MCP <- body: {body_str}"));
        }
        if let Some(response) = response {
            let serialized =
                serde_json::to_string(&response).context("serializing MCP response")?;

            if trace.enabled() {
                trace.log(format!("MCP -> body: {serialized}"));
            }

            if response_as_line {
                writer
                    .write_all(serialized.as_bytes())
                    .await
                    .context("write response body")?;
                writer
                    .write_all(b"\n")
                    .await
                    .context("write response terminator")?;
            } else {
                let header = format!(
                    "Content-Length: {}\r\nContent-Type: application/json\r\n\r\n",
                    serialized.len()
                );
                writer
                    .write_all(header.as_bytes())
                    .await
                    .context("write response header")?;
                writer
                    .write_all(serialized.as_bytes())
                    .await
                    .context("write response body")?;
            }
            writer.flush().await.context("flush MCP response")?;
        } else if trace.enabled() {
            trace.log("MCP -> (notification, no response)");
        }

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
            id: json!(1),
            has_id: true,
            method: "tools/list".into(),
            params: json!({}),
        };
        let (resp, _) = dispatch(&req).await;
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
            !response.contains("Content-Length:"),
            "line-delimited response should not include headers"
        );
        assert!(
            response.ends_with('\n'),
            "line-delimited response should end with newline"
        );
        let trimmed = response.trim_end();
        let parsed: RpcResponse = serde_json::from_str(trimmed).unwrap();
        assert_eq!(parsed.result, Some(json!({})));

        server_task.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn line_delimited_request_round_trip() {
        let (client, server) = duplex(4096);
        let (server_read, server_write) = tokio::io::split(server);
        let server_reader = BufReader::new(server_read);
        let (mut client_reader, mut client_writer) = tokio::io::split(client);

        let server_task =
            tokio::spawn(async move { serve_stream(server_reader, server_write).await });

        let request_body = r#"{"jsonrpc":"2.0","id":1,"method":"ping","params":{}}"#;
        let request = format!("{request_body}\n");
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
