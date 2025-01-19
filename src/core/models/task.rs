use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{priority::Priority, recurrence::Recurrence, status::Status};

#[derive(Clone, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Task {
    pub(crate) id: String,
    pub parent: String,
    pub title: String,
    pub favorite: bool,
    pub today: bool,
    pub status: Status,
    pub priority: Priority,
    pub sub_tasks: Vec<Task>,
    pub tags: Vec<String>,
    pub notes: String,
    pub completion_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub reminder_date: Option<DateTime<Utc>>,
    pub recurrence: Recurrence,
    pub(crate) deletion_date: Option<DateTime<Utc>>,
    pub(crate) created_date_time: DateTime<Utc>,
    pub(crate) last_modified_date_time: DateTime<Utc>,
}

impl Task {
    pub fn new(title: String, parent: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            parent,
            title,
            favorite: false,
            today: false,
            status: Status::NotStarted,
            priority: Priority::Low,
            sub_tasks: vec![],
            tags: vec![],
            notes: String::new(),
            completion_date: None,
            deletion_date: None,
            due_date: None,
            reminder_date: None,
            recurrence: Default::default(),
            created_date_time: now,
            last_modified_date_time: now,
        }
    }
}
