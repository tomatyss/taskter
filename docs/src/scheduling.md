# Scheduling Agents

Taskter can run agents automatically based on cron expressions. A scheduler daemon reads the agent configuration and executes the assigned tasks at the defined times.

## Setting a Schedule

Use the `agent schedule set` command to assign a cron expression to an agent. The expression is parsed in the `America/New_York` timezone.

```bash
# Run every minute
taskter agent schedule set --id 1 --cron "0 * * * * *"
```

Pass `--once` to remove the schedule after the first run.

## Listing and Removing

List all scheduled agents with:

```bash
taskter agent schedule list
```

Remove a schedule:

```bash
taskter agent schedule remove --id 1
```

## Running the Scheduler

Start the scheduler loop with:

```bash
taskter scheduler run
```

The scheduler will execute agents at the configured times and update tasks just as if `task execute` was run manually.
When multiple tasks are assigned to the same agent, the scheduler now runs them
concurrently so long-running jobs don't block each other.
