use jiff::{civil::Date, Timestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::features::tasks::state::{COMPLETED_STATE_ID, PENDING_STATE_ID};
use crate::shared::store::source::TaskSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub notes: String,
    pub favorite: bool,
    pub today: bool,
    pub expanded: bool,
    #[serde(default)]
    pub state_id: Option<Uuid>,
    pub priority: Priority,
    pub recurrence: Recurrence,
    pub tags: Vec<String>,
    pub parent_id: Option<Uuid>,
    pub sub_task_ids: Vec<Uuid>,
    pub completion_date: Option<Timestamp>,
    pub due_date: Option<Date>,
    pub reminder_date: Option<Timestamp>,
    pub creation_date: Timestamp,
    #[serde(default)]
    pub sort_order: u32,
    /// Local (default) or mirrored from a remote Google/Microsoft account.
    #[serde(default)]
    pub source: TaskSource,
    /// Remote provider's last-modified timestamp, used for last-write-wins
    /// conflict resolution during sync.
    #[serde(default)]
    pub remote_updated_at: Option<Timestamp>,
    #[serde(default)]
    pub last_synced_at: Option<Timestamp>,
    /// Set on any local edit to a remote-sourced task; read and cleared by
    /// the sync engine to decide whether to push. Never persisted.
    #[serde(skip)]
    pub dirty: bool,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            title: String::new(),
            notes: String::new(),
            favorite: false,
            today: false,
            expanded: false,
            state_id: None,
            priority: Priority::Normal,
            recurrence: Recurrence::default(),
            tags: Vec::new(),
            parent_id: None,
            sub_task_ids: Vec::new(),
            completion_date: None,
            due_date: None,
            reminder_date: None,
            creation_date: Timestamp::now(),
            sort_order: 0,
            source: TaskSource::Local,
            remote_updated_at: None,
            last_synced_at: None,
            dirty: false,
        }
    }
}

impl Task {
    pub fn format_timestamp(ts: &jiff::Timestamp) -> String {
        let tz = jiff::tz::TimeZone::system();
        ts.to_zoned(tz).strftime("%m-%d-%Y %H:%M").to_string()
    }

    pub fn creation_date_local(&self) -> String {
        Self::format_timestamp(&self.creation_date)
    }

    #[allow(dead_code)]
    pub fn completion_date_local(&self) -> Option<String> {
        self.completion_date.as_ref().map(Self::format_timestamp)
    }

    pub fn is_completed(&self) -> bool {
        self.completion_date.is_some()
    }

    pub fn effective_state_id(&self) -> Uuid {
        self.state_id.unwrap_or(if self.is_completed() {
            COMPLETED_STATE_ID
        } else {
            PENDING_STATE_ID
        })
    }

    pub fn new(title: impl ToString) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.to_string(),
            notes: String::new(),
            favorite: false,
            today: false,
            expanded: false,
            state_id: None,
            priority: Priority::Normal,
            recurrence: Recurrence::default(),
            tags: Vec::new(),
            parent_id: None,
            sub_task_ids: Vec::new(),
            completion_date: None,
            due_date: None,
            reminder_date: None,
            creation_date: Timestamp::now(),
            sort_order: 0,
            source: TaskSource::Local,
            remote_updated_at: None,
            last_synced_at: None,
            dirty: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Priority {
    #[default]
    Low,
    Normal,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrashedTask {
    pub task: Task,
    pub original_list_id: uuid::Uuid,
    pub deleted_at: jiff::Timestamp,
}

impl TrashedTask {
    pub fn new(task: Task, original_list_id: uuid::Uuid) -> Self {
        Self {
            task,
            original_list_id,
            deleted_at: jiff::Timestamp::now(),
        }
    }
    pub fn deleted_at_local(&self) -> String {
        Task::format_timestamp(&self.deleted_at)
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Recurrence {
    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A task saved before the `source`/`remote_updated_at`/`last_synced_at`
    /// fields existed must still deserialize, defaulting to a local task.
    #[test]
    fn pre_accounts_ron_deserializes_as_local() {
        let ron = r#"
Task(
    id: "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    title: "Legacy task",
    notes: "",
    favorite: false,
    today: false,
    expanded: false,
    priority: Normal,
    recurrence: (
        monday: false, tuesday: false, wednesday: false, thursday: false,
        friday: false, saturday: false, sunday: false,
    ),
    tags: [],
    parent_id: None,
    sub_task_ids: [],
    completion_date: None,
    due_date: None,
    reminder_date: None,
    creation_date: "2026-01-01T00:00:00Z",
)
"#;
        let task: Task = ron::from_str(ron).expect("legacy ron without `source` must still parse");
        assert!(task.source.is_local());
        assert_eq!(task.remote_updated_at, None);
        assert_eq!(task.last_synced_at, None);
        assert!(!task.dirty);
    }
}
