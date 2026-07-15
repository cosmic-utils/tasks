use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The built-in "not completed" state, seeded on first run and used as the
/// default state for new tasks and as the migration target for legacy
/// `Status::NotStarted` tasks.
pub const PENDING_STATE_ID: Uuid = Uuid::from_u128(1);
/// The built-in "completed" state, seeded on first run and used as the
/// migration target for legacy `Status::Completed` tasks.
pub const COMPLETED_STATE_ID: Uuid = Uuid::from_u128(2);

/// A user-defined task state (e.g. "Pending", "In Progress", "Completed").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskState {
    /// The unique identifier for the state.
    pub id: Uuid,
    /// The display name of the state.
    pub name: String,
    /// Whether tasks in this state are treated as completed, i.e. checked
    /// off, struck through, and hidden when "hide completed" is enabled.
    pub is_completed: bool,
    /// The display order of the state relative to other states.
    pub position: u32,
}

impl TaskState {
    #[allow(dead_code)]
    pub fn new(name: impl ToString, is_completed: bool, position: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            is_completed,
            position,
        }
    }
}

/// The default states seeded the first time the state registry is loaded.
pub fn default_states() -> Vec<TaskState> {
    vec![
        TaskState {
            id: PENDING_STATE_ID,
            name: "Pending".to_string(),
            is_completed: false,
            position: 0,
        },
        TaskState {
            id: COMPLETED_STATE_ID,
            name: "Completed".to_string(),
            is_completed: true,
            position: 1,
        },
    ]
}
