# Agent System

Taskter supports LLM-based agents that can be assigned to tasks. These agents can execute tasks using a mocked Gemini API for tool-calling.

## Creating an Agent

You can create an agent using the `agent add` subcommand. You need to provide a prompt, a list of tools, and a model.

```bash
taskter agent add --prompt "You are a helpful assistant." --tools "email" "calendar" --model "gemini-pro"
```

The `--tools` option accepts either paths to JSON files describing a tool or the name of a built-in tool. Built-in tools are located in the `tools/` directory of the repository.

Available built-in tools:
- `taskter_task`
- `taskter_agent`
- `taskter_okrs`
- `taskter_tools`
- `get_description`
- `run_bash`
- `run_python`
- `send_email`
- `web_search`
- `project_files`
- `file_ops`

You can display this list at any time with:

```bash
taskter tools list
```

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

## Creating Custom Tools

Agents can use custom tools in addition to the built‑in ones. A tool is defined
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
