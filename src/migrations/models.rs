use jiff::{civil::Date, Timestamp};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::model::{Priority, Recurrence, Status};

/// Old Task structure with nested sub-tasks
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    #[serde(default)]
    pub path: PathBuf,
    pub title: String,
    pub favorite: bool,
    pub today: bool,
    pub status: Status,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub notes: String,
    pub completion_date: Option<Timestamp>,
    pub due_date: Option<Date>,
    pub reminder_date: Option<Timestamp>,
    pub recurrence: Recurrence,
    #[serde(default)]
    pub expanded: bool,
    pub sub_tasks: Vec<Task>,
    pub deletion_date: Option<Timestamp>,
    pub created_date_time: Timestamp,
    pub last_modified_date_time: Timestamp,
}

/// Old List structure with file_path
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct List {
    pub id: String,
    #[serde(default)]
    pub file_path: PathBuf,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub hide_completed: bool,
}
