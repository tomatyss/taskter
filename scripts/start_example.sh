#!/bin/sh
# Set up an example Taskter board with sample data.

set -e

# Build the project so we have the taskter binary available.
cargo build --release

BIN="target/release/taskter"

# Create a fresh example workspace
rm -rf example
mkdir example
cd example

# Initialise a new board
../$BIN init

# Add a project description
../$BIN description "Example Taskter project demonstrating basic features."

# Add some tasks
../$BIN add -t "Set up CI pipeline" -d "Configure continuous integration"
../$BIN add -t "Write documentation" -d "Document how to use Taskter"
../$BIN add -t "Prepare first release" -d "Package binaries and publish"

# Add an OKR
../$BIN add-okr -o "Launch MVP" -k "Complete core features" "Publish docs"

# Prepare a tool description for the demo agent
cat > email_tool.json <<'JSON'
{
  "name": "send_email",
  "description": "Send an email to a recipient",
  "parameters": {
    "type": "object",
    "properties": {
      "to": {"type": "string"},
      "subject": {"type": "string"},
      "body": {"type": "string"}
    },
    "required": ["to", "subject", "body"]
  }
}
JSON

# Add the agent and assign it to the first task
../$BIN add-agent --prompt "You are a helpful email assistant" --tools email_tool.json --model "gemini-pro"
../$BIN assign --task-id 1 --agent-id 1

# Add a log entry
../$BIN log "Example project initialised"

echo "Example project created in $(pwd)"
