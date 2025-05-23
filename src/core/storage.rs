use std::path::PathBuf;

use crate::{
    app::markdown::Markdown,
    core::models::{List, Task},
    Error, LocalStorageError, TasksError,
};

#[derive(Debug, Clone)]
pub struct LocalStorage {
    paths: LocalStoragePaths,
}

#[derive(Debug, Clone)]
pub struct LocalStoragePaths {
    lists: PathBuf,
    tasks: PathBuf,
}

impl LocalStorage {
    pub fn new(application_id: &str) -> Result<Self, LocalStorageError> {
        let base_path = dirs::data_local_dir()
            .ok_or(LocalStorageError::XdgLocalDirNotFound)?
            .join(&application_id);
        let lists_path = base_path.join("lists");
        let tasks_path = base_path.join("tasks");
        if !base_path.exists() {
            std::fs::create_dir_all(&base_path)
                .map_err(LocalStorageError::LocalStorageDirectoryCreationFailed)?;
        }
        if !lists_path.exists() {
            std::fs::create_dir_all(&lists_path)
                .map_err(LocalStorageError::ListsDirectoryCreationFailed)?;
        }
        if !tasks_path.exists() {
            std::fs::create_dir_all(&tasks_path)
                .map_err(LocalStorageError::TasksDirectoryCreationFailed)?;
        }
        let storage = Self {
            paths: LocalStoragePaths {
                lists: lists_path,
                tasks: tasks_path,
            },
        };

        Ok(storage)
    }

    pub fn tasks(&self, list_id: &str) -> Result<Vec<Task>, Error> {
        let mut tasks = vec![];
        let path = self.paths.tasks.join(list_id);
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
        for entry in self.paths.lists.read_dir()? {
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
            .paths
            .tasks
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
            .paths
            .tasks
            .join(&task.parent)
            .join(&task.id)
            .with_extension("ron");
        if !path.exists() {
            std::fs::create_dir_all(self.paths.tasks.join(&task.parent))?;
            let content = ron::to_string(&task)?;
            std::fs::write(path, content)?;
            Ok(task)
        } else {
            Err(Error::Tasks(TasksError::ExistingTask))
        }
    }

    pub fn update_task(&self, task: Task) -> Result<(), Error> {
        let path = self
            .paths
            .tasks
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
            .paths
            .tasks
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
        let path = self.paths.lists.join(list_id).with_extension("ron");
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let list = ron::from_str(&content)?;
            Ok(list)
        } else {
            Err(Error::Tasks(TasksError::ListNotFound))
        }
    }

    pub fn create_list(&self, list: List) -> Result<List, Error> {
        let path = self.paths.lists.join(&list.id).with_extension("ron");
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
        let path = self.paths.lists.join(&list.id).with_extension("ron");
        if path.exists() {
            let content = ron::to_string(&list)?;
            std::fs::write(path, content)?;
            Ok(())
        } else {
            Err(Error::Tasks(TasksError::ListNotFound))
        }
    }

    pub fn delete_list(&self, list_id: &str) -> Result<(), Error> {
        let path = self.paths.lists.join(list_id).with_extension("ron");
        let tasks = self.paths.tasks.join(list_id);
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
}
