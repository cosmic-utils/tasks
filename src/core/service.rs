use std::path::Path;

use crate::Error;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, Connection};

use crate::{
    core::models::{List, Priority, Recurrence, Status, Task},
    core::services::computer::ComputerStorage,
    core::task_service::TasksProvider,
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

#[allow(unused)]
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

    pub async fn migrate(app_id: &str) -> Result<(), Error> {
        if let Some(mut service) = TaskService::new(app_id, Provider::Computer).get_service() {
            let database_path = dirs::config_dir()
                .unwrap()
                .join(app_id)
                .join("v1/database/")
                .join(format!("{}.db", app_id));
            let lists = get_lists(&database_path).await?;
            let tasks = get_tasks(&database_path).await?;
            for list in lists {
                service.create_list(list.clone()).await?;
                let tasks = tasks
                    .iter()
                    .filter(|task| task.parent == list.id)
                    .cloned()
                    .collect::<Vec<Task>>();
                for task in tasks {
                    service.create_task(task).await?;
                }
            }
        }
        Ok(())
    }
}

use sqlx::Row;

async fn get_tasks(database_path: &Path) -> Result<Vec<Task>, Error> {
    let mut conn = sqlx::SqliteConnection::connect(database_path.to_str().unwrap()).await?;
    let tasks = sqlx::query("SELECT * FROM tasks")
        .map(|row: SqliteRow| Task {
            id: row.get(0),
            parent: row.get(1),
            title: row.get(2),
            notes: row.get(3),
            priority: Priority::from(row.get::<i32, _>(4)),
            favorite: row.get(5),
            status: Status::from(row.get::<i32, _>(6)),
            completion_date: NaiveDateTime::parse_from_str(row.get(7), "%Y-%m-%d %H:%M:%S.%f")
                .map(|ndt| ndt.and_utc())
                .ok(),
            due_date: NaiveDateTime::parse_from_str(row.get(8), "%Y-%m-%d %H:%M:%S.%f")
                .map(|ndt| ndt.and_utc())
                .ok(),
            reminder_date: NaiveDateTime::parse_from_str(row.get(9), "%Y-%m-%d %H:%M:%S.%f")
                .map(|ndt| ndt.and_utc())
                .ok(),
            created_date_time: NaiveDateTime::parse_from_str(row.get(10), "%Y-%m-%d %H:%M:%S.%f")
                .unwrap()
                .and_utc(),
            last_modified_date_time: NaiveDateTime::parse_from_str(
                row.get(11),
                "%Y-%m-%d %H:%M:%S.%f",
            )
            .unwrap()
            .and_utc(),
            sub_tasks: serde_json::from_str(row.get(12)).unwrap(),
            tags: serde_json::from_str(row.get(13)).unwrap(),
            today: row.get(14),
            deletion_date: NaiveDateTime::parse_from_str(row.get(15), "%Y-%m-%d %H:%M:%S.%f")
                .map(|ndt| ndt.and_utc())
                .ok(),
            recurrence: Recurrence::from_string(row.get(16)),
        })
        .fetch_all(&mut conn)
        .await?;
    Ok(tasks)
}

async fn get_lists(database_path: &Path) -> Result<Vec<List>, Error> {
    let mut conn = sqlx::SqliteConnection::connect(database_path.to_str().unwrap()).await?;
    let tasks = sqlx::query("SELECT * FROM lists")
        .map(|row: SqliteRow| List {
            id: row.get(0),
            name: row.get(1),
            description: row.get(2),
            icon: row.get(3),
        })
        .fetch_all(&mut conn)
        .await?;
    Ok(tasks)
}
