# TUI Guide

Taskter's interactive Terminal User Interface (TUI) provides a visual way to manage your Kanban board. To launch it, run:

```bash
taskter board
```

## Keybindings

The TUI is controlled with keyboard shortcuts. Here is a list of the available keybindings:

| Key(s)              | Action                               |
| ------------------- | ------------------------------------ |
| `q`                 | Quit the application                 |
| `←` / `→` or `Tab`  | Navigate between columns             |
| `↑` / `↓`           | Navigate between tasks in a column   |
| `h` / `l`           | Move a selected task to the next or previous column |
| `n`                 | Create a new task                    |
| `u`                 | Edit the selected task               |
| `d`                 | Delete the selected task             |
| `a`                 | Assign an agent to the selected task |
| `c`                 | Add a comment to the selected task   |
| `L`                 | View project logs                    |
| `A`                 | List available agents                |
| `O`                 | Show project OKRs                    |

When a task is selected, you can press `Enter` to view its details, including the full description, any comments, and the assigned agent ID.
