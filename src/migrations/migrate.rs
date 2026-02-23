use crate::model::{List, Task};
use crate::services::store::Store;
use crate::Result;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::models::{List as PreviousList, Task as PreviousTask};

/// Migrator handles the conversion from old storage format to new storage format
pub struct Migrator {
    old_base_dir: PathBuf,
    store: Store,
}

impl Migrator {
    /// Create a new migrator with the old base directory and new store
    pub fn new(old_base_dir: impl AsRef<Path>, store: Store) -> Self {
        Self {
            old_base_dir: old_base_dir.as_ref().to_path_buf(),
            store,
        }
    }

    /// Run the complete migration
    pub fn migrate(&self) -> Result<MigrationReport> {
        let mut report = MigrationReport::default();

        // Find and migrate all lists
        let old_lists_dir = self.old_base_dir.join("lists");
        if !old_lists_dir.exists() {
            tracing::info!("No old lists directory found at {:?}", old_lists_dir);
            return Ok(report);
        }

        tracing::info!("Starting migration from {:?}", self.old_base_dir);
        tracing::info!("Reading lists from {:?}", old_lists_dir);

        // Read all old lists
        for entry in fs::read_dir(&old_lists_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) != Some("ron") {
                continue;
            }

            match self.migrate_list(&path) {
                Ok((list_count, task_count)) => {
                    report.lists_migrated += list_count;
                    report.tasks_migrated += task_count;
                }
                Err(e) => {
                    tracing::error!("Failed to migrate list from {:?}: {}", path, e);
                    report.errors.push(format!("{:?}: {}", path, e));
                }
            }
        }

        tracing::info!("Migration complete!");
        tracing::info!("Lists migrated: {}", report.lists_migrated);
        tracing::info!("Tasks migrated: {}", report.tasks_migrated);
        if !report.errors.is_empty() {
            tracing::info!("Errors encountered: {}", report.errors.len());
            for error in &report.errors {
                tracing::info!("  - {}", error);
            }
        }

        // Create marker file to indicate successful migration
        let marker_file = self.old_base_dir.join("migrated");
        fs::write(&marker_file, "")?;
        tracing::info!("Created migration marker file at {:?}", marker_file);

