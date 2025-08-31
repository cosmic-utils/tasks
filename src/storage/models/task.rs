use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Priority, Recurrence, Status, ChecklistItem};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    
    pub title: String,
    pub today: bool,
    pub status: Status,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub notes: String,
    pub completion_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub reminder_date: Option<DateTime<Utc>>,
    pub recurrence: Recurrence,
    #[serde(default)]
    pub expanded: bool,
    
    // Checklist functionality
    #[serde(default)]
    pub checklist_items: Vec<ChecklistItem>,
    #[serde(default)]
    pub checklist_sync_status: ChecklistSyncStatus,
    
    pub created_date_time: DateTime<Utc>,
    pub last_modified_date_time: DateTime<Utc>,
    pub  list_id: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChecklistSyncStatus {
    Synced,
    PendingSync,
    SyncFailed(String),
}

impl Default for ChecklistSyncStatus {
    fn default() -> Self {
        Self::Synced
    }
}

impl Task {
    pub fn new(title: String, list_id : Option<String>) -> Self {
        let now = Utc::now();
        let mut task = Self {
            id: Uuid::new_v4().to_string(),
            
            title,
            today: false,
            status: Status::NotStarted,
            priority: Priority::Low,
            tags: vec![],
            notes: String::new(),
            completion_date: None,
            due_date: None,
            reminder_date: None,
            recurrence: Default::default(),
            expanded: false,
            
            // Checklist initialization
            checklist_items: Vec::new(),
            checklist_sync_status: ChecklistSyncStatus::Synced,
            
            created_date_time: now,
            last_modified_date_time: now,
            list_id: list_id,
        };
        
        // Calculate today field based on due date
        task.update_today_field();
        task
    }

    /// Add a new checklist item
    pub fn add_checklist_item(&mut self, title: String) -> &ChecklistItem {
        let item = ChecklistItem::new(title);
        self.checklist_items.push(item.clone());
        self.checklist_sync_status = ChecklistSyncStatus::PendingSync;
        self.last_modified_date_time = Utc::now();
        self.checklist_items.last().unwrap()
    }

    /// Remove a checklist item by ID
    pub fn remove_checklist_item(&mut self, item_id: &str) -> bool {
        let initial_len = self.checklist_items.len();
        self.checklist_items.retain(|item| item.id != item_id);
        let removed = self.checklist_items.len() < initial_len;
        if removed {
            self.checklist_sync_status = ChecklistSyncStatus::PendingSync;
            self.last_modified_date_time = Utc::now();
        }
        removed
    }

    /// Toggle checklist item completion status
    pub fn toggle_checklist_item(&mut self, item_id: &str) -> bool {
        if let Some(item) = self.checklist_items.iter_mut().find(|item| item.id == item_id) {
            item.toggle();
            self.checklist_sync_status = ChecklistSyncStatus::PendingSync;
            self.last_modified_date_time = Utc::now();
            true
        } else {
            false
        }
    }

    /// Update checklist item title
    pub fn update_checklist_item_title(&mut self, item_id: &str, new_title: String) -> bool {
        if let Some(item) = self.checklist_items.iter_mut().find(|item| item.id == item_id) {
            item.display_name = new_title;
            self.checklist_sync_status = ChecklistSyncStatus::PendingSync;
            self.last_modified_date_time = Utc::now();
            true
        } else {
            false
        }
    }

    /// Get checklist completion percentage
    pub fn checklist_completion_percentage(&self) -> f32 {
        if self.checklist_items.is_empty() {
            return 0.0;
        }
        let completed = self.checklist_items.iter().filter(|item| item.is_checked).count();
        (completed as f32 / self.checklist_items.len() as f32) * 100.0
    }

    /// Check if all checklist items are completed
    pub fn is_checklist_complete(&self) -> bool {
        !self.checklist_items.is_empty() && self.checklist_items.iter().all(|item| item.is_checked)
    }

    /// Check if the task is due today
    pub fn is_due_today(&self) -> bool {
        if let Some(due_date) = self.due_date {
            let today = Utc::now().date_naive();
            due_date.date_naive() == today
        } else {
            false
        }
    }

    /// Update the today field based on due date
    pub fn update_today_field(&mut self) {
        self.today = self.is_due_today();
    }
}
