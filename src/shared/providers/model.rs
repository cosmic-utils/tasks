/// Provider-agnostic representation of a remote task list, mapped from
/// either Google Tasks or Microsoft Graph To Do API responses.
#[derive(Debug, Clone)]
pub struct RemoteList {
    pub remote_id: String,
    pub title: String,
}

/// Provider-agnostic representation of a remote task.
#[derive(Debug, Clone)]
pub struct RemoteTask {
    pub remote_id: String,
    pub title: String,
    pub notes: String,
    pub due_date: Option<jiff::civil::Date>,
    pub completed: bool,
    /// Provider's last-modified timestamp, used for last-write-wins conflict
    /// resolution against the local `Task::remote_updated_at`.
    pub updated_at: Option<jiff::Timestamp>,
}

/// Fields sent when creating/updating a remote task. Kept separate from
/// `RemoteTask` since providers ignore `remote_id`/`updated_at` on write.
#[derive(Debug, Clone)]
pub struct RemoteTaskDraft {
    pub title: String,
    pub notes: String,
    pub due_date: Option<jiff::civil::Date>,
    pub completed: bool,
}

impl From<&RemoteTask> for RemoteTaskDraft {
    fn from(task: &RemoteTask) -> Self {
        Self {
            title: task.title.clone(),
            notes: task.notes.clone(),
            due_date: task.due_date,
            completed: task.completed,
        }
    }
}
