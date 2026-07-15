use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PENDING_STATE_ID: Uuid = Uuid::from_u128(1);
pub const COMPLETED_STATE_ID: Uuid = Uuid::from_u128(2);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskState {
    pub id: Uuid,
    pub name: String,
    pub is_completed: bool,
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
