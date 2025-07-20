# CLI Usage

Taskter exposes multiple subcommands. Run `taskter --help` to see the available options. The README lists common workflows.

## Quick Start

This section provides a quick overview of how to get started with Taskter.

### 1. Initialize the board

First, navigate to your project's directory and initialize the Taskter board:

```bash
taskter init
```

This will create a `.taskter` directory to store all your tasks, agents, and project data.

### 2. Create an agent

Next, create an agent to help you with your tasks. For this example, we'll create a simple agent that can run bash commands:

```bash
taskter add-agent --prompt "You are a helpful assistant that can run bash commands." --tools "run_bash" --model "gemini-pro"
```

You can list all available agents using:

```bash
taskter show agents
```

### 3. Create a task

Now, let's create a task for your agent to complete:

```bash
taskter add -t "List files in the current directory" -d "Use the ls -la command to list all files and folders in the current directory."
```

You can see all your tasks by running:

```bash
taskter list
```

### 4. Assign the task to an agent

Assign the newly created task to your agent:

```bash
taskter assign --task-id 1 --agent-id 1
```

### 5. Execute the task

Finally, execute the task:

```bash
taskter execute --task-id 1
```

The agent will now run the task. If it's successful, the task will be marked as "Done". You can view the board at any time using the interactive UI:

```bash
taskter board
```
