use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a list of tasks in the application.
pub struct List {
    /// The unique identifier for the list.
    pub id: Uuid,
    /// The name of the list.
    pub name: String,
    /// A description of the list.
    pub description: String,
    /// An optional icon for the list.
    pub icon: Option<String>,
    /// Whether to hide completed tasks in the list.
    pub hide_completed: bool,
    /// When the list was created, used to preserve creation order.
    #[serde(default = "Timestamp::now")]
    pub created_at: Timestamp,
}

impl Default for List {
    fn default() -> Self {
        Self {
            id: Uuid::default(),
            name: String::default(),
            description: String::default(),
            icon: None,
            hide_completed: false,
            created_at: Timestamp::now(),
        }
    }
}

impl List {
    pub fn new(name: impl ToString) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: String::new(),
            icon: None,
            hide_completed: false,
            created_at: Timestamp::now(),
        }
    }
}
