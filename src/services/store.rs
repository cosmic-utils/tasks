use crate::model::{List, Task, TrashedTask};
use crate::StoreError;
use crate::{Error, Result};
use ron::ser::PrettyConfig;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const LISTS_REGISTRY: &str = "lists.ron";
const TRASH_DIR: &str = "_trash";

fn pretty() -> PrettyConfig {
    PrettyConfig::new().depth_limit(6).struct_names(true)
}

#[derive(Debug, Clone)]
pub struct Store {
    base_dir: PathBuf,
}

impl Store {
    pub fn open(base_dir: impl AsRef<Path>) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        fs::create_dir_all(&base_dir)?;
        Ok(Self { base_dir })
    }

    pub fn lists(&self) -> ListStore<'_> {
        ListStore { store: self }
    }

    pub fn trash(&self) -> TrashStore<'_> {
        TrashStore { store: self }
    }

    pub fn tasks(&self, list_id: Uuid) -> TaskStore<'_> {
        TaskStore {
            store: self,
            list_id,
        }
    }

    fn registry_path(&self) -> PathBuf {
        self.base_dir.join(LISTS_REGISTRY)
    }

    fn trash_dir(&self) -> PathBuf {
        self.base_dir.join(TRASH_DIR)
    }

    fn trashed_task_path(&self, task_id: Uuid) -> PathBuf {
        self.trash_dir().join(format!("{task_id}.ron"))
    }

    fn list_dir(&self, list_id: Uuid) -> PathBuf {
        self.base_dir.join(list_id.to_string())
    }

    fn task_path(&self, list_id: Uuid, task_id: Uuid) -> PathBuf {
        self.list_dir(list_id).join(format!("{task_id}.ron"))
    }
}

pub struct TrashStore<'s> {
    store: &'s Store,
}

impl TrashStore<'_> {
    /// Persist a trashed task.
    pub fn save(&self, trashed: &TrashedTask) -> crate::Result<()> {
        fs::create_dir_all(self.store.trash_dir())?;
        let path = self.store.trashed_task_path(trashed.task.id);
        let content = ron::ser::to_string_pretty(trashed, pretty())?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Load all trashed tasks, sorted newest-first.
    pub fn load_all(&self) -> crate::Result<Vec<TrashedTask>> {
        let trash_dir = self.store.trash_dir();
        if !trash_dir.exists() {
            return Ok(vec![]);
        }
        let mut tasks = Vec::new();
        for entry in fs::read_dir(&trash_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("ron") {
                continue;
            }
            match fs::read_to_string(&path).map(|s| ron::from_str::<TrashedTask>(&s)) {
                Ok(Ok(task)) => tasks.push(task),
                Ok(Err(e)) => tracing::error!("skipping {:?}: {e}", path.file_name()),
                Err(e) => tracing::error!("could not read {:?}: {e}", path.file_name()),
            }
        }
        tasks.sort_by(|a, b| b.deleted_at.cmp(&a.deleted_at));
        Ok(tasks)
    }

    /// Permanently delete a single trashed task.
    pub fn delete(&self, task_id: Uuid) -> crate::Result<()> {
        let path = self.store.trashed_task_path(task_id);
        fs::remove_file(&path)
            .map_err(|_| crate::Error::Store(crate::StoreError::TaskNotFound(task_id)))
    }
}

pub struct ListStore<'s> {
    store: &'s Store,
}

