use std::path::PathBuf;

use chrono::{DateTime, Utc};
use cosmic::Application;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::TasksApp;

use super::{Priority, Recurrence, Status};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    
    pub title: String,
    pub favorite: bool,
    pub today: bool,
    pub status: Status,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub notes: String,
    pub completion_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub reminder_date: Option<DateTime<Utc>>,
    pub recurrence: Recurrence,
    #[serde(default)]
    pub expanded: bool,
    
    pub deletion_date: Option<DateTime<Utc>>,
    pub created_date_time: DateTime<Utc>,
    pub last_modified_date_time: DateTime<Utc>,
    pub  list_id: Option<String>
}

impl Task {
    pub fn new(title: String, list_id : Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            
            title,
            favorite: false,
            today: false,
            status: Status::NotStarted,
            priority: Priority::Low,
            tags: vec![],
            notes: String::new(),
            completion_date: None,
            due_date: None,
            reminder_date: None,
            recurrence: Default::default(),
            expanded: false,
            
            deletion_date: None,
            created_date_time: now,
            last_modified_date_time: now,
            list_id: list_id,
        }
    }

    

  
}
