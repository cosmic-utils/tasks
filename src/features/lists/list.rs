use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub hide_completed: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrashedList {
    pub list: List,
    pub deleted_at: Timestamp,
}

impl TrashedList {
    pub fn new(list: List) -> Self {
        Self {
            list,
            deleted_at: Timestamp::now(),
        }
    }

    pub fn deleted_at_local(&self) -> String {
        let tz = jiff::tz::TimeZone::system();
        self.deleted_at
            .to_zoned(tz)
            .strftime("%m-%d-%Y %H:%M")
            .to_string()
    }
}
