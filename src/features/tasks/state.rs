use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PENDING_STATE_ID: Uuid = Uuid::from_u128(1);
pub const COMPLETED_STATE_ID: Uuid = Uuid::from_u128(2);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TaskState {
    pub id: Uuid,
    pub is_completed: bool,
    pub name: String,
    pub position: u32,
}

impl TaskState {
    #[allow(dead_code)]
    pub fn new(name: impl ToString, is_completed: bool, position: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            is_completed,
            name: name.to_string(),
            position,
        }
    }
}

pub fn default_states() -> Vec<TaskState> {
    vec![
        TaskState {
            id: PENDING_STATE_ID,
            is_completed: false,
            name: "Pending".to_string(),
            position: 0,
        },
        TaskState {
            id: COMPLETED_STATE_ID,
            is_completed: true,
            name: "Completed".to_string(),
            position: 1,
        },
    ]
}
