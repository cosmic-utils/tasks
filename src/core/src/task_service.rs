use async_trait::async_trait;

use crate::{
    models::{List, Task},
    Error,
};

#[async_trait]
pub trait TasksProvider: Sync + Send {
    /// Reads a single task by its id.
    async fn get_task(&mut self, task_list_id: String, task_id: String) -> Result<Task, Error>;

    /// Read all the tasks from a list.
    async fn get_tasks_from_list(&mut self, parent_list: String) -> Result<Vec<Task>, Error>;

    /// Creates a single task.
    async fn create_task(&mut self, task: Task) -> Result<Task, Error>;

    /// Updates a single task.
    async fn update_task(&mut self, task: Task) -> Result<(), Error>;

    /// Deltes a single task.
    async fn delete_task(&mut self, list_id: String, task_id: String) -> Result<(), Error>;

    /// Read all the lists from a service.
    async fn get_lists(&mut self) -> Result<Vec<List>, Error>;

    /// Read a single list from a service.
    async fn get_list(&mut self, id: String) -> Result<List, Error>;

    /// Creates a single task list.
    async fn create_list(&mut self, list: List) -> Result<List, Error>;

    /// Updates a single task list.
    async fn update_list(&mut self, list: List) -> Result<(), Error>;

    /// Deletes a single task list.
    async fn delete_list(&mut self, id: String) -> Result<(), Error>;
}
