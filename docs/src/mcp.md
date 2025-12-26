# Model Context Protocol (MCP) server

Taskter ships a minimal MCP server so you can expose its built-in tools to MCP-capable clients (e.g., IDE copilots or agent runtimes).

## What it supports

- Transport: stdio with `Content-Length` framing
- Methods: `initialize`, `ping`, `tools/list`, `tools/call`, `shutdown`
- Tools: every Taskter built-in tool is surfaced as an MCP tool descriptor; calls are forwarded to `tools/call`
- Protocol version: `2025-06-18`

Resources and alternate transports (HTTP/SSE) are not implemented yet.

## Running the server

From any initialized Taskter project, start the MCP server over stdio:

```bash
taskter mcp serve
```

The process stays attached to your terminal. MCP clients should launch Taskter with this command and communicate via stdin/stdout using the standard MCP JSON-RPC framing.

### Tips

- Ensure your client sends `Content-Length` headers and newline delimiters per MCP framing.
- For compatibility with some MCP clients, Taskter also accepts a single line-delimited JSON-RPC request (no `Content-Length` header).
- Tool arguments are passed through as JSON; Taskter returns tool output as plain text content blocks.
- Use `shutdown` to request a clean exit; EOF also ends the server loop.

### Tracing

Set `TASKTER_MCP_TRACE=1` to capture MCP traffic. By default logs are written to a temp file
(`taskter_mcp_trace.log` in your system temp directory) to avoid polluting the MCP stdout stream.
You can override the output path with `TASKTER_MCP_TRACE_FILE=/path/to/file`. If you explicitly
want stderr output (for local debugging), set `TASKTER_MCP_TRACE_STDERR=1`.
