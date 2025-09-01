use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Local representation of a checklist item
/// Compatible with Microsoft Graph API checklistItem resource
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChecklistItem {
    pub id: String,
    pub display_name: String,
    pub is_checked: bool,
    pub created_date_time: DateTime<Utc>,
    pub checked_date_time: Option<DateTime<Utc>>,
}

impl ChecklistItem {
    /// Create a new checklist item
    pub fn new(display_name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            display_name,
            is_checked: false,
            created_date_time: now,
            checked_date_time: None,
        }
    }

    /// Create from MS Graph API data
    pub fn from_ms_graph(
        id: String,
        display_name: String,
        is_checked: bool,
        created_date_time: String,
        checked_date_time: Option<String>,
    ) -> Result<Self, chrono::ParseError> {
        let created = DateTime::parse_from_rfc3339(&created_date_time)
            .map(|dt| dt.with_timezone(&Utc))?;
        
        let checked = if let Some(checked_str) = checked_date_time {
            Some(DateTime::parse_from_rfc3339(&checked_str)
                .map(|dt| dt.with_timezone(&Utc))?)
        } else {
            None
        };

        Ok(Self {
            id,
            display_name,
            is_checked,
            created_date_time: created,
            checked_date_time: checked,
        })
    }

    /// Mark as checked
    pub fn check(&mut self) {
        if !self.is_checked {
            self.is_checked = true;
            self.checked_date_time = Some(Utc::now());
        }
    }

    /// Toggle the checked status
    pub fn toggle(&mut self) {
        self.is_checked = !self.is_checked;
        if self.is_checked {
            self.checked_date_time = Some(Utc::now());
        } else {
            self.checked_date_time = None;
        }
    }






}

impl Default for ChecklistItem {
    fn default() -> Self {
        Self::new("New Item".to_string())
    }
}
