/// Provider-agnostic representation of a remote task list, mapped from
/// either Google Tasks or Microsoft Graph To Do API responses.
#[derive(Clone, Debug)]
pub struct RemoteList {
    pub remote_id: String,
    pub title: String,
}

/// Provider-agnostic representation of a remote task.
#[derive(Clone, Debug)]
pub struct RemoteTask {
    pub completed: bool,
    pub due_date: Option<jiff::civil::Date>,
    pub notes: String,
    pub remote_id: String,
    pub title: String,
    /// Provider's last-modified timestamp, used for last-write-wins conflict
    /// resolution against the local `Task::remote_updated_at`.
    pub updated_at: Option<jiff::Timestamp>,
}

/// Fields sent when creating/updating a remote task. Kept separate from
/// `RemoteTask` since providers ignore `remote_id`/`updated_at` on write.
#[derive(Clone, Debug)]
pub struct RemoteTaskDraft {
    pub completed: bool,
    pub due_date: Option<jiff::civil::Date>,
    pub notes: String,
    pub title: String,
}

impl From<&RemoteTask> for RemoteTaskDraft {
    fn from(task: &RemoteTask) -> Self {
        Self {
            completed: task.completed,
            due_date: task.due_date,
            notes: task.notes.clone(),
            title: task.title.clone(),
        }
    }
}
