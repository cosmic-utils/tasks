use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::service::Service;

#[derive(
	Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct List {
	pub id: String,
	pub name: String,
	pub description: String,
	pub icon: Option<String>,
	pub service: Service,
}

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
	pub fn new(name: &str, service: Service) -> Self {
		Self {
			id: Uuid::new_v4().to_string(),
			name: name.to_string(),
			service,
			description: String::new(),
			icon: Some("✍️".to_string()),
		}
	}
}