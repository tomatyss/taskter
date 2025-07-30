#!/bin/sh
set -e

if [ ! -d .taskter ]; then
    echo "Board not initialized. Run 'taskter init' first." >&2
    exit 1
fi

# Objective-Setter
taskter agent add \
    --prompt "Translate the project description into a single SMART objective and <= 3 key results." \
    --tools taskter_okrs get_description \
    --model "gemini-2.5-pro"

# Planner
taskter agent add \
    --prompt "For the current OKR, propose tasks that fully satisfy every KR." \
    --tools taskter_task taskter_okrs \
    --model "gemini-2.5-pro"

# Recruiter
taskter agent add \
    --prompt "Given a task description, decide whether a new agent profile is required; if yes, create it with the minimum tool set." \
    --tools taskter_agent taskter_tools \
    --model "gemini-2.5-pro"

# Dispatcher
taskter agent add \
    --prompt "Match tasks to agents whose tools and model are sufficient; otherwise request the Recruiter to create one." \
    --tools taskter_task taskter_agent \
    --model "gemini-2.5-pro"

# Worker-Bash
taskter agent add \
    --prompt "Execute only safe bash commands required to satisfy the task." \
    --tools run_bash taskter_task \
    --model "gemini-2.5-pro"

# Worker-Python
taskter agent add \
    --prompt "Execute only safe Python code required to satisfy the task." \
    --tools run_python taskter_task \
    --model "gemini-2.5-pro"

# Worker-Web
taskter agent add \
    --prompt "Perform web searches and summarise the results to complete the task." \
    --tools web_search taskter_task \
    --model "gemini-2.5-pro"

# Reviewer
taskter agent add \
    --prompt "Check output against the task description and relevant key result metrics." \
    --tools taskter_task taskter_okrs \
    --model "gemini-2.5-pro"

# Progress-Monitor
taskter agent add \
    --prompt "Every hour, evaluate KR values; if all satisfied, close project and notify stakeholders." \
    --tools taskter_okrs taskter_task send_email \
    --model "gemini-2.5-pro"

# Closer
taskter agent add \
    --prompt "Create a timestamped archive of the board, then remove all schedules and deactivate agents." \
    --tools taskter_task taskter_agent taskter_okrs run_bash \
    --model "gemini-2.5-pro"

