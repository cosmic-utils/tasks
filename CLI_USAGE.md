# msToDO CLI Usage Guide

## Overview

The msToDO application supports both a GUI and a comprehensive CLI for automation and scripting. The application automatically detects whether to run in CLI or GUI mode:

- **GUI Mode**: Run without any arguments → `./ms-todo-app`
- **CLI Mode**: Run with any command → `./ms-todo-app <command>`

## Installation

Build the application:
```bash
cargo build --release
```

The binary will be at `./target/release/ms-todo-app`

## Global Options

- `-o, --output <FORMAT>`: Output format (text or json)
  - `text` - Human-readable output (default)
  - `json` - Machine-readable JSON output

## Commands

### 1. List All Lists

Show all available todo lists:

```bash
# Show regular lists only
mstodo lists

# Include virtual lists (Today, Planned, All)
mstodo lists --include-virtual

# JSON output for scripting
mstodo lists -o json
```

**Example output (text)**:
```
AAMkAGRm...ZjQ - Tasks (5 tasks)
AAMkAGRm...YjI - Shopping (2 tasks)
virtual_MyDay - My Day (3 tasks) [virtual]
```

**Example output (json)**:
```json
{
  "success": true,
  "data": {
    "id": "AAMkAGRm...ZjQ",
    "name": "Tasks",
    "task_count": 5,
    "is_virtual": false
  }
}
```

### 2. List Tasks

Show tasks within a specific list:

```bash
# Show all unfinished tasks
mstodo tasks <list-id>

# Filter by substring in title
mstodo tasks <list-id> --filter "meeting"

# Show only today's tasks
mstodo tasks <list-id> --today

# Include finished tasks
mstodo tasks <list-id> --include-finished

# Combined filters
mstodo tasks <list-id> --filter "report" --today --include-finished

# JSON output
mstodo tasks <list-id> -o json
```

**Example output (text)**:
```
○ AAMkADRl...xMw - Buy groceries (due: 2025-10-05)
○ AAMkADRl...yNg - Finish report (due: 2025-10-04)
✓ AAMkADRl...zOh - Call client
```

### 3. Show Task Details

Display detailed information about a specific task:

```bash
# Text format
mstodo task <task-id>

# JSON format
mstodo task <task-id> -o json
```

**Example output (text)**:
```
Task: Finish quarterly report
ID: AAMkADRl...yNg
Status: NotStarted
Priority: High
Due Date: 2025-10-10
Reminder: 2025-10-10T09:00:00Z
Tags: work, urgent

Notes:
Include Q3 metrics and projections for Q4

Created: 2025-10-01 14:30:00
Modified: 2025-10-04 10:15:00
```

### 4. Create Task

Create a new task in a list:

```bash
# Minimal task
mstodo create --list <list-id> --title "Task title"

# Task with due date
mstodo create --list <list-id> --title "Submit report" --due-date "2025-10-15"

# Task with due date and reminder
mstodo create \
  --list <list-id> \
  --title "Team meeting" \
  --due-date "2025-10-10" \
  --reminder "2025-10-10T09:00:00Z"

# Task with notes
mstodo create \
  --list <list-id> \
  --title "Research project" \
  --due-date "2025-10-20" \
  --note "Focus on competitive analysis and market trends"
```

**Date Formats**:
- Due date: `YYYY-MM-DD` (e.g., `2025-10-15`)
- Reminder: `YYYY-MM-DDTHH:MM:SSZ` (e.g., `2025-10-15T14:30:00Z`)

**Output**:
```
✅ Task created with ID: AAMkADRl...new
```

### 5. Update Task

Update an existing task:

```bash
# Update title
mstodo update <task-id> --title "New title"

# Update due date
mstodo update <task-id> --due-date "2025-10-20"

# Update reminder
mstodo update <task-id> --reminder "2025-10-20T10:00:00Z"

# Update notes
mstodo update <task-id> --note "Updated description"

# Mark as completed
mstodo update <task-id> --status finished

# Mark as pending
mstodo update <task-id> --status pending

# Update multiple fields
mstodo update <task-id> \
  --title "Updated title" \
  --due-date "2025-10-25" \
  --status finished
```

**Status values**: `pending`, `notstarted`, `finished`, `completed`

