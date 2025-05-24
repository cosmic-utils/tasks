use std::path::PathBuf;

use cosmic::Application;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::Tasks;

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct List {
    pub id: String,
    pub file_path: PathBuf,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub hide_completed: bool,
}

unsafe impl Send for List {}

impl FromIterator<List> for List {
    fn from_iter<T: IntoIterator<Item = List>>(iter: T) -> Self {
        let mut list = Self::default();
        for item in iter {
            list.name.push_str(&item.name);
        }
        list
    }
}

impl List {
    pub fn new(name: &str) -> Self {
        let id = Uuid::new_v4().to_string();
        let file_path = dirs::data_local_dir()
            .unwrap()
            .join(Tasks::APP_ID)
            .join("lists")
            .join(&id)
            .with_extension("ron");
        Self {
            id,
            file_path,
            name: name.to_string(),
            description: String::new(),
            icon: Some(emojis::get_by_shortcode("pencil").unwrap().to_string()),
            hide_completed: false,
        }
    }

    pub fn tasks_path(&self) -> PathBuf {
        dirs::data_local_dir()
            .unwrap()
            .join(Tasks::APP_ID)
            .join("tasks")
            .join(&self.id)
    }
}
