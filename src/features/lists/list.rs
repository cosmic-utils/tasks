use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::store::source::TaskSource;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct List {
    #[serde(default = "Timestamp::now")]
    pub created_at: Timestamp,
    pub description: String,
    pub hide_completed: bool,
    pub icon: Option<String>,
    pub id: Uuid,
    pub name: String,
    /// Local (default) or mirrored from a remote Google/Microsoft account.
    #[serde(default)]
    pub source: TaskSource,
}

impl Default for List {
    fn default() -> Self {
        Self {
            created_at: Timestamp::now(),
            description: String::default(),
            hide_completed: false,
            icon: None,
            id: Uuid::default(),
            name: String::default(),
            source: TaskSource::Local,
        }
    }
}

impl List {
    pub fn new(name: impl ToString) -> Self {
        Self {
            created_at: Timestamp::now(),
            description: String::new(),
            hide_completed: false,
            icon: None,
            id: Uuid::new_v4(),
            name: name.to_string(),
            source: TaskSource::Local,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TrashedList {
    pub deleted_at: Timestamp,
    pub list: List,
}

impl TrashedList {
    pub fn new(list: List) -> Self {
        Self {
            deleted_at: Timestamp::now(),
            list,
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
