pub mod migration;
pub mod models;

use std::path::PathBuf;

use crate::{
    app::markdown::Markdown,
    storage::models::{List, Task},
    Error, LocalStorageError, TasksError,
};

#[derive(Debug, Clone)]
pub struct LocalStorage {
    paths: LocalStoragePaths,
}

#[derive(Debug, Clone)]
pub struct LocalStoragePaths {
    lists: PathBuf,
}

impl LocalStorage {
    pub fn new(application_id: &str) -> Result<Self, LocalStorageError> {
        let base_path = dirs::data_local_dir()
            .ok_or(LocalStorageError::XdgLocalDirNotFound)?
            .join(application_id);
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
            paths: LocalStoragePaths { lists: lists_path },
        };

        Ok(storage)
    }

    pub fn tasks(&self, list: &List) -> Result<Vec<Task>, Error> {
        let mut tasks = vec![];
        let path = list.tasks_path();
        if !path.exists() {
            return Ok(tasks);
        }
        for entry in path.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let content = std::fs::read_to_string(&path)?;
                let mut task: Task = ron::from_str(&content)?;
                if let Some(stem) = path.file_stem() {
                    let folder_path = path.parent().unwrap().join(stem);
                    if folder_path.is_dir() {
                        task.sub_tasks = Self::sub_tasks(&task)?;
                    }
                }
                tasks.push(task);
            }
        }
        Ok(tasks)
    }

    pub fn sub_tasks(task: &Task) -> Result<Vec<Task>, Error> {
        let mut tasks = vec![];
        let path = task.sub_tasks_path();
        if !path.exists() {
            return Ok(tasks);
        }
        for entry in path.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let content = std::fs::read_to_string(&path)?;
                let mut task: Task = ron::from_str(&content)?;
                if let Some(stem) = path.file_stem() {
                    let folder_path = path.parent().unwrap().join(stem);
                    if folder_path.is_dir() {
                        task.sub_tasks = Self::sub_tasks(&task)?;
                    }
                }
                tasks.push(task);
            }
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

    pub fn create_task(&self, task: &Task) -> Result<Task, Error> {
        let path = task.file_path();
        if !path.exists() {
            std::fs::create_dir_all(&task.path)?;
            let content = ron::to_string(&task)?;
            std::fs::write(path, content)?;
            Ok(task.clone())
        } else {
            Err(Error::Tasks(TasksError::ExistingTask))
        }
    }

    pub fn update_task(&self, task: &Task) -> Result<(), Error> {
        let path = task.file_path();
        if path.exists() {
            let content = ron::to_string(&task)?;
            std::fs::write(path, content)?;
            Ok(())
        } else {
            Err(Error::Tasks(TasksError::TaskNotFound))
        }
    }

    pub fn delete_task(&self, task: &Task) -> Result<(), Error> {
        let path = task.file_path();
        if path.exists() {
            std::fs::remove_file(path)?;
            if task.sub_tasks_path().exists() {
                std::fs::remove_dir_all(task.sub_tasks_path())?;
            }
            let mut entries = std::fs::read_dir(&task.path)?;
            let entry = entries.next();
            if entry.is_none() {
                std::fs::remove_dir_all(&task.path)?;
            }
            Ok(())
        } else {
            Err(Error::Tasks(TasksError::TaskNotFound))
        }
    }

    pub fn create_list(&self, list: &List) -> Result<List, Error> {
        if !list.file_path.exists() {
            let content = ron::to_string(&list)?;
            std::fs::write(&list.file_path, content)?;
            Ok(list.clone())
        } else {
            Err(Error::Tasks(TasksError::ExistingList))
        }
    }

    pub fn update_list(&self, list: &List) -> Result<(), Error> {
        if list.file_path.exists() {
            let content = ron::to_string(&list)?;
            std::fs::write(&list.file_path, content)?;
            Ok(())
        } else {
            Err(Error::Tasks(TasksError::ListNotFound))
        }
    }

    pub fn delete_list(&self, list: &List) -> Result<(), Error> {
        if list.file_path.exists() {
            std::fs::remove_file(&list.file_path)?;
            std::fs::remove_dir_all(list.tasks_path())?;
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
