use crate::features::lists::list::{List, TrashedList};
use crate::features::tasks::state::{default_states, TaskState};
use crate::features::tasks::task::{Task, TrashedTask};
use crate::StoreError;
use crate::{Error, Result};
use ron::ser::PrettyConfig;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const LISTS_REGISTRY: &str = "lists.ron";
const STATES_REGISTRY: &str = "states.ron";
const TRASH_DIR: &str = "_trash";
const TRASHED_LISTS_REGISTRY: &str = "lists.ron";
const TRASHED_LISTS_DIR: &str = "lists";

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

    pub fn states(&self) -> StateStore<'_> {
        StateStore { store: self }
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    fn registry_path(&self) -> PathBuf {
        self.base_dir.join(LISTS_REGISTRY)
    }

    fn states_registry_path(&self) -> PathBuf {
        self.base_dir.join(STATES_REGISTRY)
    }

    fn trash_dir(&self) -> PathBuf {
        self.base_dir.join(TRASH_DIR)
    }

    fn trashed_task_path(&self, task_id: Uuid) -> PathBuf {
        self.trash_dir().join(format!("{task_id}.ron"))
    }

    fn trashed_lists_dir(&self) -> PathBuf {
        self.trash_dir().join(TRASHED_LISTS_DIR)
    }

    fn trashed_lists_registry_path(&self) -> PathBuf {
        self.trashed_lists_dir().join(TRASHED_LISTS_REGISTRY)
    }

    fn trashed_list_data_dir(&self, list_id: Uuid) -> PathBuf {
        self.trashed_lists_dir().join(list_id.to_string())
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
    pub fn save(&self, trashed: &TrashedTask) -> crate::Result<()> {
        fs::create_dir_all(self.store.trash_dir())?;
        let path = self.store.trashed_task_path(trashed.task.id);
        let content = ron::ser::to_string_pretty(trashed, pretty())?;
        fs::write(path, content)?;
        Ok(())
    }

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

    pub fn delete(&self, task_id: Uuid) -> crate::Result<()> {
        let path = self.store.trashed_task_path(task_id);
        fs::remove_file(&path)
            .map_err(|_| crate::Error::Store(crate::StoreError::TaskNotFound(task_id)))
    }

    pub fn trash_list(&self, list_id: Uuid) -> crate::Result<()> {
        let list = self.store.lists().detach(list_id)?;

        fs::create_dir_all(self.store.trashed_lists_dir())?;
        let list_dir = self.store.list_dir(list_id);
        if list_dir.exists() {
            fs::rename(&list_dir, self.store.trashed_list_data_dir(list_id))?;
        }

        let mut lists = self.load_all_lists()?;
        lists.retain(|t| t.list.id != list_id);
        lists.push(TrashedList::new(list));
        self.flush_lists_registry(&lists)
    }

    pub fn load_all_lists(&self) -> crate::Result<Vec<TrashedList>> {
        let path = self.store.trashed_lists_registry_path();
        if !path.exists() {
            return Ok(vec![]);
        }
        let content = fs::read_to_string(&path)?;
        Ok(ron::from_str(&content)?)
    }

    pub fn restore_list(&self, list_id: Uuid) -> crate::Result<List> {
        let mut lists = self.load_all_lists()?;
        let pos = lists
            .iter()
            .position(|t| t.list.id == list_id)
            .ok_or(Error::Store(StoreError::ListNotFound(list_id)))?;
        let trashed = lists.remove(pos);

        let data_dir = self.store.trashed_list_data_dir(list_id);
        let list_dir = self.store.list_dir(list_id);
        if data_dir.exists() {
            fs::rename(&data_dir, &list_dir)?;
        } else {
            fs::create_dir_all(&list_dir)?;
        }

        self.store.lists().save(&trashed.list)?;
        self.flush_lists_registry(&lists)?;
        Ok(trashed.list)
    }

    pub fn delete_list(&self, list_id: Uuid) -> crate::Result<()> {
        let mut lists = self.load_all_lists()?;
        let before = lists.len();
        lists.retain(|t| t.list.id != list_id);

        if lists.len() == before {
            return Err(Error::Store(StoreError::ListNotFound(list_id)));
        }

        let data_dir = self.store.trashed_list_data_dir(list_id);
        if data_dir.exists() {
            fs::remove_dir_all(&data_dir)?;
        }

        self.flush_lists_registry(&lists)
    }

    fn flush_lists_registry(&self, lists: &[TrashedList]) -> crate::Result<()> {
        fs::create_dir_all(self.store.trashed_lists_dir())?;
        let content = ron::ser::to_string_pretty(lists, pretty())?;
        fs::write(self.store.trashed_lists_registry_path(), content)?;
        Ok(())
    }

    pub fn load_trashed_list_tasks(&self, list_id: Uuid) -> crate::Result<Vec<Task>> {
        let data_dir = self.store.trashed_list_data_dir(list_id);
        if !data_dir.exists() {
            return Ok(vec![]);
        }

        let mut tasks = Vec::new();
        for entry in fs::read_dir(&data_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("ron") {
                continue;
            }
            match fs::read_to_string(&path).map(|s| ron::from_str::<Task>(&s)) {
                Ok(Ok(task)) => tasks.push(task),
                Ok(Err(e)) => tracing::error!("skipping {:?}: {e}", path.file_name()),
                Err(e) => tracing::error!("could not read {:?}: {e}", path.file_name()),
            }
        }
        tasks.sort_by(|a, b| a.creation_date.cmp(&b.creation_date));
        Ok(tasks)
    }

    pub fn restore_task_from_list(&self, list_id: Uuid, task_id: Uuid) -> crate::Result<List> {
        let mut lists = self.load_all_lists()?;
        let pos = lists
            .iter()
            .position(|t| t.list.id == list_id)
            .ok_or(Error::Store(StoreError::ListNotFound(list_id)))?;
        let list = lists[pos].list.clone();

        self.store.lists().save(&list)?;
        let list_dir = self.store.list_dir(list_id);
        fs::create_dir_all(&list_dir)?;

        let data_dir = self.store.trashed_list_data_dir(list_id);
        let src = data_dir.join(format!("{task_id}.ron"));
        let dest = self.store.task_path(list_id, task_id);
        fs::rename(&src, &dest)
            .map_err(|_| crate::Error::Store(crate::StoreError::TaskNotFound(task_id)))?;

        if !Self::dir_has_ron_files(&data_dir) {
            if data_dir.exists() {
                fs::remove_dir_all(&data_dir)?;
            }
            lists.remove(pos);
            self.flush_lists_registry(&lists)?;
        }

        Ok(list)
    }

    pub fn delete_task_from_list(&self, list_id: Uuid, task_id: Uuid) -> crate::Result<()> {
        let data_dir = self.store.trashed_list_data_dir(list_id);
        let path = data_dir.join(format!("{task_id}.ron"));
        fs::remove_file(&path)
            .map_err(|_| crate::Error::Store(crate::StoreError::TaskNotFound(task_id)))?;

        if !Self::dir_has_ron_files(&data_dir) {
            if data_dir.exists() {
                fs::remove_dir_all(&data_dir)?;
            }
            let mut lists = self.load_all_lists()?;
            lists.retain(|t| t.list.id != list_id);
            self.flush_lists_registry(&lists)?;
        }

        Ok(())
    }

    fn dir_has_ron_files(dir: &Path) -> bool {
        fs::read_dir(dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .any(|e| e.path().extension().and_then(|x| x.to_str()) == Some("ron"))
            })
            .unwrap_or(false)
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

    pub fn save(&self, list: &List) -> Result<()> {
        fs::create_dir_all(self.store.list_dir(list.id))?;

        let mut lists = self.load_all()?;
        match lists.iter_mut().find(|l| l.id == list.id) {
            Some(existing) => *existing = list.clone(),
            None => lists.push(list.clone()),
        }

        self.flush_registry(&lists)
    }

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

    /// Removes a list from the registry without touching its task directory.
    /// Used when moving a list to trash, where the directory is relocated
    /// rather than deleted.
    pub fn detach(&self, list_id: Uuid) -> Result<List> {
        let mut lists = self.load_all()?;
        let pos = lists
            .iter()
            .position(|l| l.id == list_id)
            .ok_or(Error::Store(StoreError::ListNotFound(list_id)))?;
        let list = lists.remove(pos);
        self.flush_registry(&lists)?;
        Ok(list)
    }

    fn flush_registry(&self, lists: &[List]) -> Result<()> {
        let content = ron::ser::to_string_pretty(lists, pretty())?;
        fs::write(self.store.registry_path(), content)?;
        Ok(())
    }
}

