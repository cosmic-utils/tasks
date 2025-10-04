use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use thiserror::Error;
use std::io::{self, Write};

use crate::storage::{LocalStorage, models::{List, Task, Status}};
use super::Commands;
use super::output::{OutputWriter, ListOutput, TaskOutput, TaskDetailOutput};

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Temporary failure: {0}")]
    TemporaryFailure(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
}

pub async fn execute_command(
    command: Commands,
    writer: &mut OutputWriter,
) -> Result<(), CliError> {
    // Initialize storage
    let mut storage = LocalStorage::new("com.github.digit1024.ms-todo-app")
        .map_err(|e| CliError::AuthenticationError(e.to_string()))?;

    match command {
        Commands::Lists { include_virtual } => cmd_lists(&mut storage, include_virtual, writer).await,
        Commands::Tasks { list_id, filter, today, include_finished } => {
            cmd_tasks(&mut storage, &list_id, filter.as_deref(), today, include_finished, writer).await
        }
        Commands::Task { task_id } => cmd_task(&mut storage, &task_id, writer).await,
        Commands::Create { list, title, due_date, reminder, note } => {
            cmd_create(&mut storage, &list, &title, due_date.as_deref(), reminder.as_deref(), note.as_deref(), writer).await
        }
        Commands::Update { task_id, title, due_date, reminder, note, status } => {
            cmd_update(&mut storage, &task_id, title.as_deref(), due_date.as_deref(), reminder.as_deref(), note.as_deref(), status.as_deref(), writer).await
        }
        Commands::Delete { task_id, yes } => {
            cmd_delete(&mut storage, &task_id, yes, writer).await
        }
    }
}

async fn cmd_lists(
    storage: &mut LocalStorage,
    include_virtual: bool,
    writer: &mut OutputWriter,
) -> Result<(), CliError> {
    let lists = storage
        .lists()
        .await
        .map_err(|e| CliError::InternalError(e.to_string()))?;

    let filtered_lists: Vec<_> = lists
        .iter()
        .filter(|list| include_virtual || !list.is_virtual)
        .map(|list| ListOutput {
            id: list.id.clone(),
            name: list.name.clone(),
            task_count: list.number_of_tasks,
            is_virtual: list.is_virtual,
        })
        .collect();

    if filtered_lists.is_empty() {
        writer.success_message("No lists found");
    } else {
        for list in &filtered_lists {
            writer.success(list);
        }
    }

    Ok(())
}

async fn cmd_tasks(
    storage: &mut LocalStorage,
    list_id: &str,
    filter: Option<&str>,
    today: bool,
    include_finished: bool,
    writer: &mut OutputWriter,
) -> Result<(), CliError> {
    // Find the list
    let lists = storage
        .lists()
        .await
        .map_err(|e| CliError::InternalError(e.to_string()))?;

    let list = lists
        .iter()
        .find(|l| l.id == list_id)
        .ok_or_else(|| CliError::NotFound(format!("List with id '{}' not found", list_id)))?;

    // Get tasks for the list
    let tasks = storage
        .tasks(list)
        .await
        .map_err(|e| CliError::InternalError(e.to_string()))?;

    // Apply filters
    let filtered_tasks: Vec<_> = tasks
        .iter()
        .filter(|task| {
            // Filter by completion status
            if !include_finished && task.status == Status::Completed {
                return false;
            }

            // Filter by substring
            if let Some(substr) = filter {
                if !task.title.to_lowercase().contains(&substr.to_lowercase()) {
                    return false;
                }
            }

            // Filter by today
            if today {
                let now = Utc::now().date_naive();
                let is_today = task.due_date.map(|d| d.date_naive() == now).unwrap_or(false)
                    || task.reminder_date.map(|d| d.date_naive() == now).unwrap_or(false);
                if !is_today {
                    return false;
                }
            }

            true
        })
        .map(|task| TaskOutput {
            id: task.id.clone(),
            title: task.title.clone(),
            status: format!("{:?}", task.status),
            due_date: task.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
            reminder_date: task.reminder_date.map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            notes: task.notes.clone(),
            tags: task.tags.clone(),
            list_id: task.list_id.clone(),
        })
        .collect();

    if filtered_tasks.is_empty() {
        writer.success_message("No tasks found matching criteria");
    } else {
        for task in &filtered_tasks {
            writer.success(task);
        }
    }

    Ok(())
}

