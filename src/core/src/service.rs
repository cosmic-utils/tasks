use serde::{Deserialize, Serialize};

use crate::{services::computer::ComputerStorage, task_service::TasksProvider};

#[derive(Debug, Clone)]
pub struct TaskService {
    pub app_id: String,
    pub provider: Provider,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Provider {
    #[default]
    Computer,
}

impl TaskService {
    pub fn new(app_id: &str, provider: Provider) -> Self {
        Self {
            app_id: app_id.to_string(),
            provider,
        }
    }

    pub fn services<'a>() -> &'a [Provider] {
        &[Provider::Computer]
    }

    pub fn get_service(&self) -> Option<Box<dyn TasksProvider>> {
        match self.provider {
            Provider::Computer => Some(Box::new(ComputerStorage::new(&self.app_id)?)),
        }
    }

    pub fn title(&self) -> &str {
        match self.provider {
            Provider::Computer => "Computer",
        }
    }

    pub fn icon(&self) -> &str {
        match self.provider {
            Provider::Computer => "computer-symbolic",
        }
    }
}
