pub mod migration;
pub mod models;
pub mod notes_crypto;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    Error, LocalStorageError, TasksError,
    app::markdown::{ImportedList, ImportedTask, Markdown},
    storage::models::{List, Status, Task},
};

#[derive(Debug, Clone)]
pub struct LocalStorage {
    paths: LocalStoragePaths,
    /// Whether new writes should encrypt `Task::notes`. Reads always
    /// auto-detect, so this only governs the *outgoing* shape. Wrapped
    /// in `Arc<AtomicBool>` so all clones in the app see the latest
    /// value the moment the user toggles the setting.
    encrypt_notes: Arc<AtomicBool>,
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
            encrypt_notes: Arc::new(AtomicBool::new(false)),
        };

        Ok(storage)
    }

    pub fn set_encrypt_notes(&self, on: bool) {
        self.encrypt_notes.store(on, Ordering::Relaxed);
    }

    pub fn encrypt_notes_enabled(&self) -> bool {
        self.encrypt_notes.load(Ordering::Relaxed)
    }

    /// Decrypt `task.notes` in place. Errors are downgraded to a warning
    /// and the (still-encrypted) value is left as-is — that way a missing
    /// key (e.g. on a fresh machine that hasn't unlocked the keyring yet)
    /// does not crash list views.
    fn decrypt_notes(task: &mut Task) {
        if !notes_crypto::is_encrypted(&task.notes) {
            return;
        }
        match notes_crypto::decrypt(&task.notes) {
            Ok(plain) => task.notes = plain,
            Err(e) => tracing::warn!("decrypt notes for task {}: {e}", task.id),
        }
    }

    fn encrypt_notes_for_write(&self, task: &mut Task) {
        if !self.encrypt_notes_enabled() {
            return;
        }
        if task.notes.is_empty() {
            return;
        }
        match notes_crypto::encrypt(&task.notes) {
            Ok(ct) => task.notes = ct,
            Err(e) => tracing::warn!("encrypt notes for task {}: {e}", task.id),
        }
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
                Self::decrypt_notes(&mut task);
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
                Self::decrypt_notes(&mut task);
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
            let mut to_write = task.clone();
            self.encrypt_notes_for_write(&mut to_write);
            let content = ron::to_string(&to_write)?;
            std::fs::write(path, content)?;
            Ok(task.clone())
        } else {
            Err(Error::Tasks(TasksError::ExistingTask))
        }
    }

    /// Local edit: bumps last_modified_date_time so the sync engine pushes it.
    pub fn update_task(&self, task: &Task) -> Result<(), Error> {
        let path = task.file_path();
        if path.exists() {
            let mut touched = task.clone();
            touched.last_modified_date_time = chrono::Utc::now();
            self.encrypt_notes_for_write(&mut touched);
            let content = ron::to_string(&touched)?;
            std::fs::write(path, content)?;
            Ok(())
        } else {
            Err(Error::Tasks(TasksError::TaskNotFound))
        }
    }

    /// Sync write: preserves last_modified_date_time as set by the caller.
    /// Used when pulling remote state into local storage.
    pub fn replace_task(&self, task: &Task) -> Result<(), Error> {
        let path = task.file_path();
        if path.exists() {
            let mut to_write = task.clone();
            self.encrypt_notes_for_write(&mut to_write);
            let content = ron::to_string(&to_write)?;
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

    /// Materialize a parsed markdown document as a brand-new local list with
    /// its tasks (and sub-task tree) on disk. The list is created fresh so
    /// it does not collide with any existing CalDAV-bound list; users can
    /// later attach a remote URL via sync. Returns the persisted `List`.
    pub fn import_list(&self, parsed: ImportedList, fallback_name: &str) -> Result<List, Error> {
        let name = parsed
            .name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(fallback_name);
        let list = List::new(name);
        let list = self.create_list(&list)?;
        let tasks_path = list.tasks_path();
        if !tasks_path.exists() {
            std::fs::create_dir_all(&tasks_path)?;
        }
        for task in &parsed.tasks {
            self.write_imported_task(task, &tasks_path)?;
        }
        Ok(list)
    }

    fn write_imported_task(
        &self,
        task: &ImportedTask,
        parent_dir: &std::path::Path,
    ) -> Result<(), Error> {
        let now = chrono::Utc::now();
        let mut model = Task::new(task.title.clone(), parent_dir.to_path_buf());
        model.status = if task.completed {
            Status::Completed
        } else {
            Status::NotStarted
        };
        if task.completed {
            model.completion_date = Some(now);
        }
        self.create_task(&model)?;
        if !task.children.is_empty() {
            let sub_dir = model.sub_tasks_path();
            std::fs::create_dir_all(&sub_dir)?;
            for child in &task.children {
                self.write_imported_task(child, &sub_dir)?;
            }
        }
        Ok(())
    }
}
