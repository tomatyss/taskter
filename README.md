# Taskter

[![Test](https://github.com/tomatyss/taskter/actions/workflows/test.yml/badge.svg)](https://github.com/tomatyss/taskter/actions/workflows/test.yml)
[![Crates.io](https://img.shields.io/crates/v/taskter)](https://crates.io/crates/taskter)
[![Documentation](https://img.shields.io/badge/docs-gh--pages-informational)](https://tomatyss.github.io/taskter/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Taskter is a terminal Kanban board CLI tool built with Rust.

> **Warning**
> This project is currently in a *pre-alpha* state and is actively maintained. Expect breaking changes and incomplete features.

## Features

- Kanban board with tasks (ToDo, InProgress, Done)
- Project description
- Operation logs
- OKRs (Objectives and Key Results)
- Parallel task execution when running the scheduler

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
taskter agent add --prompt "You are a helpful assistant that can run bash commands." --tools "run_bash" --model "gemini-2.5-pro"
```

You can list all available agents using:

```bash
taskter agent list
```

### 3. Create a task

Now, let's create a task for your agent to complete:

```bash
taskter task add -t "List files in the current directory" -d "Use the ls -la command to list all files and folders in the current directory."
```

You can see all your tasks by running:

```bash
taskter task list
```

### 4. Assign the task to an agent

Assign the newly created task to your agent:

```bash
taskter task assign --task-id 1 --agent-id 1
```

If you need to remove the agent before execution:

```bash
taskter task unassign --task-id 1
```

### 5. Execute the task

Finally, execute the task:

```bash
taskter task execute --task-id 1
```

The agent will now run the task. If it's successful, the task will be marked as "Done". You can view the board at any time using the interactive UI:

```bash
taskter board
```


## Build and Installation

Taskter can be installed from prebuilt packages or built from source.

### Homebrew

```
brew tap tomatyss/taskter
brew install taskter
```

### Linux packages

Prebuilt `.deb` archives are generated with `cargo deb` and published on the
GitHub release page.  You can install them with:

```
sudo dpkg -i taskter_0.1.0_amd64.deb
```

For Alpine Linux an `APKBUILD` script is provided under `packaging/apk/`.  Run
`abuild -r` inside that directory to create an `apk` package.

### Build from source

To build Taskter from source, you need to have Rust and Cargo installed.

1. Clone the repository:
   ```bash
   git clone <repository_url>
   cd taskter
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

The executable will be located at `target/release/taskter`. To make it available system-wide, you can copy it to a directory that is in your system's `PATH`.

For example, on macOS or Linux, you can move it to `/usr/local/bin`:
```bash
sudo cp target/release/taskter /usr/local/bin/taskter
```
After this, you can run `taskter` from any directory.

Alternatively, you can install the latest published version from [crates.io](https://crates.io/crates/taskter):
```bash
cargo install taskter
```
You can also install directly from the repository for a development build:
```bash
cargo install --path .
```
Both methods place the `taskter` executable in your Cargo bin directory (usually `~/.cargo/bin/`), which should be in your `PATH`.

## Docker

You can build and run Taskter without installing Rust locally by using Docker.
The included `Dockerfile` uses the official `rust:1.88.0` image to build the
application.

Build the container image:

```bash
docker build -t taskter .
```

Start the application with Docker Compose. If you use the Gemini integration,
pass your API key as an environment variable:

```bash
GEMINI_API_KEY=<your_key> docker compose run --rm taskter --help
```


## Usage

### Initialize the board

To start using Taskter, you need to initialize a board in your project's directory:

```bash
taskter init
```

This will create a `.taskter` directory with the necessary files.

### Interactive Board

Taskter also provides an interactive terminal UI to manage the Kanban board.

```bash
taskter board
```

In the interactive board, you can use the following keys:
- `q`: Quit
- `←` / `→` or `Tab`: Navigate between columns
- `↑` / `↓`: Navigate between tasks
- `h` / `l`: Move a task to the previous/next column
- `a`: Assign an agent to the selected task
- `r`: Unassign the selected task's agent
- `c`: Add a comment to the selected task
- `n`: Create a new task
- `u`: Edit the selected task
- `d`: Delete the selected task
- `L`: View project logs
- `A`: List available agents
- `O`: Show OKRs
- `?`: Show available commands
- In popups (logs, OKRs, etc.), use `↑` / `↓` to scroll

### Manage tasks

- **Add a new task:**
  ```bash
  taskter task add -t "My new task" -d "A description for my task"
  ```
-  In the interactive board (`taskter board`), press `n` to add a task interactively. Enter the title, press `Enter`, then provide the description and press `Enter` again.

- **Edit a task:** Press `u` while the task is selected in the board to update its title and description.

- **Delete a task:** Press `d` while the task is selected in the board.

- **List all tasks:**
  ```bash
  taskter task list
  ```

- **Mark a task as done:**
  ```bash
  taskter task complete --id <task_id>
  ```
- **Add a comment to a task:**
  ```bash
  taskter task comment --task-id <task_id> --comment "Your note"
  ```

### Project information

- **Show project description:**
  ```bash
  taskter show description
  ```

- **Show project OKRs:**
  ```bash
  taskter okrs list
  ```

- **Show operation logs:**
  ```bash
  taskter logs list
  ```

### Manage OKRs

- **Add a new OKR:**
  ```bash
  taskter okrs add -o "My objective" -k "Key result 1" "Key result 2"
  ```

### Manage logs

- **Add a log entry:**
  ```bash
  taskter logs add "This is a log message"
  ```

### Agents

Taskter now supports LLM-based agents that can be assigned to tasks. These agents can execute tasks using a mocked Gemini API for tool-calling.

- **Add a new agent:**
  ```bash
  taskter agent add --prompt "You are a helpful assistant." --tools "email" "calendar" --model "gemini-pro"
  ```
  The `--tools` option accepts either paths to JSON files describing a tool or
  the name of a built-in tool. Built-ins live under the `tools/` directory of
  the repository. For example `email` resolves to `tools/send_email.json`.
  Other built-ins include `taskter_task`, `taskter_agent`, `taskter_okrs`,
  `taskter_tools`, `get_description`, `run_bash`, `run_python`, `file_ops`,
  `send_email`, and `web_search`.
  The `taskter_*` tools wrap the corresponding CLI subcommands. Example:
  ```json
  {"tool": "taskter_task", "args": {"args": ["list"]}}
  ```

- **Assign an agent to a task:**
  ```bash
  taskter task assign --task-id 1 --agent-id 1
  ```
- **Unassign an agent from a task:**
  ```bash
  taskter task unassign --task-id 1
  ```

- **Execute a task with an agent:**
  ```bash
  taskter task execute --task-id 1
  ```
- **List available agents:**
  ```bash
  taskter agent list
  ```
- **List available tools:**
  ```bash
  taskter tools list
  ```
- **Delete an agent:**
  ```bash
  taskter agent remove --id 1
  ```

When a task is executed, the agent will attempt to perform the task. If successful, the task is marked as "Done". If it fails, the task is moved back to "To Do", unassigned, and a comment from the agent is added.

In the interactive board (`taskter board`), tasks assigned to an agent will be marked with a `*`. You can view the assigned agent ID and any comments by selecting the task and pressing `Enter`.

### Email configuration

Agent email tools read credentials from `.taskter/email_config.json`. Place this
file inside the board directory (next to `board.json` and `agents.json`). All
agents share the same configuration. The currently recognised keys are:

```json
{
  "smtp_server": "smtp.example.com",
  "smtp_port": 587,
  "username": "user@example.com",
  "password": "secret",
  "imap_server": "imap.example.com",  // optional
  "imap_port": 993                   // optional
}
```

Only the SMTP fields are used by the built-in `send_email` tool today. There are
no default values, so you must supply valid server details. If the file is
missing the tool returns `Email configuration not found`. When the application
runs without a `GEMINI_API_KEY` the email tool is skipped entirely, which keeps
tests working even without credentials.


### Gemini API key

Agent execution uses the Gemini API. Provide the `GEMINI_API_KEY` environment
variable when you want agents to call the real service. If the variable is
absent or empty, Taskter falls back to an offline mode where only built-in
tools are executed.

```bash
export GEMINI_API_KEY=your_key_here
```

See the book's **Configuration** chapter for a detailed explanation of this
variable and the email configuration file.

### Example project

If you want to see Taskter in action without manually creating data, run the
provided helper script:

```bash
./scripts/setup_example_project.sh
```

The script removes any existing `.taskter` directory, creates a new board with a
few example tasks, sets a project description, defines OKRs and adds an agent
using the built-in email tool. Once it finishes you can inspect the board with
`taskter task list` or launch the TUI via `taskter board`.

### Minimal agent roster

When starting a new autonomous project you can automatically create a baseline
set of agents:

```bash
./scripts/setup_agent_roster.sh
```

The script expects an initialised `.taskter` directory and adds agents for OKR
management, task planning, recruitment, dispatching, execution, review,
progress monitoring and archiving. All of them use the `gemini-2.5-pro` model.


## Development

Run the included helper script before committing changes to ensure the code is
formatted, linted and all tests pass:

```bash
./scripts/precommit.sh
```

To automatically format the project and apply Clippy suggestions, you can run:

```bash
./scripts/fix_lints.sh
```

If you want Git to run it automatically, create a pre-commit hook:

```bash
ln -s ../../scripts/precommit.sh .git/hooks/pre-commit
```

## Documentation

Rendered documentation is available on GitHub Pages: <https://tomatyss.github.io/taskter/>.

To contribute to the book, edit the Markdown files under `docs/src/` and open a pull request. The `Deploy Docs` workflow will rebuild the book and publish it automatically when changes land on `main`.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.
