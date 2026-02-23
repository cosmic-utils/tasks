use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
}

impl List {
    pub fn new(name: impl ToString) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: String::new(),
            icon: None,
            hide_completed: false,
        }
    }
}
