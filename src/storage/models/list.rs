
use cosmic::Application;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::TasksApp;

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct List {
    pub id: String,
    
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub hide_completed: bool,
    #[serde(default)]
    pub number_of_tasks: u32,
    pub well_known_list_name: Option<String>,
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
            .join(TasksApp::APP_ID)
            .join("lists")
            .join(&id)
            .with_extension("ron");
        Self {
            id,
            name: name.to_string(),
            description: String::new(),
            icon: Some("view-list-symbolic".to_string()),
            hide_completed: false,
            number_of_tasks: 0,
            well_known_list_name: None,
        }
    }


}
