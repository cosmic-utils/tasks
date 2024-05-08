use serde::{Deserialize, Serialize};

use crate::{
    models::{list::List, task::Task},
    services::computer::ComputerStorage,
    task_service::TasksProvider,
};

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
            Provider::Computer => ComputerStorage::new(&self.app_id)
                .map(|storage| Box::new(storage) as Box<dyn TasksProvider>),
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

    pub async fn migrate(app_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut service) = TaskService::new(app_id, Provider::Computer).get_service() {
            let path = dirs::config_dir()
                .unwrap()
                .join(app_id)
                .join("v1/database/migration.ron");
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                let migration: MigrationData = ron::from_str(&content)?;
                for list in migration.lists {
                    service.create_list(list.clone()).await?;
                    let tasks = migration
                        .tasks
                        .iter()
                        .filter(|task| task.parent == list.id)
                        .cloned()
                        .collect::<Vec<Task>>();
                    for task in tasks {
                        service.create_task(task).await?;
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MigrationData {
    lists: Vec<List>,
    tasks: Vec<Task>,
}