pub struct StateStore<'s> {
    store: &'s Store,
}

impl StateStore<'_> {
    pub fn load_all(&self) -> Result<Vec<TaskState>> {
        let path = self.store.states_registry_path();
        if !path.exists() {
            let states = default_states();
            self.flush_registry(&states)?;
            return Ok(states);
        }
        let content = fs::read_to_string(&path)?;
        Ok(ron::from_str(&content)?)
    }

    #[allow(dead_code)]
    pub fn save(&self, state: &TaskState) -> Result<()> {
        let mut states = self.load_all()?;
        match states.iter_mut().find(|s| s.id == state.id) {
            Some(existing) => *existing = state.clone(),
            None => states.push(state.clone()),
        }
        self.flush_registry(&states)
    }

    #[allow(dead_code)]
    pub fn update<F>(&self, state_id: Uuid, f: F) -> Result<TaskState>
    where
        F: FnOnce(&mut TaskState),
    {
        let mut states = self.load_all()?;
        let state = states
            .iter_mut()
            .find(|s| s.id == state_id)
            .ok_or(Error::Store(StoreError::StateNotFound(state_id)))?;

        f(state);
        let updated = state.clone();
        self.flush_registry(&states)?;
        Ok(updated)
    }

    #[allow(dead_code)]
    pub fn delete(&self, state_id: Uuid) -> Result<()> {
        let mut states = self.load_all()?;
        let before = states.len();
        states.retain(|s| s.id != state_id);

        if states.len() == before {
            return Err(Error::Store(StoreError::StateNotFound(state_id)));
        }

        self.flush_registry(&states)
    }

    fn flush_registry(&self, states: &[TaskState]) -> Result<()> {
        let content = ron::ser::to_string_pretty(states, pretty())?;
        fs::write(self.store.states_registry_path(), content)?;
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
