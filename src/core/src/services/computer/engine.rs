use std::path::PathBuf;

use crate::models::{list::List, task::Task};

#[derive(Debug, Clone)]
pub struct ComputerStorageEngine {
    path: PathBuf,
}

impl ComputerStorageEngine {
    pub fn new(application_id: &str) -> Option<Self> {
        let path = dirs::data_local_dir()?.join(application_id);
        let engine = Self { path };
        if !engine.path.exists() {
            std::fs::create_dir_all(&engine.path).ok()?;
        }
        if !engine.lists_path().exists() {
            std::fs::create_dir_all(&engine.lists_path()).ok()?;
        }
        if !engine.tasks_path().exists() {
            std::fs::create_dir_all(&engine.tasks_path()).ok()?;
        }
        Some(engine)
    }

    pub fn tasks(&self, list_id: &str) -> anyhow::Result<Vec<Task>> {
        let mut tasks = vec![];
        for entry in self.tasks_path().join(list_id).read_dir()? {
            let entry = entry?;
            let path = entry.path();
            let content = std::fs::read_to_string(&path)?;
            let task = ron::from_str(&content)?;
            tasks.push(task);
        }
        Ok(tasks)
    }

    pub fn lists(&self) -> anyhow::Result<Vec<List>> {
        let mut tasks = vec![];
        for entry in self.lists_path().read_dir()? {
            let entry = entry?;
            let path = entry.path();
            let content = std::fs::read_to_string(&path)?;
            let task = ron::from_str(&content)?;
            tasks.push(task);
        }
        Ok(tasks)
    }

    pub fn get_task(&self, list_id: &str, task_id: &str) -> anyhow::Result<Task> {
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
            Err(anyhow::anyhow!("Task does not exist"))
        }
    }

    pub fn create_task(&self, task: Task) -> anyhow::Result<Task> {
        let path = self
            .tasks_path()
            .join(&task.parent)
            .join(&task.id)
            .with_extension("ron");
        if !path.exists() {
            std::fs::create_dir_all(&self.tasks_path().join(&task.parent))?;
            let content = ron::to_string(&task)?;
            std::fs::write(path, content)?;
            Ok(task)
        } else {
            Err(anyhow::anyhow!("Task already exists"))
        }
    }

    pub fn update_task(&self, task: Task) -> anyhow::Result<()> {
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
            Err(anyhow::anyhow!("Task does not exist"))
        }
    }

    pub fn delete_task(&self, list_id: &str, task_id: &str) -> anyhow::Result<()> {
        let path = self
            .tasks_path()
            .join(list_id)
            .join(task_id)
            .with_extension("ron");
        if path.exists() {
            std::fs::remove_file(path)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task does not exist"))
        }
    }

    pub fn get_list(&self, list_id: &str) -> anyhow::Result<List> {
        let path = self.lists_path().join(list_id).with_extension("ron");
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let list = ron::from_str(&content)?;
            Ok(list)
        } else {
            Err(anyhow::anyhow!("List does not exist"))
        }
    }

    pub fn create_list(&self, list: List) -> anyhow::Result<List> {
        let path = self.lists_path().join(&list.id).with_extension("ron");
        if !path.exists() {
            let content = ron::to_string(&list)?;
            std::fs::write(path, content).unwrap();
            Ok(list)
        } else {
            Err(anyhow::anyhow!("List already exists"))
        }
    }

    pub fn update_list(&self, list: List) -> anyhow::Result<()> {
        let path = self.lists_path().join(&list.id).with_extension("ron");
        if path.exists() {
            let content = ron::to_string(&list)?;
            std::fs::write(path, content)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("List does not exist"))
        }
    }

    pub fn delete_list(&self, list_id: &str) -> anyhow::Result<()> {
        let path = self.lists_path().join(list_id).with_extension("ron");
        if path.exists() {
            std::fs::remove_file(path)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("List does not exist"))
        }
    }

    pub fn lists_path(&self) -> PathBuf {
        self.path.join("lists")
    }

    pub fn tasks_path(&self) -> PathBuf {
        self.path.join("tasks")
    }
}
