use std::fmt::Display;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};

use crate::{
	services::{
		local::service::ComputerStorage,
	},
	task_service::TodoProvider,
};

static APP_ID: OnceLock<&str> = OnceLock::new();

pub struct Services;

impl Services {
	pub fn init(app_id: &'static str) {
		APP_ID.get_or_init(|| app_id);
	}
}

#[derive(
	Debug,
	Default,
	EnumIter,
	EnumString,
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Serialize,
	Deserialize,
)]
pub enum Service {
	#[default]
	Computer,
}

impl Service {
	/// Finds the requested service and returns it.
	/// After implementing the Service trait in your service
	/// struct, register your service here.
	pub fn get_service(&self) -> Box<dyn TodoProvider> {
		if APP_ID.get().is_none() {
			panic!("Must call Service::init before trying to get a service");
		}

		let app_id = APP_ID.get().unwrap().to_string();

		match self {
			Service::Computer => Box::new(ComputerStorage::new(app_id)),
		}
	}

	/// Convenience method to get the list of services.
	pub fn list() -> Vec<Self> {
		Self::iter().collect()
	}

	/// Returns the icon for the service.
	pub fn icon(&self) -> &str {
		match self {
			Service::Computer => {
				"/dev/edfloreshz/Done/icons/scalable/services/computer.png"
			},
		}
	}
}

impl Display for Service {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let str = match self {
			Service::Computer => "Computer".to_string(),
		};
		write!(f, "{}", str)
	}
}
