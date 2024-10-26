use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Getters,
)]
pub struct List {
    pub(crate) id: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
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
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: String::new(),
            icon: Some(emojis::get_by_shortcode("pencil").unwrap().to_string()),
        }
    }
}
