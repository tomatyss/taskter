# Agent System

Taskter supports LLM-based agents that can be assigned to tasks. Agents are model-agnostic via a provider layer. Gemini is the default provider today, and additional providers can be added without changing the agent loop. See [Model Providers](providers.md).

This chapter explains how the runtime coordinates models and tools, how to inspect runs, and how to extend the system.

## Execution Flow

1. **Bootstrap history** – Taskter combines the agent’s system prompt with the selected task (title plus description, when available) and asks the resolved provider for the next action.
2. **Provider response** – Providers return either:
   - `Text`: a final message, which is recorded in the task comment log and marks execution as successful.
   - `ToolCall`: name, arguments, and optional `call_id`. Tool calls are executed synchronously on the host.
3. **Tool execution** – Built-in tools are dispatched through `tools::execute_tool`. Any failure is surfaced as an agent failure with the tool error message.
4. **Loop** – Providers receive the tool result (including `call_id` wiring for multi-turn APIs) and the process repeats until a final text response arrives.
5. **Logging** – High-level events are appended to `.taskter/logs.log`. Raw provider requests and responses are mirrored to `.taskter/api_responses.log` for debugging.

If the provider requires an API key and none is present in the environment, Taskter enters **offline simulation mode**. Agents that include the `send_email` tool are treated as successful with a stubbed comment; other agents fail and explain that the required tool is unavailable. This keeps tests deterministic while signalling that a real API key is needed for end-to-end execution.

## Creating an Agent

You can create an agent using the `agent add` subcommand. You need to provide a prompt, a list of tools, and a model.

```bash
taskter agent add --prompt "You are a helpful assistant." --tools "project_files" "run_bash" --model "gemini-2.5-pro"
```

The `--tools` option accepts either paths to JSON files describing a tool or the name of a built-in tool. Built-in tools live under `tools/` in the repository, and their declarations are bundled into the binary.

You can display the registry at any time with:

```bash
taskter tools list
```

### Built-in Tool Reference

| Tool name | Purpose | Required arguments | Notes |
| --- | --- | --- | --- |
| `run_bash` | Execute a shell command inside the project directory | `command` (string) | Returns trimmed stdout; non-zero status bubbles up as a failure |
| `run_python` | Execute inline Python and return stdout | `code` (string) | Uses the system Python interpreter |
| `project_files` | Read, create, update, or search text files | `action`; create/read/update: `path`; update: `content`; search: `query` | Uses the supplied path verbatim (no sandbox); alias `file_ops` |
| `get_description` | Retrieve the project description text from `.taskter/description.md` | _none_ | Handy for planning/reporting agents |
| `send_email` / `email` | Send email via SMTP | `to`, `subject`, `body` | Requires `.taskter/email_config.json`; `email` is an alias |
| `taskter_task` | Proxy to `taskter task …` CLI | `args` (array of strings) | Invoke task subcommands (add, list, assign, execute, etc.) |
| `taskter_agent` | Proxy to `taskter agent …` CLI | `args` (array of strings) | Manage agents programmatically |
| `taskter_okrs` | Proxy to `taskter okrs …` CLI | `args` (array of strings) | Add or list OKRs |
| `taskter_tools` | Proxy to `taskter tools list` | `args` (array of strings) | Usually `["list"]`; useful for self-inspection |
| `web_search` | Fetch a DuckDuckGo summary | `query` (string) | Respects `SEARCH_API_ENDPOINT`; requires outbound network access |

## Assigning an Agent to a Task

Once you have created an agent, you can assign it to a task using the `assign` subcommand:

```bash
taskter task assign --task-id 1 --agent-id 1
```

### Unassigning an Agent

Remove an agent from a task without executing it:

```bash
taskter task unassign --task-id 1
```

## Executing a Task

To execute a task with an assigned agent, use the `execute` subcommand:

```bash
taskter task execute --task-id 1
```

When a task is executed, the agent will attempt to perform the task. If successful, the task is marked as "Done". If it fails, the task is moved back to "To Do", unassigned, and a comment from the agent is added.

In the interactive board (`taskter board`), tasks assigned to an agent will be marked with a `*`. You can view the assigned agent ID and any comments by selecting the task and pressing `Enter`.

## Updating an Agent

Use the `agent update` command to modify an existing agent's configuration:

```bash
taskter agent update --id 1 --prompt "New prompt" --tools "taskter_task" --model "gemini-pro"
```

All three options are required and the previous configuration is overwritten.

## Debugging Agent Runs

- **High-level activity** – Inspect `.taskter/logs.log` for a chronological record of agent starts, tool invocations, and outcomes.
- **Provider payloads** – `.taskter/api_responses.log` stores JSON requests and responses for each step; this is invaluable when bringing up a new provider or debugging schema issues.
- **CLI status** – CLI commands currently exit with status `0` even when an agent reports a failure. Rely on the printed message or the comment added to the task (along with the logs above) to detect unsuccessful runs.

## Creating Custom Tools

Agents can use custom tools in addition to the built-in ones. A tool is defined
in a JSON file with three required fields:

- `name` – the unique identifier for the tool
- `description` – a short explanation of what the tool does
- `parameters` – JSON Schema describing the arguments

An example tool definition:

```json
{
  "name": "say_hello",
  "description": "Return a greeting for the provided name",
  "parameters": {
    "type": "object",
    "properties": {
      "name": { "type": "string", "description": "Name to greet" }
    },
    "required": ["name"]
  }
}
```

Save this JSON to a file, for instance `hello_tool.json`, and pass the path to
`taskter agent add`:

```bash
taskter agent add --prompt "Be friendly" --tools "./hello_tool.json" --model "gemini-pro"
```

You can mix file paths and built-in tool names in the `--tools` list.