        Ok(report)
    }

    /// Migrate a single list and all its tasks
    fn migrate_list(&self, list_path: &Path) -> Result<(usize, usize)> {
        tracing::info!("Migrating list from {:?}", list_path);

        // Read old list
        let content = fs::read_to_string(list_path)?;
        let old_list: PreviousList = ron::from_str(&content)?;

        tracing::info!("List: {} (old ID: {})", old_list.name, old_list.id);

        // Convert to new list format
        let new_list = List {
            id: parse_or_generate_uuid(&old_list.id),
            name: old_list.name.clone(),
            description: old_list.description,
            icon: old_list.icon,
            hide_completed: old_list.hide_completed,
        };

        // Save the new list
        self.store.lists().save(&new_list)?;
        tracing::info!("List saved with new ID: {}", new_list.id);

        // Find tasks for this list
        let old_tasks_dir = self.old_base_dir.join("tasks").join(&old_list.id);
        let task_count = if old_tasks_dir.exists() {
            tracing::info!("Reading tasks from {:?}", old_tasks_dir);
            self.migrate_tasks(&old_tasks_dir, new_list.id, None)?
        } else {
            tracing::info!("No tasks directory found for this list");
            0
        };

        tracing::info!("{} tasks migrated", task_count);

        Ok((1, task_count))
    }

    /// Recursively migrate tasks from a directory
    /// Returns the number of tasks migrated
    fn migrate_tasks(
        &self,
        tasks_dir: &Path,
        list_id: Uuid,
        parent_id: Option<Uuid>,
    ) -> Result<usize> {
        let mut count = 0;

        if !tasks_dir.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(tasks_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process .ron files
            if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("ron") {
                continue;
            }

            match self.migrate_task(&path, list_id, parent_id) {
                Ok(task_count) => count += task_count,
                Err(e) => {
                    tracing::error!("    Failed to migrate task from {:?}: {}", path, e);
                }
            }
        }

        Ok(count)
    }

    /// Migrate a single task file and its sub-tasks
    /// Returns the total number of tasks migrated (including sub-tasks)
    fn migrate_task(
        &self,
        task_path: &Path,
        list_id: Uuid,
        parent_id: Option<Uuid>,
    ) -> Result<usize> {
        // Read old task
        let content = fs::read_to_string(task_path)?;
        let old_task: PreviousTask = ron::from_str(&content)?;

        // Convert to new task format
        let new_task_id = parse_or_generate_uuid(&old_task.id);

        tracing::info!("Migrating task: {} ({})", old_task.title, new_task_id);

        // Collect sub-task IDs first (we'll migrate them recursively)
        let mut sub_task_ids = Vec::new();
        let mut sub_task_count = 0;

        // Process sub-tasks if they exist
        if !old_task.sub_tasks.is_empty() {
            tracing::info!("Processing {} sub-tasks", old_task.sub_tasks.len());
            for old_sub_task in &old_task.sub_tasks {
                let sub_task_id = parse_or_generate_uuid(&old_sub_task.id);
                sub_task_ids.push(sub_task_id);

                // Recursively convert sub-task
                let converted =
                    self.convert_task_recursive(old_sub_task, list_id, Some(new_task_id))?;
                sub_task_count += converted;
            }
        }

        // Also check for sub-tasks directory (old nested format)
        if let Some(stem) = task_path.file_stem() {
            let sub_tasks_dir = task_path.parent().unwrap().join(stem);
            if sub_tasks_dir.is_dir() {
                tracing::info!("Found sub-tasks directory: {:?}", sub_tasks_dir);
                let nested_count =
                    self.migrate_tasks(&sub_tasks_dir, list_id, Some(new_task_id))?;
                sub_task_count += nested_count;
            }
        }

        let new_task = Task {
            id: new_task_id,
            title: old_task.title,
            notes: old_task.notes,
            favorite: old_task.favorite,
            today: old_task.today,
            expanded: old_task.expanded,
            status: old_task.status,
            priority: old_task.priority,
            recurrence: old_task.recurrence,
            tags: old_task.tags,
            parent_id,
            sub_task_ids,
            completion_date: old_task.completion_date,
            due_date: old_task.due_date,
            reminder_date: old_task.reminder_date,
            creation_date: old_task.created_date_time,
        };

        // Save the task
        self.store.tasks(list_id).save(&new_task)?;

        // Return 1 for this task + count of all sub-tasks
        Ok(1 + sub_task_count)
    }

    /// Convert a task and all its nested sub-tasks recursively
    fn convert_task_recursive(
        &self,
        old_task: &PreviousTask,
        list_id: Uuid,
        parent_id: Option<Uuid>,
    ) -> Result<usize> {
        let new_task_id = parse_or_generate_uuid(&old_task.id);
        let mut sub_task_ids = Vec::new();
        let mut total_count = 0;

        // First, recursively convert all sub-tasks
        for old_sub_task in &old_task.sub_tasks {
            let sub_task_id = parse_or_generate_uuid(&old_sub_task.id);
            sub_task_ids.push(sub_task_id);

            let converted =
                self.convert_task_recursive(old_sub_task, list_id, Some(new_task_id))?;
            total_count += converted;
        }

        // Create and save the new task
        let new_task = Task {
            id: new_task_id,
            title: old_task.title.clone(),
            notes: old_task.notes.clone(),
            favorite: old_task.favorite,
            today: old_task.today,
            expanded: old_task.expanded,
            status: old_task.status,
            priority: old_task.priority,
            recurrence: old_task.recurrence,
            tags: old_task.tags.clone(),
            parent_id,
            sub_task_ids,
            completion_date: old_task.completion_date,
            due_date: old_task.due_date,
            reminder_date: old_task.reminder_date,
            creation_date: old_task.created_date_time,
        };

        self.store.tasks(list_id).save(&new_task)?;

        // Return 1 for this task + count of all descendants
        Ok(1 + total_count)
    }
}

/// Parse a string UUID or generate a new one if parsing fails
fn parse_or_generate_uuid(id: &str) -> Uuid {
    Uuid::parse_str(id).unwrap_or_else(|_| {
        tracing::error!("      Warning: Invalid UUID '{}', generating new one", id);
        Uuid::new_v4()
    })
}

#[derive(Debug, Default)]
pub struct MigrationReport {
    pub lists_migrated: usize,
    pub tasks_migrated: usize,
    pub errors: Vec<String>,
}