**Output**:
```
✅ Task 'AAMkADRl...abc' updated successfully
```

### 6. Delete Task

Delete a task:

```bash
# With confirmation prompt
mstodo delete <task-id>

# Skip confirmation
mstodo delete <task-id> --yes
```

**Output**:
```
Delete task 'Buy groceries' (y/N)? y
✅ Task 'AAMkADRl...abc' deleted successfully
```

## Exit Codes

Following POSIX/sysexits conventions:

| Code | Meaning | Example |
|------|---------|---------|
| 0 | Success | Operation completed successfully |
| 64 | Invalid arguments | Invalid date format, unknown status |
| 65 | Data/validation error | Invalid task data |
| 66 | Not found | List or task doesn't exist |
| 70 | Internal error | Database error, API failure |
| 75 | Temporary failure | Network issue, retry later |
| 77 | Authentication error | No valid tokens, auth failed |

## Scripting Examples

### Daily Task Report via Email

```bash
#!/bin/bash
# Send today's tasks via email

mstodo tasks inbox --today -o json \
  | jq -r '.items[] | "- \(.title) (due: \(.due_date))"' \
  | mail -s "Today's tasks" me@domain.com
```

### Bulk Task Creation

```bash
#!/bin/bash
# Create multiple tasks from a file

LIST_ID="AAMkAGRm...ZjQ"

while IFS='|' read -r title due_date; do
  mstodo create --list "$LIST_ID" --title "$title" --due-date "$due_date"
  echo "Created: $title"
done < tasks.txt
```

Format of `tasks.txt`:
```
Buy groceries|2025-10-05
Finish report|2025-10-10
Team meeting|2025-10-15
```

### Cron Job - Daily Reminder

```bash
# Add to crontab: Run at 8 AM daily
0 8 * * * /path/to/ms-todo-app tasks inbox --today -o json | \
  /path/to/notify-script.sh
```

### JSON Processing with jq

```bash
# Get all task IDs from a list
mstodo tasks <list-id> -o json | jq -r '.data.id'

# Count tasks by status
mstodo tasks <list-id> --include-finished -o json | \
  jq -r '.data.status' | sort | uniq -c

# Extract tasks with high priority
mstodo tasks <list-id> -o json | \
  jq '.data | select(.priority == "High")'
```

### Find Overdue Tasks

```bash
#!/bin/bash
# Find tasks with due dates in the past

today=$(date +%Y-%m-%d)
mstodo lists -o json | jq -r '.data.id' | while read list_id; do
  mstodo tasks "$list_id" -o json | \
    jq -r --arg today "$today" \
    '.data | select(.due_date != null and .due_date < $today) | 
    "\(.title) - Overdue: \(.due_date)"'
done
```

## Error Handling in Scripts

### Bash Example

```bash
#!/bin/bash

if mstodo create --list "$LIST_ID" --title "Test task" 2>/dev/null; then
  echo "✅ Task created"
else
  exit_code=$?
  case $exit_code in
    64) echo "❌ Invalid arguments" ;;
    66) echo "❌ List not found" ;;
    70) echo "❌ Internal error" ;;
    77) echo "❌ Authentication failed" ;;
    *) echo "❌ Unknown error: $exit_code" ;;
  esac
  exit $exit_code
fi
```

### JSON Error Format

When using `-o json`, errors are returned as:

```json
{
  "success": false,
  "error": {
    "message": "List with id 'invalid-id' not found"
  }
}
```

## Tips

1. **Get List IDs**: Run `mstodo lists -o json` to get list IDs for use in other commands

2. **Automation**: Use JSON output (`-o json`) with tools like `jq` for scripting

3. **Confirmation**: Use `--yes` flag with `delete` in automation scripts to skip prompts

4. **Date Formats**: 
   - Due dates: Simple `YYYY-MM-DD`
   - Reminders: Full ISO8601 with timezone `YYYY-MM-DDTHH:MM:SSZ`

5. **Virtual Lists**: Use `--include-virtual` to see system lists like "My Day" and "Planned"

6. **Filtering**: Combine `--filter`, `--today`, and `--include-finished` for precise queries

## GUI Mode

To launch the graphical interface, simply run without arguments:

```bash
./ms-todo-app
```

The GUI provides the full visual interface for managing tasks, while the CLI is optimized for automation and scripting.

