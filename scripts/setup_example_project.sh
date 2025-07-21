#!/bin/sh
set -e

# Start from a clean board
rm -rf .taskter

# Initialize project board
taskter init

# Project description
taskter description "Example project demonstrating Taskter features."

# Add example tasks
taskter task add -t "Write documentation" -d "Describe how to use Taskter."
taskter task add -t "Plan v1 release" -d "Define scope and timeline."
taskter task add -t "Send status email" -d "Notify stakeholders about progress."

# Define OKRs
taskter okrs add -o "Deliver MVP" -k "Ship v1" "Collect user feedback"

# Create an agent with the built-in email tool
taskter agent add --prompt "You are a helpful assistant." --tools email --model "gemini-pro"

# Assign agent to the email task
taskter task assign --task-id 3 --agent-id 1

cat <<MSG
Example board initialized. Run 'taskter task list' or 'taskter board' to explore.
MSG
