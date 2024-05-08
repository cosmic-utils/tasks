use anyhow::Result;
use async_trait::async_trait;

use crate::{
    models::{list::List, task::Task},
    task_service::TasksProvider,
};

use self::engine::ComputerStorageEngine;

mod engine;

#[derive(Debug, Clone)]
pub struct ComputerStorage {
    engine: ComputerStorageEngine,
}

impl ComputerStorage {
    pub(crate) fn new(application_id: &str) -> Option<Self> {
        ComputerStorageEngine::new(application_id).map(|engine| Self { engine })
    }
}

#[async_trait]
impl TasksProvider for ComputerStorage {
    async fn get_task(&mut self, list_id: String, task_id: String) -> Result<Task> {
        self.engine.get_task(&list_id, &task_id)
    }

    async fn get_tasks_from_list(&mut self, parent_list: String) -> Result<Vec<Task>> {
        self.engine.tasks(&parent_list)
    }

    async fn create_task(&mut self, task: Task) -> Result<Task> {
        self.engine.create_task(task)
    }

    async fn update_task(&mut self, task: Task) -> Result<()> {
        self.engine.update_task(task)
    }

    async fn delete_task(&mut self, list_id: String, task_id: String) -> Result<()> {
        self.engine.delete_task(&list_id, &task_id)
    }

    async fn get_lists(&mut self) -> Result<Vec<List>> {
        self.engine.lists()
    }

    async fn get_list(&mut self, id: String) -> Result<List> {
        self.engine.get_list(&id)
    }

    async fn create_list(&mut self, list: List) -> Result<List> {
        self.engine.create_list(list)
    }

    async fn update_list(&mut self, list: List) -> Result<()> {
        self.engine.update_list(list)
    }

    async fn delete_list(&mut self, id: String) -> Result<()> {
        self.engine.delete_list(&id)
    }
}