async fn cmd_task(
    storage: &mut LocalStorage,
    task_id: &str,
    writer: &mut OutputWriter,
) -> Result<(), CliError> {
    // Get all lists and search for the task
    let lists = storage
        .lists()
        .await
        .map_err(|e| CliError::InternalError(e.to_string()))?;

    for list in &lists {
        let tasks = storage
            .tasks(list)
            .await
            .map_err(|e| CliError::InternalError(e.to_string()))?;

        if let Some(task) = tasks.iter().find(|t| t.id == task_id) {
            let output = TaskDetailOutput {
                id: task.id.clone(),
                title: task.title.clone(),
                status: format!("{:?}", task.status),
                priority: format!("{:?}", task.priority),
                due_date: task.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
                reminder_date: task.reminder_date.map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
                completion_date: task.completion_date.map(|d| d.format("%Y-%m-%d").to_string()),
                notes: task.notes.clone(),
                tags: task.tags.clone(),
                list_id: task.list_id.clone(),
                created_date: task.created_date_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                modified_date: task.last_modified_date_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            writer.success(&output);
            return Ok(());
        }
    }

    Err(CliError::NotFound(format!("Task with id '{}' not found", task_id)))
}

async fn cmd_create(
    storage: &mut LocalStorage,
    list_id: &str,
    title: &str,
    due_date: Option<&str>,
    reminder: Option<&str>,
    note: Option<&str>,
    writer: &mut OutputWriter,
) -> Result<(), CliError> {
    // Validate list exists
    let lists = storage
        .lists()
        .await
        .map_err(|e| CliError::InternalError(e.to_string()))?;

    let _list = lists
        .iter()
        .find(|l| l.id == list_id)
        .ok_or_else(|| CliError::NotFound(format!("List with id '{}' not found", list_id)))?;

    // Create task
    let mut task = Task::new(title.to_string(), Some(list_id.to_string()));

    // Parse and set due date
    if let Some(date_str) = due_date {
        task.due_date = Some(parse_date(date_str)?);
    }

    // Parse and set reminder
    if let Some(reminder_str) = reminder {
        task.reminder_date = Some(parse_datetime(reminder_str)?);
    }

    // Set notes
    if let Some(note_text) = note {
        task.notes = note_text.to_string();
    }

    // Create via storage
    let created_task = storage
        .create_task(&task)
        .await
        .map_err(|e| CliError::InternalError(e.to_string()))?;

    writer.success_message(&format!("Task created with ID: {}", created_task.id));
    Ok(())
}

async fn cmd_update(
    storage: &mut LocalStorage,
    task_id: &str,
    title: Option<&str>,
    due_date: Option<&str>,
    reminder: Option<&str>,
    note: Option<&str>,
    status: Option<&str>,
    writer: &mut OutputWriter,
) -> Result<(), CliError> {
    // Find the task
    let lists = storage
        .lists()
        .await
        .map_err(|e| CliError::InternalError(e.to_string()))?;

    let mut found_task: Option<Task> = None;
    for list in &lists {
        let tasks = storage
            .tasks(list)
            .await
            .map_err(|e| CliError::InternalError(e.to_string()))?;

        if let Some(task) = tasks.iter().find(|t| t.id == task_id) {
            found_task = Some(task.clone());
            break;
        }
    }

    let mut task = found_task
        .ok_or_else(|| CliError::NotFound(format!("Task with id '{}' not found", task_id)))?;

    // Update fields
    if let Some(new_title) = title {
        task.title = new_title.to_string();
    }

    if let Some(date_str) = due_date {
        task.due_date = Some(parse_date(date_str)?);
    }

    if let Some(reminder_str) = reminder {
        task.reminder_date = Some(parse_datetime(reminder_str)?);
    }

    if let Some(note_text) = note {
        task.notes = note_text.to_string();
    }

    if let Some(status_str) = status {
        task.status = match status_str.to_lowercase().as_str() {
            "pending" | "notstarted" => Status::NotStarted,
            "finished" | "completed" => Status::Completed,
            _ => return Err(CliError::InvalidArgument(
                format!("Invalid status: '{}'. Use 'pending' or 'finished'", status_str)
            )),
        };
    }

    // Update via storage
    storage
        .update_task(&task)
        .await
        .map_err(|e| CliError::InternalError(e.to_string()))?;

    writer.success_message(&format!("Task '{}' updated successfully", task_id));
    Ok(())
}

async fn cmd_delete(
    storage: &mut LocalStorage,
    task_id: &str,
    skip_confirm: bool,
    writer: &mut OutputWriter,
) -> Result<(), CliError> {
    // Find the task
    let lists = storage
        .lists()
        .await
        .map_err(|e| CliError::InternalError(e.to_string()))?;

    let mut found_task: Option<Task> = None;
    for list in &lists {
        let tasks = storage
            .tasks(list)
            .await
            .map_err(|e| CliError::InternalError(e.to_string()))?;

        if let Some(task) = tasks.iter().find(|t| t.id == task_id) {
            found_task = Some(task.clone());
            break;
        }
    }

    let task = found_task
        .ok_or_else(|| CliError::NotFound(format!("Task with id '{}' not found", task_id)))?;

    // Confirm deletion
    if !skip_confirm {
        print!("Delete task '{}' (y/N)? ", task.title);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        if !input.trim().eq_ignore_ascii_case("y") {
            writer.success_message("Deletion cancelled");
            return Ok(());
        }
    }

    // Delete via storage
    storage
        .delete_task(&task)
        .await
        .map_err(|e| CliError::InternalError(e.to_string()))?;

    writer.success_message(&format!("Task '{}' deleted successfully", task_id));
    Ok(())
}

// Helper functions for date parsing
fn parse_date(date_str: &str) -> Result<DateTime<Utc>, CliError> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| CliError::InvalidArgument(
            format!("Invalid date format: '{}'. Expected YYYY-MM-DD", date_str)
        ))
        .map(|date| {
            let datetime = date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
            Utc.from_utc_datetime(&datetime)
        })
}

fn parse_datetime(datetime_str: &str) -> Result<DateTime<Utc>, CliError> {
    DateTime::parse_from_rfc3339(datetime_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| CliError::InvalidArgument(
            format!("Invalid datetime format: '{}'. Expected YYYY-MM-DDTHH:MM:SSZ", datetime_str)
        ))
}

