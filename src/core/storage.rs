use std::path::PathBuf;

use crate::{
    app::markdown::Markdown,
    core::{
        models::{List, Task},
        TasksError,
    },
    Error,
};

#[derive(Debug, Clone)]
pub struct ComputerStorage {
    application_id: String,
}

impl ComputerStorage {
    /// Creates a new instance of `ComputerStorage`.
    /// If the directory does not exist, it will be created.
    /// Returns `None` if the data local directory cannot be determined.
    /// # Arguments
    /// * `application_id` - The application ID used to create the storage path.
    /// # Returns
    /// * `Some(ComputerStorage)` - A new instance of the storage engine.
    /// * `None` - If the data local directory cannot be determined.
    pub fn new(application_id: &str) -> Self {
        let storage = Self {
            application_id: application_id.to_string(),
        };
        if !storage.path().exists() {
            std::fs::create_dir_all(&storage.application_id)
                .expect("Failed to create storage directory");
        }
        if !storage.lists_path().exists() {
            std::fs::create_dir_all(storage.lists_path())
                .expect("Failed to create lists directory");
        }
        if !storage.tasks_path().exists() {
            std::fs::create_dir_all(storage.tasks_path())
                .expect("Failed to create tasks directory");
        }
        storage
    }

    pub fn path(&self) -> PathBuf {
        dirs::data_local_dir()
            .expect("Failed to get data local dir")
            .join(&self.application_id)
    }

    pub fn tasks(&self, list_id: &str) -> Result<Vec<Task>, Error> {
        let mut tasks = vec![];
        let path = self.tasks_path().join(list_id);
        if !path.exists() {
            return Ok(tasks);
        }
        for entry in path.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            let content = std::fs::read_to_string(&path)?;
            let task = ron::from_str(&content)?;
            tasks.push(task);
        }
        Ok(tasks)
    }

    pub fn lists(&self) -> Result<Vec<List>, Error> {
        let mut lists = vec![];
        let path = self.lists_path();
        if !path.exists() {
            return Ok(lists);
        }
        for entry in self.lists_path().read_dir()? {
            let entry = entry?;
            let path = entry.path();
            let content = std::fs::read_to_string(&path)?;
            let list = ron::from_str(&content)?;
            lists.push(list);
        }
        Ok(lists)
    }

    #[allow(unused)]
    pub fn get_task(&self, list_id: &str, task_id: &str) -> Result<Task, Error> {
        let path = self
            .tasks_path()
            .join(list_id)
            .join(task_id)
            .with_extension("ron");
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let task = ron::from_str(&content)?;
            Ok(task)
        } else {
            Err(Error::Tasks(TasksError::TaskNotFound))
        }
    }

    pub fn create_task(&self, task: Task) -> Result<Task, Error> {
        let path = self
            .tasks_path()
            .join(&task.parent)
            .join(&task.id)
            .with_extension("ron");
        if !path.exists() {
            std::fs::create_dir_all(self.tasks_path().join(&task.parent))?;
            let content = ron::to_string(&task)?;
            std::fs::write(path, content)?;
            Ok(task)
        } else {
            Err(Error::Tasks(TasksError::ExistingTask))
        }
    }

    pub fn update_task(&self, task: Task) -> Result<(), Error> {
        let path = self
            .tasks_path()
            .join(&task.parent)
            .join(&task.id)
            .with_extension("ron");
        if path.exists() {
            let content = ron::to_string(&task)?;
            std::fs::write(path, content)?;
            Ok(())
        } else {
            Err(Error::Tasks(TasksError::TaskNotFound))
        }
    }

    pub fn delete_task(&self, list_id: &str, task_id: &str) -> Result<(), Error> {
        let path = self
            .tasks_path()
            .join(list_id)
            .join(task_id)
            .with_extension("ron");
        if path.exists() {
            std::fs::remove_file(path)?;
            Ok(())
        } else {
            Err(Error::Tasks(TasksError::TaskNotFound))
        }
    }

    #[allow(unused)]
    pub fn get_list(&self, list_id: &str) -> Result<List, Error> {
        let path = self.lists_path().join(list_id).with_extension("ron");
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let list = ron::from_str(&content)?;
            Ok(list)
        } else {
            Err(Error::Tasks(TasksError::ListNotFound))
        }
    }

    pub fn create_list(&self, list: List) -> Result<List, Error> {
        let path = self.lists_path().join(&list.id).with_extension("ron");
        println!("{path:?}");
        if !path.exists() {
            let content = ron::to_string(&list)?;
            std::fs::write(path, content).unwrap();
            Ok(list)
        } else {
            Err(Error::Tasks(TasksError::ExistingList))
        }
    }

    pub fn update_list(&self, list: List) -> Result<(), Error> {
        let path = self.lists_path().join(&list.id).with_extension("ron");
        if path.exists() {
            let content = ron::to_string(&list)?;
            std::fs::write(path, content)?;
            Ok(())
        } else {
            Err(Error::Tasks(TasksError::ListNotFound))
        }
    }

    pub fn delete_list(&self, list_id: &str) -> Result<(), Error> {
        let path = self.lists_path().join(list_id).with_extension("ron");
        let tasks = self.tasks_path().join(list_id);
        if path.exists() {
            std::fs::remove_file(path)?;
            std::fs::remove_dir_all(tasks)?;
            Ok(())
        } else {
            Err(Error::Tasks(TasksError::ListNotFound))
        }
    }

    pub fn export_list(list: &List, tasks: &[Task]) -> String {
        let markdown = list.markdown();
        let tasks_markdown: String = tasks.iter().map(Markdown::markdown).collect();
        format!("{markdown}\n{tasks_markdown}")
    }

    pub fn lists_path(&self) -> PathBuf {
        self.path().join("lists")
    }

    pub fn tasks_path(&self) -> PathBuf {
        self.path().join("tasks")
    }
}
