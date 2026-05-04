use jiff::{civil::Date, Timestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a task in the application.
pub struct Task {
    /// The unique identifier for the task.
    pub id: Uuid,
    /// The title of the task.
    pub title: String,
    /// Notes about the task.
    pub notes: String,
    /// Whether the task is marked as a favorite.
    pub favorite: bool,
    /// Whether the task is marked for today.
    pub today: bool,
    /// Whether the task is expanded in the UI.
    pub expanded: bool,
    /// The status of the task (e.g., not started, completed).
    pub status: Status,
    /// The priority level of the task (e.g., low, normal, high).
    pub priority: Priority,
    /// The recurrence pattern for the task (e.g., which days of the week it recurs on).
    pub recurrence: Recurrence,
    /// The tags associated with the task.
    pub tags: Vec<String>,
    /// The parent task ID (None if this is a top-level task).
    pub parent_id: Option<Uuid>,
    /// The IDs of direct child tasks.
    pub sub_task_ids: Vec<Uuid>,
    /// The date and time when the task was completed, if applicable.
    pub completion_date: Option<Timestamp>,
    /// The date and time when the task is due, if applicable.
    pub due_date: Option<Date>,
    /// The date and time when the task is scheduled, if applicable.
    pub reminder_date: Option<Timestamp>,
    /// The date and time when the task was created.
    pub creation_date: Timestamp,
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
            status: Status::NotStarted,
            priority: Priority::Normal,
            recurrence: Recurrence::default(),
            tags: Vec::new(),
            parent_id: None,
            sub_task_ids: Vec::new(),
            completion_date: None,
            due_date: None,
            reminder_date: None,
            creation_date: Timestamp::now(),
        }
    }
}

impl Task {
    /// Formats a UTC [`jiff::Timestamp`] as a string in the system's local timezone.
    ///
    /// This is the canonical helper for converting stored UTC timestamps into
    /// human-readable local time. All UI code that needs to display a timestamp
    /// should go through this function (or the per-field helpers below) so that
    /// the "store UTC, display local" invariant is enforced in one place.
    pub fn format_timestamp(ts: &jiff::Timestamp) -> String {
        let tz = jiff::tz::TimeZone::system();
        ts.to_zoned(tz).strftime("%m-%d-%Y %H:%M").to_string()
    }

    /// Returns `creation_date` formatted in the system's local timezone.
    pub fn creation_date_local(&self) -> String {
        Self::format_timestamp(&self.creation_date)
    }

    /// Returns `completion_date` formatted in the system's local timezone,
    /// or `None` if the task has not been completed.
    #[allow(dead_code)]
    pub fn completion_date_local(&self) -> Option<String> {
        self.completion_date.as_ref().map(Self::format_timestamp)
    }

    /// Creates a new task with the given title and default values for other fields.
    pub fn new(title: impl ToString) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.to_string(),
            notes: String::new(),
            favorite: false,
            today: false,
            expanded: false,
            status: Status::NotStarted,
            priority: Priority::Normal,
            recurrence: Recurrence::default(),
            tags: Vec::new(),
            parent_id: None,
            sub_task_ids: Vec::new(),
            completion_date: None,
            due_date: None,
            reminder_date: None,
            creation_date: Timestamp::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    #[default]
    /// The task has not been started yet.
    NotStarted,
    /// The task is completed.
    Completed,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Priority {
    #[default]
    /// The task has low priority.
    Low,
    /// The task has normal priority.
    Normal,
    /// The task has high priority.
    High,
}

/// Represents a task that has been moved to the trash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrashedTask {
    /// The task that was trashed.
    pub task: Task,
    /// The list the task originally belonged to.
    pub original_list_id: uuid::Uuid,
    /// When the task was moved to trash.
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
    /// Returns `deleted_at` formatted in the system's local timezone.
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
