use jiff::{civil::Date, Timestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::features::tasks::state::{COMPLETED_STATE_ID, PENDING_STATE_ID};

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
    /// The ID of the task's user-assigned state (see
    /// [`crate::features::tasks::state::TaskState`]), if any. A task is not
    /// required to have a state. This is independent of completion, which is
    /// tracked via `completion_date`.
    #[serde(default)]
    pub state_id: Option<Uuid>,
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
    #[serde(default)]
    pub sort_order: u32,
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

    /// A task is completed if (and only if) it has a `completion_date`.
    /// Completion is tracked independently of `state_id`: state is an
    /// optional, user-assigned label, while completion is a plain boolean
    /// signal used by the checkbox, strikethrough, and "hide completed".
    pub fn is_completed(&self) -> bool {
        self.completion_date.is_some()
    }

    /// The state ID to group this task under for display purposes: its
    /// user-assigned `state_id` if set, otherwise the built-in "Pending" or
    /// "Completed" state matching its current completion, so tasks without
    /// an explicit state still fall into a sensible section.
    pub fn effective_state_id(&self) -> Uuid {
        self.state_id.unwrap_or(if self.is_completed() {
            COMPLETED_STATE_ID
        } else {
            PENDING_STATE_ID
        })
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
        }
    }
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
