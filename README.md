# Taskter

Taskter is a terminal Kanban board CLI tool built with Rust.

## Features

- Kanban board with tasks (ToDo, InProgress, Done)
- Project description
- Operation logs
- OKRs (Objectives and Key Results)

## Build and Installation

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

Alternatively, you can use `cargo install`:
```bash
cargo install --path .
```
This will install the `taskter` executable in your Cargo bin directory (usually `~/.cargo/bin/`), which should be in your `PATH`.


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

### Manage tasks

- **Add a new task:**
  ```bash
  taskter add -t "My new task" -d "A description for my task"
  ```

- **List all tasks:**
  ```bash
  taskter list
  ```

- **Mark a task as done:**
  ```bash
  taskter done <task_id>
  ```

### Project information

- **Show project description:**
  ```bash
  taskter show description
  ```

- **Show project OKRs:**
  ```bash
  taskter show okrs
  ```

- **Show operation logs:**
  ```bash
  taskter show logs
  ```

### Manage OKRs

- **Add a new OKR:**
  ```bash
  taskter add-okr -o "My objective" -k "Key result 1" "Key result 2"
  ```

### Manage logs

- **Add a log entry:**
  ```bash
  taskter log "This is a log message"
  ```
