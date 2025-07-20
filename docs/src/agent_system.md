# Agent System

Taskter supports LLM-based agents that can be assigned to tasks. These agents can execute tasks using a mocked Gemini API for tool-calling.

## Creating an Agent

You can create an agent using the `add-agent` subcommand. You need to provide a prompt, a list of tools, and a model.

```bash
taskter add-agent --prompt "You are a helpful assistant." --tools "email" "calendar" --model "gemini-pro"

# Create an agent capable of creating other agents

taskter add-agent --prompt "Agent factory" --tools "create_agent" --model "gemini-2.5-flash"

# Agent that can update existing agents

taskter add-agent --prompt "Agent supervisor" --tools "update_agent" --model "gemini-2.5-flash"
```

If you omit the `--model` flag, the `create_agent` tool uses `gemini-2.5-flash` by default.

The `--tools` option accepts either paths to JSON files describing a tool or the name of a built-in tool. Built-in tools are located in the `tools/` directory of the repository.

Available built-in tools:
- `create_task`
- `assign_agent`
- `create_agent`
- `add_log`
- `add_okr`
- `update_agent`
- `list_tasks`
- `list_agents`
- `get_description`
- `run_bash`
- `run_python`
- `send_email`

## Assigning an Agent to a Task

Once you have created an agent, you can assign it to a task using the `assign` subcommand:

```bash
taskter assign --task-id 1 --agent-id 1
```

## Executing a Task

To execute a task with an assigned agent, use the `execute` subcommand:

```bash
taskter execute --task-id 1
```

When a task is executed, the agent will attempt to perform the task. If successful, the task is marked as "Done". If it fails, the task is moved back to "To Do", unassigned, and a comment from the agent is added.

In the interactive board (`taskter board`), tasks assigned to an agent will be marked with a `*`. You can view the assigned agent ID and any comments by selecting the task and pressing `Enter`.
