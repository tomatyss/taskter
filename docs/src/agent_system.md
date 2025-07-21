# Agent System

Taskter supports LLM-based agents that can be assigned to tasks. These agents can execute tasks using a mocked Gemini API for tool-calling.

## Creating an Agent

You can create an agent using the `agent add` subcommand. You need to provide a prompt, a list of tools, and a model.

```bash
taskter agent add --prompt "You are a helpful assistant." --tools "email" "calendar" --model "gemini-pro"
```

The `--tools` option accepts either paths to JSON files describing a tool or the name of a built-in tool. Built-in tools are located in the `tools/` directory of the repository.

Available built-in tools:
- `create_task`
- `assign_agent`
- `add_log`
- `add_okr`
- `list_tasks`
- `list_agents`
- `get_description`
- `run_bash`
- `run_python`
- `send_email`

You can display this list at any time with:

```bash
taskter show tools
```

## Assigning an Agent to a Task

Once you have created an agent, you can assign it to a task using the `assign` subcommand:

```bash
taskter task assign --task-id 1 --agent-id 1
```

## Executing a Task

To execute a task with an assigned agent, use the `execute` subcommand:

```bash
taskter task execute --task-id 1
```

When a task is executed, the agent will attempt to perform the task. If successful, the task is marked as "Done". If it fails, the task is moved back to "To Do", unassigned, and a comment from the agent is added.

In the interactive board (`taskter board`), tasks assigned to an agent will be marked with a `*`. You can view the assigned agent ID and any comments by selecting the task and pressing `Enter`.
