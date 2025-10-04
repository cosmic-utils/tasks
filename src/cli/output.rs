use clap::ValueEnum;
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

pub struct OutputWriter {
    format: OutputFormat,
}

impl OutputWriter {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    pub fn success<T: Serialize + Display>(&self, data: &T) {
        match self.format {
            OutputFormat::Json => {
                let output = json!({
                    "success": true,
                    "data": data
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            }
            OutputFormat::Text => {
                println!("{}", data);
            }
        }
    }

    pub fn success_message(&self, message: &str) {
        match self.format {
            OutputFormat::Json => {
                let output = json!({
                    "success": true,
                    "message": message
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            }
            OutputFormat::Text => {
                println!("✅ {}", message);
            }
        }
    }

    pub fn error(&self, error: &dyn std::error::Error) {
        match self.format {
            OutputFormat::Json => {
                let output = json!({
                    "success": false,
                    "error": {
                        "message": error.to_string()
                    }
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            }
            OutputFormat::Text => {
                eprintln!("❌ Error: {}", error);
            }
        }
    }
}

// Helper structures for JSON serialization
#[derive(Serialize, Deserialize, Debug)]
pub struct ListOutput {
    pub id: String,
    pub name: String,
    pub task_count: u32,
    pub is_virtual: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskOutput {
    pub id: String,
    pub title: String,
    pub status: String,
    pub due_date: Option<String>,
    pub reminder_date: Option<String>,
    pub notes: String,
    pub tags: Vec<String>,
    pub list_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskDetailOutput {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub due_date: Option<String>,
    pub reminder_date: Option<String>,
    pub completion_date: Option<String>,
    pub notes: String,
    pub tags: Vec<String>,
    pub list_id: Option<String>,
    pub created_date: String,
    pub modified_date: String,
}

impl Display for ListOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let virtual_indicator = if self.is_virtual { " [virtual]" } else { "" };
        write!(
            f,
            "{} - {} ({} tasks){}",
            self.id, self.name, self.task_count, virtual_indicator
        )
    }
}

impl Display for TaskOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_icon = match self.status.as_str() {
            "Completed" => "✓",
            _ => "○",
        };
        
        let due_info = if let Some(due) = &self.due_date {
            format!(" (due: {})", due)
        } else {
            String::new()
        };

        write!(f, "{} {} - {}{}", status_icon, self.id, self.title, due_info)
    }
}

impl Display for TaskDetailOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Task: {}", self.title)?;
        writeln!(f, "ID: {}", self.id)?;
        writeln!(f, "Status: {}", self.status)?;
        writeln!(f, "Priority: {}", self.priority)?;
        
        if let Some(due) = &self.due_date {
            writeln!(f, "Due Date: {}", due)?;
        }
        
        if let Some(reminder) = &self.reminder_date {
            writeln!(f, "Reminder: {}", reminder)?;
        }
        
        if let Some(completed) = &self.completion_date {
            writeln!(f, "Completed: {}", completed)?;
        }
        
        if !self.tags.is_empty() {
            writeln!(f, "Tags: {}", self.tags.join(", "))?;
        }
        
        if !self.notes.is_empty() {
            writeln!(f, "\nNotes:\n{}", self.notes)?;
        }
        
        writeln!(f, "\nCreated: {}", self.created_date)?;
        writeln!(f, "Modified: {}", self.modified_date)?;
        
        Ok(())
    }
}

