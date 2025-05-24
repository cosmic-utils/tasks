use std::path::PathBuf;

use chrono::{DateTime, Utc};
use cosmic::Application;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::Tasks;

use super::{Priority, Recurrence, Status};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub path: PathBuf,
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
    pub expanded: bool,
    #[serde(skip)]
    pub sub_tasks: Vec<Task>,
    pub deletion_date: Option<DateTime<Utc>>,
    pub created_date_time: DateTime<Utc>,
    pub last_modified_date_time: DateTime<Utc>,
}

impl Task {
    pub fn new(title: String, path: PathBuf) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            path,
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
            sub_tasks: Vec::new(),
            deletion_date: None,
            created_date_time: now,
            last_modified_date_time: now,
        }
    }

    pub fn file_path(&self) -> PathBuf {
        self.path.join(&self.id).with_extension("ron")
    }

    pub fn sub_tasks_path(&self) -> PathBuf {
        self.path.join(&self.id)
    }

    #[allow(dead_code)]
    pub fn parent(&self) -> Option<Task> {
        self.path.parent().and_then(|parent| {
            let content = std::fs::read_to_string(parent.join("ron")).ok()?;
            ron::from_str(&content).ok()
        })
    }

    #[allow(dead_code)]
    pub fn list(&self) -> Option<super::List> {
        self.path.ancestors().find_map(|ancestor| {
            ancestor.parent().and_then(|parent| {
                if parent.ends_with("lists") {
                    ancestor.file_name().and_then(|name| {
                        let list_path = dirs::data_local_dir()
                            .unwrap()
                            .join(Tasks::APP_ID)
                            .join("lists")
                            .join(name)
                            .with_extension("ron");
                        if list_path.exists() {
                            let content = std::fs::read_to_string(list_path).ok()?;
                            ron::from_str(&content).ok()
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            })
        })
    }
}
