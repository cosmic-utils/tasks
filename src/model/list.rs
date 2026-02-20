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
