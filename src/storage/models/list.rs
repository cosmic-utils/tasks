
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VirtualListType {
    MyDay,
    Planned,
    All,
}

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
    
    // Virtual list fields
    #[serde(default)]
    pub is_virtual: bool,
    pub virtual_type: Option<VirtualListType>,
    #[serde(default)]
    pub sort_order: i32,
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
        
        Self {
            id,
            name: name.to_string(),
            description: String::new(),
            icon: Some("view-list-symbolic".to_string()),
            hide_completed: false,
            number_of_tasks: 0,
            well_known_list_name: None,
            is_virtual: false,
            virtual_type: None,
            sort_order: 0,
        }
    }

    pub fn new_virtual(virtual_type: VirtualListType, name: &str) -> Self {
        let sort_order = match virtual_type {
            VirtualListType::MyDay => 1,
            VirtualListType::Planned => 2,
            VirtualListType::All => 3,
        };
        
        let icon = match virtual_type {
            VirtualListType::MyDay => "alarm-symbolic",
            VirtualListType::Planned => "calendar-symbolic",
            VirtualListType::All => "view-list-symbolic",
        };
        
        Self {
            id: format!("virtual_{:?}", virtual_type),
            name: name.to_string(),
            description: String::new(),
            icon: Some(icon.to_string()),
            hide_completed: false,
            number_of_tasks: 0,
            well_known_list_name: None,
            is_virtual: true,
            virtual_type: Some(virtual_type),
            sort_order,
        }
    }


}