impl ListStore<'_> {
    #[allow(dead_code)]
    pub fn get(&self, list_id: Uuid) -> Result<List> {
        self.load_all()?
            .into_iter()
            .find(|l| l.id == list_id)
            .ok_or(Error::Store(StoreError::ListNotFound(list_id)))
    }

    pub fn load_all(&self) -> Result<Vec<List>> {
        let path = self.store.registry_path();
        if !path.exists() {
            return Ok(vec![]);
        }
        let content = fs::read_to_string(&path)?;
        Ok(ron::from_str(&content)?)
    }

    /// Insert or overwrite a list in the registry and create its task directory.
    pub fn save(&self, list: &List) -> Result<()> {
        fs::create_dir_all(self.store.list_dir(list.id))?;

        let mut lists = self.load_all()?;
        match lists.iter_mut().find(|l| l.id == list.id) {
            Some(existing) => *existing = list.clone(),
            None => lists.push(list.clone()),
        }

        self.flush_registry(&lists)
    }

    /// Update a list in place via a closure.
    pub fn update<F>(&self, list_id: Uuid, f: F) -> Result<List>
    where
        F: FnOnce(&mut List),
    {
        let mut lists = self.load_all()?;
        let list = lists
            .iter_mut()
            .find(|l| l.id == list_id)
            .ok_or(Error::Store(StoreError::ListNotFound(list_id)))?;

        f(list);
        let updated = list.clone();
        self.flush_registry(&lists)?;
        Ok(updated)
    }

    /// Delete a list and all its tasks from disk.
    pub fn delete(&self, list_id: Uuid) -> Result<()> {
        let list_dir = self.store.list_dir(list_id);
        if list_dir.exists() {
            fs::remove_dir_all(&list_dir)?;
        }

        let mut lists = self.load_all()?;
        let before = lists.len();
        lists.retain(|l| l.id != list_id);

        if lists.len() == before {
            return Err(Error::Store(StoreError::ListNotFound(list_id)));
        }

        self.flush_registry(&lists)
    }

    fn flush_registry(&self, lists: &[List]) -> Result<()> {
        let content = ron::ser::to_string_pretty(lists, pretty())?;
        fs::write(self.store.registry_path(), content)?;
        Ok(())
    }
}

pub struct TaskStore<'s> {
    store: &'s Store,
    list_id: Uuid,
}

impl TaskStore<'_> {
    pub fn get(&self, task_id: Uuid) -> Result<Task> {
        let path = self.store.task_path(self.list_id, task_id);
        let content = fs::read_to_string(&path)
            .map_err(|_| Error::Store(StoreError::TaskNotFound(task_id)))?;
        Ok(ron::from_str(&content)?)
    }

    pub fn load_all(&self) -> Result<Vec<Task>> {
        let list_dir = self.store.list_dir(self.list_id);
        if !list_dir.exists() {
            return Err(Error::Store(StoreError::ListNotFound(self.list_id)));
        }

        let mut tasks = Vec::new();
        for entry in fs::read_dir(&list_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) != Some("ron") {
                continue;
            }

            match fs::read_to_string(&path).map(|s| ron::from_str::<Task>(&s)) {
                Ok(Ok(task)) => tasks.push(task),
                Ok(Err(e)) => tracing::error!("warn: skipping {:?}: {e}", path.file_name()),
                Err(e) => tracing::error!("warn: could not read {:?}: {e}", path.file_name()),
            }
        }

        tasks.sort_by(|a, b| a.creation_date.cmp(&b.creation_date));
        Ok(tasks)
    }

    /// Insert or overwrite a task.
    pub fn save(&self, task: &Task) -> Result<()> {
        let list_dir = self.store.list_dir(self.list_id);
        if !list_dir.exists() {
            return Err(Error::Store(StoreError::ListNotFound(self.list_id)));
        }

        let path = self.store.task_path(self.list_id, task.id);
        let content = ron::ser::to_string_pretty(task, pretty())?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Update a task in place via a closure.
    pub fn update<F>(&self, task_id: Uuid, f: F) -> Result<Task>
    where
        F: FnOnce(&mut Task),
    {
        let mut task = self.get(task_id)?;
        f(&mut task);
        self.save(&task)?;
        Ok(task)
    }

    pub fn delete(&self, task_id: Uuid) -> Result<()> {
        let path = self.store.task_path(self.list_id, task_id);
        fs::remove_file(&path).map_err(|_| Error::Store(StoreError::TaskNotFound(task_id)))
    }

    #[allow(dead_code)]
    pub fn query<F>(&self, predicate: F) -> Result<Vec<Task>>
    where
        F: Fn(&Task) -> bool,
    {
        Ok(self.load_all()?.into_iter().filter(predicate).collect())
    }
}
