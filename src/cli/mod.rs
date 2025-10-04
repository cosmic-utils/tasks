mod commands;
mod output;

use clap::{Parser, Subcommand};
use output::{OutputFormat, OutputWriter};

/// Microsoft Todo CLI - Automation-friendly task management
#[derive(Parser, Debug)]
#[command(name = "ms-todo-app")]
#[command(author, version)]
#[command(about = "Microsoft Todo CLI - Automation-friendly task management")]
#[command(long_about = r#"MS TODO App - Command-line interface for Microsoft Todo

This tool provides full command-line access to your Microsoft Todo tasks.
Perfect for automation, scripting, and integration with other tools.

MODES:
  - GUI Mode: Run without arguments to launch the graphical interface
  - CLI Mode: Run with any command for command-line operations

EXAMPLES:
  # Launch GUI
  ms-todo-app

  # List all lists including virtual ones
  ms-todo-app lists --include-virtual

  # Show today's tasks as JSON
  ms-todo-app tasks <list-id> --today -o json

  # Create a task with due date
  ms-todo-app create --list <list-id> --title "Finish report" --due-date "2025-12-31"

  # Mark task as completed
  ms-todo-app update <task-id> --status finished

  # Delete without confirmation
  ms-todo-app delete <task-id> --yes

EXIT CODES:
  0   Success
  64  Invalid arguments
  65  Validation error
  66  Not found (list/task)
  70  Internal error
  75  Temporary failure
  77  Authentication error

For detailed documentation, see CLI_USAGE.md
"#)]
pub struct Cli {
    /// Output format
    #[arg(short, long, value_enum, default_value = "text", global = true)]
    output: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all todo lists
    /// 
    /// Shows all your Microsoft Todo lists with task counts.
    /// Use --include-virtual to also see system lists like "My Day" and "Planned".
    /// 
    /// EXAMPLES:
    ///   ms-todo-app lists
    ///   ms-todo-app lists --include-virtual
    ///   ms-todo-app lists -o json | jq -r '.data.id'
    Lists {
        /// Include virtual lists (Today, Overdue, Completed)
        #[arg(long)]
        include_virtual: bool,
    },

    /// List tasks in a specific list
    /// 
    /// Query tasks with flexible filtering options.
    /// By default shows only unfinished tasks.
    /// 
    /// EXAMPLES:
    ///   ms-todo-app tasks <list-id>
    ///   ms-todo-app tasks <list-id> --today
    ///   ms-todo-app tasks <list-id> --filter "meeting" --include-finished
    ///   ms-todo-app tasks <list-id> --today -o json | jq '.data.title'
    Tasks {
        /// List ID to query
        list_id: String,

        /// Filter tasks by substring in title
        #[arg(long)]
        filter: Option<String>,

        /// Show only tasks due or reminded today
        #[arg(long)]
        today: bool,

        /// Include finished tasks
        #[arg(long)]
        include_finished: bool,
    },

    /// Show detailed information about a specific task
    /// 
    /// Display complete task details including title, status, dates,
    /// priority, notes, and metadata.
    /// 
    /// EXAMPLES:
    ///   ms-todo-app task <task-id>
    ///   ms-todo-app task <task-id> -o json
    Task {
        /// Task ID to display
        task_id: String,
    },

    /// Create a new task
    /// 
    /// Create a new task with optional due date, reminder, and notes.
    /// Returns the created task ID on success.
    /// 
    /// DATE FORMATS:
    ///   --due-date: YYYY-MM-DD (e.g., 2025-12-31)
    ///   --reminder: YYYY-MM-DDTHH:MM:SSZ (e.g., 2025-12-31T09:00:00Z)
    /// 
    /// EXAMPLES:
    ///   ms-todo-app create --list <list-id> --title "Buy groceries"
    ///   ms-todo-app create --list <list-id> --title "Meeting" --due-date "2025-10-15"
    ///   ms-todo-app create --list <list-id> --title "Review PR" \
    ///     --due-date "2025-10-20" --reminder "2025-10-20T09:00:00Z" \
    ///     --note "Check security implications"
    Create {
        /// List ID where the task should be created
        #[arg(long)]
        list: String,

        /// Task title
        #[arg(long)]
        title: String,

        /// Due date (YYYY-MM-DD)
        #[arg(long)]
        due_date: Option<String>,

        /// Reminder date and time (YYYY-MM-DDTHH:MM:SSZ)
        #[arg(long)]
        reminder: Option<String>,

        /// Task notes
        #[arg(long)]
        note: Option<String>,
    },

    /// Update an existing task
    /// 
    /// Modify one or more fields of an existing task.
    /// Only specified fields will be updated.
    /// 
    /// STATUS VALUES:
    ///   pending, notstarted - Mark as not started
    ///   finished, completed - Mark as completed
    /// 
    /// EXAMPLES:
    ///   ms-todo-app update <task-id> --title "Updated title"
    ///   ms-todo-app update <task-id> --status finished
    ///   ms-todo-app update <task-id> --due-date "2025-12-31" --note "Extended deadline"
    Update {
        /// Task ID to update
        task_id: String,

        /// New title
        #[arg(long)]
        title: Option<String>,

        /// New due date (YYYY-MM-DD)
        #[arg(long)]
        due_date: Option<String>,

        /// New reminder (YYYY-MM-DDTHH:MM:SSZ)
        #[arg(long)]
        reminder: Option<String>,

        /// New notes
        #[arg(long)]
        note: Option<String>,

        /// Task status
        #[arg(long)]
        status: Option<String>,
    },

    /// Delete a task
    /// 
    /// Permanently delete a task. By default, prompts for confirmation.
    /// Use --yes to skip confirmation (useful in scripts).
    /// 
    /// EXAMPLES:
    ///   ms-todo-app delete <task-id>
    ///   ms-todo-app delete <task-id> --yes
    Delete {
        /// Task ID to delete
        task_id: String,

        /// Skip confirmation
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut writer = OutputWriter::new(cli.output);

    match commands::execute_command(cli.command, &mut writer).await {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            writer.error(&e);
            // Exit with appropriate code based on error type
            std::process::exit(determine_exit_code(&e));
        }
    }
}

fn determine_exit_code(error: &commands::CliError) -> i32 {
    use commands::CliError;
    match error {
        CliError::InvalidArgument(_) => 64,
        CliError::ValidationError(_) => 65,
        CliError::NotFound(_) => 66,
        CliError::InternalError(_) => 70,
        CliError::TemporaryFailure(_) => 75,
        CliError::AuthenticationError(_) => 77,
    }
}

