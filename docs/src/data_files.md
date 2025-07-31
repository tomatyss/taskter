# Data Files

Taskter keeps all project state inside a `.taskter` directory. This folder is created when you run `taskter init`.
The following files are stored there and are automatically updated by Taskter.

## board.json

Holds the Kanban board in JSON format. The file contains all tasks with their status, descriptions and assigned agent. It is rewritten whenever you add, edit or complete tasks from the CLI or TUI.

## agents.json

Stores the list of agents. Each agent entry records the system prompt, available tools, model and optional schedule. The file is created on demand and modified by the various `agent` subcommands.

## okrs.json

Contains your objectives and key results. Commands under `taskter okrs` load and save this file.

## logs.log

Plain text log with timestamps. New lines are appended when you run `logs add` or when agents execute tasks.

## description.md

Markdown file describing the project. `taskter init` creates a placeholder that you can edit manually or through the TUI.

## email_config.json

Optional email credentials used by the `send_email` tool. Place it in the directory if agents need to send messages. The exact keys are documented in the [Configuration](configuration.html) chapter.
