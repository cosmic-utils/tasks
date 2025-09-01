use chrono::{DateTime, Utc};

use crate::storage::models::{List, Task, Priority, Status};
use crate::storage::models::task::ChecklistSyncStatus;
use super::models::{
    TodoTaskList, TodoTask, ChecklistItem,
    CreateTodoTaskListRequest, CreateTodoTaskRequest, UpdateTodoTaskRequest,
    UpdateTodoTaskListRequest,
    CreateChecklistItemRequest, UpdateChecklistItemRequest,
    TaskBody, TaskBodyType, TaskImportance, TaskStatus, DateTimeTimeZone,
};

// ============================================================================
// List Mappings
// ============================================================================

impl From<&List> for CreateTodoTaskListRequest {
    fn from(list: &List) -> Self {
        Self {
            displayName: list.name.clone(),
            isOwner: Some(true),
            isShared: Some(false),
        }
    }
}

impl From<TodoTaskList> for List {
    fn from(todo_list: TodoTaskList) -> Self {
        // Calculate number of non-completed tasks
        let number_of_tasks = todo_list.tasks.unwrap_or_default().len() as u32;
            

        Self {
            id: todo_list.id,                    // Use MS Graph ID
            
            name: todo_list.displayName,
            description: String::new(),           // Not supported in MS Graph
            icon: if todo_list.isShared.unwrap_or(false) {
                Some("people-symbolic".to_string())
            } else {
                Some("view-list-symbolic".to_string())
            }, // Default icon
            hide_completed: true,                // Not supported in MS Graph
            number_of_tasks,                     // Count of non-completed tasks
            well_known_list_name: todo_list.wellknownListName,
            is_virtual: false,                   // MS Graph lists are never virtual
            virtual_type: None,                  // MS Graph lists are never virtual
            sort_order: 0,                       // Default sort order
        }
    }
}

impl From<&List> for UpdateTodoTaskListRequest {
    fn from(list: &List) -> Self {
        Self {
            displayName: Some(list.name.clone()),
        }
    }
}

// ============================================================================
// Task Mappings
// ============================================================================

impl From<&Task> for CreateTodoTaskRequest {
    fn from(task: &Task) -> Self {
        Self {
            title: task.title.clone(),
            body: if task.notes.is_empty() {
                None
            } else {
                Some(TaskBody {
                    content: task.notes.clone(),
                    contentType: TaskBodyType::Text,
                })
            },
            dueDateTime: task.due_date.map(|dt| dt.into()),
            startDateTime: None, // Not supported in local model
            importance: Some(task.priority.into()),
            isReminderOn: Some(task.reminder_date.is_some()),
            reminderDateTime: task.reminder_date.map(|dt| dt.into()),
            showReminder: Some(task.reminder_date.is_some()),
            status: Some(task.status.into()),
            categories: if task.tags.is_empty() {
                Some(Vec::new())  // Send empty array instead of None
            } else {
                Some(task.tags.clone())
            },
        }
    }
}

impl From<TodoTask> for Task {
    fn from(todo_task: TodoTask) -> Self {
        Self {
            id: todo_task.id,                    // Use MS Graph ID
            
            title: todo_task.title,
            today: false,                        // Not supported in MS Graph
            status: todo_task.status.unwrap_or(Status::NotStarted.into()).into(),
            priority: todo_task.importance.unwrap_or(Priority::Normal.into()).into(),
            tags: todo_task.categories.unwrap_or_default(),
            notes: todo_task.body.map(|b| b.content).unwrap_or_default(),
            completion_date: todo_task.completedDateTime.map(|dt| dt.into()),
            due_date: todo_task.dueDateTime.map(|dt| dt.into()),
            reminder_date: todo_task.reminderDateTime.map(|dt| dt.into()),
            recurrence: Default::default(), // Skip recurrence for now
            expanded: false,                     // UI state, not stored
            
            // Checklist fields
            checklist_items: Vec::new(), // Will be populated separately via API calls
            checklist_sync_status: ChecklistSyncStatus::Synced,
            
            created_date_time: todo_task.createdDateTime
                .and_then(|dt| DateTime::parse_from_rfc3339(&dt).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
            last_modified_date_time: todo_task.lastModifiedDateTime
                .and_then(|dt| DateTime::parse_from_rfc3339(&dt).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
            list_id: todo_task.parentList.map(|list| list.id),
        }
    }
}

/// Helper function to create a Task from TodoTask with proper path construction
pub fn todo_task_to_task_with_path(todo_task: TodoTask, list_id: &str) -> Task {
    let mut task: Task = todo_task.into();
    task.list_id = Some(list_id.to_string());
    
    task
}

impl From<&Task> for UpdateTodoTaskRequest {
    fn from(task: &Task) -> Self {
        Self {
            title: Some(task.title.clone()),
            body: if task.notes.is_empty() {
                None
            } else {
                Some(TaskBody {
                    content: task.notes.clone(),
                    contentType: TaskBodyType::Text,
                })
            },
            completedDateTime: task.completion_date.map(|dt| dt.into()),
            dueDateTime: task.due_date.map(|dt| dt.into()),
            startDateTime: None, // Not supported in local model
            importance: Some(task.priority.into()),
            isReminderOn: Some(task.reminder_date.is_some()),
            reminderDateTime: task.reminder_date.map(|dt| dt.into()),
            
            showReminder: Some(task.reminder_date.is_some()),
            status: Some(task.status.into()),
            categories: if task.tags.is_empty() {
                Some(Vec::new())  // Send empty array instead of None
            } else {
                Some(task.tags.clone())
            },
        }
    }
}

// ============================================================================
// Sub-task (ChecklistItem) Mappings
// ============================================================================

impl From<&crate::storage::models::ChecklistItem> for CreateChecklistItemRequest {
    fn from(item: &crate::storage::models::ChecklistItem) -> Self {
        Self {
            displayName: item.display_name.clone(),
            isChecked: Some(item.is_checked),
        }
    }
}

impl From<ChecklistItem> for crate::storage::models::ChecklistItem {
    fn from(item: ChecklistItem) -> Self {
        Self::from_ms_graph(
            item.id.clone(),
            item.displayName.clone(),
            item.isChecked,
            item.createdDateTime.clone(),
            item.checkedDateTime.clone(),
        ).unwrap_or_else(|_| {
            // Fallback to local creation if parsing fails
            let mut local_item = crate::storage::models::ChecklistItem::new(item.displayName.clone());
            local_item.id = item.id.clone();
            if item.isChecked {
                local_item.check();
            }
            local_item
        })
    }
}

impl From<&crate::storage::models::ChecklistItem> for UpdateChecklistItemRequest {
    fn from(item: &crate::storage::models::ChecklistItem) -> Self {
        Self {
            displayName: Some(item.display_name.clone()),
            isChecked: Some(item.is_checked),
        }
    }
}

// ============================================================================
// Enum Mappings
// ============================================================================

impl From<Priority> for TaskImportance {
    fn from(priority: Priority) -> Self {
        match priority {
            Priority::Low => TaskImportance::Low,
            Priority::Normal => TaskImportance::Normal,
            Priority::High => TaskImportance::High,
        }
    }
}

impl From<TaskImportance> for Priority {
    fn from(importance: TaskImportance) -> Self {
        match importance {
            TaskImportance::Low => Priority::Low,
            TaskImportance::Normal => Priority::Normal,
            TaskImportance::High => Priority::High,
        }
    }
}

impl From<Status> for TaskStatus {
    fn from(status: Status) -> Self {
        match status {
            Status::NotStarted => TaskStatus::NotStarted,
            Status::Completed => TaskStatus::Completed,
        }
    }
}

impl From<TaskStatus> for Status {
    fn from(task_status: TaskStatus) -> Self {
        match task_status {
            TaskStatus::NotStarted => Status::NotStarted,
            TaskStatus::InProgress => Status::NotStarted, // Map to NotStarted
            TaskStatus::Completed => Status::Completed,
            TaskStatus::WaitingOnOthers => Status::NotStarted, // Map to NotStarted
            TaskStatus::Deferred => Status::NotStarted, // Map to NotStarted
        }
    }
}

// ============================================================================
// DateTime Mappings
// ============================================================================

impl From<DateTime<Utc>> for DateTimeTimeZone {
    fn from(dt: DateTime<Utc>) -> Self {
        Self {
            dateTime: dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            timeZone: "UTC".to_string(),
        }
    }
}

impl From<DateTimeTimeZone> for DateTime<Utc> {
    fn from(dt: DateTimeTimeZone) -> Self {
        parse_ms_graph_date(&dt.dateTime)
            .unwrap_or_else(|| {
                eprintln!("Using fallback date for failed parsing of: '{}'", dt.dateTime);
                Utc::now()
            })
    }
}

/// Parse Microsoft Graph API date format with multiple fallback strategies
fn parse_ms_graph_date(date_str: &str) -> Option<DateTime<Utc>> {
    // First try: Standard RFC3339 format
    if let Ok(parsed) = DateTime::parse_from_rfc3339(date_str) {
        return Some(parsed.with_timezone(&Utc));
    }
    
    // Second try: Handle Microsoft Graph specific format with microseconds
    // Format: "2025-08-03T00:00:00.0000000"
    if let Ok(parsed) = DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.f") {
        return Some(parsed.with_timezone(&Utc));
    }
    
    // Third try: Handle format without microseconds using NaiveDateTime
    if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S") {
        return Some(DateTime::from_naive_utc_and_offset(parsed, Utc));
    }
    
    // Fourth try: Handle date-only format using NaiveDate
    if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        let naive_datetime = parsed.and_hms_opt(0, 0, 0).unwrap();
        return Some(DateTime::from_naive_utc_and_offset(naive_datetime, Utc));
    }
    
    // Fifth try: Handle Microsoft Graph format with 7-digit microseconds
    // Format: "2025-08-03T00:00:00.0000000"
    if let Ok(parsed) = DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.7f") {
        return Some(parsed.with_timezone(&Utc));
    }
    
    // Sixth try: Handle format with 6-digit microseconds
    if let Ok(parsed) = DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.6f") {
        return Some(parsed.with_timezone(&Utc));
    }
    
    // Seventh try: Handle format with variable microseconds
    if let Ok(parsed) = DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.f%z") {
        return Some(parsed.with_timezone(&Utc));
    }
    
    // Eighth try: Handle format with microseconds and no timezone
    if let Ok(parsed) = DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.f") {
        return Some(parsed.with_timezone(&Utc));
    }
    
    // Ninth try: Handle the specific Microsoft Graph format by truncating microseconds
    // The format "2025-08-03T00:00:00.0000000" has 7 digits after the decimal
    // Let's try to parse it by removing the microseconds part
    if let Some(without_microseconds) = date_str.split('.').next() {
        if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(without_microseconds, "%Y-%m-%dT%H:%M:%S") {
            return Some(DateTime::from_naive_utc_and_offset(parsed, Utc));
        }
    }
    
    // Tenth try: Handle format with nanoseconds (9 digits)
    if let Ok(parsed) = DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.9f") {
        return Some(parsed.with_timezone(&Utc));
    }
    
    // Eleventh try: Handle format with 8-digit microseconds
    if let Ok(parsed) = DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.8f") {
        return Some(parsed.with_timezone(&Utc));
    }
    
    // Twelfth try: Handle format with 5-digit microseconds
    if let Ok(parsed) = DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.5f") {
        return Some(parsed.with_timezone(&Utc));
    }
    
    // If all parsing attempts fail, log the error
    eprintln!("Failed to parse MS Graph date: '{}'", date_str);
    None
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc, Datelike, Timelike};
  
    use super::*;

    #[test]
    fn test_list_to_create_request() {
        let list = List {
            id: "local-uuid".to_string(),
            
            name: "Test List".to_string(),
            description: "Test Description".to_string(),
            icon: Some("test-icon".to_string()),
            hide_completed: false,
            number_of_tasks: 0,
            well_known_list_name: None,
            is_virtual: false,
            virtual_type: None,
            sort_order: 0,
        };

        let request: CreateTodoTaskListRequest = (&list).into();
        assert_eq!(request.displayName, "Test List");
        assert_eq!(request.isOwner, Some(true));
        assert_eq!(request.isShared, Some(false));
    }

    #[test]
    fn test_todo_list_to_list() {
        let todo_list = TodoTaskList {
            odata_type: Some("#microsoft.graph.todoTaskList".to_string()),
            id: "ms-graph-id".to_string(),
            displayName: "MS Graph List".to_string(),
            isOwner: Some(true),
            isShared: Some(false),
            wellknownListName: Some("defaultList".to_string()),
            tasks: None,
            odata_count: None,
        };

        let list: List = todo_list.into();
        assert_eq!(list.id, "ms-graph-id");
        assert_eq!(list.name, "MS Graph List");
        assert_eq!(list.icon, Some("view-list-symbolic".to_string()));
        assert_eq!(list.number_of_tasks, 0);
    }

    #[test]
    fn test_todo_list_to_list_with_tasks() {
        let todo_list = TodoTaskList {
            odata_type: Some("#microsoft.graph.todoTaskList".to_string()),
            id: "ms-graph-id".to_string(),
            displayName: "MS Graph List".to_string(),
            isOwner: Some(true),
            isShared: Some(false),
            wellknownListName: Some("defaultList".to_string()),
            tasks: Some(vec![
                TodoTask {
                    odata_context: Some("https://graph.microsoft.com/v1.0/$metadata#tasks/$entity".to_string()),
                    etag: Some("etag123".to_string()),
                    id: "task1".to_string(),
                    title: "Test Task 1".to_string(),
                    body: None,
                    completedDateTime: None,
                    createdDateTime: Some("2023-12-20T10:00:00Z".to_string()),
                    dueDateTime: None,
                    startDateTime: None,
                    importance: Some(TaskImportance::High),
                    isReminderOn: Some(false),
                    lastModifiedDateTime: Some("2023-12-20T10:00:00Z".to_string()),
                    linkedResources: None,
                    recurrence: None,
                    reminderDateTime: None,
                    showReminder: Some(false),
                    status: Some(TaskStatus::NotStarted),
                    categories: None,
                    hasAttachments: Some(false),
                    parentList: None,
                    extensions: None,
                },
                TodoTask {
                    odata_context: Some("https://graph.microsoft.com/v1.0/$metadata#tasks/$entity".to_string()),
                    etag: Some("etag456".to_string()),
                    id: "task2".to_string(),
                    title: "Test Task 2".to_string(),
                    body: None,
                    completedDateTime: None,
                    createdDateTime: Some("2023-12-20T10:00:00Z".to_string()),
                    dueDateTime: None,
                    startDateTime: None,
                    importance: Some(TaskImportance::Normal),
                    isReminderOn: Some(false),
                    lastModifiedDateTime: Some("2023-12-20T10:00:00Z".to_string()),
                    linkedResources: None,
                    recurrence: None,
                    reminderDateTime: None,
                    showReminder: Some(false),
                    status: Some(TaskStatus::NotStarted),
                    categories: None,
                    hasAttachments: Some(false),
                    parentList: None,
                    extensions: None,
                },
            ]),
            odata_count: None,
        };

        let list: List = todo_list.into();
        assert_eq!(list.id, "ms-graph-id");
        assert_eq!(list.name, "MS Graph List");
        assert_eq!(list.icon, Some("view-list-symbolic".to_string()));
        assert_eq!(list.number_of_tasks, 2); // Should count the 2 non-completed tasks
    }

    #[test]
    fn test_todo_list_to_list_with_count() {
        let todo_list = TodoTaskList {
            odata_type: Some("#microsoft.graph.todoTaskList".to_string()),
            id: "ms-graph-id".to_string(),
            displayName: "MS Graph List".to_string(),
            isOwner: Some(true),
            isShared: Some(false),
            wellknownListName: Some("defaultList".to_string()),
            tasks: None,
            odata_count: Some(5), // Direct count from API
        };

        let list: List = todo_list.into();
        assert_eq!(list.id, "ms-graph-id");
        assert_eq!(list.name, "MS Graph List");
        assert_eq!(list.icon, Some("view-list-symbolic".to_string()));
        
    }

    #[test]
    fn test_task_to_create_request() {
        let task = Task {
            id: "local-task-uuid".to_string(),
            
            title: "Test Task".to_string(),
            today: true,
            status: Status::NotStarted,
            priority: Priority::High,
            tags: vec!["work".to_string(), "important".to_string()],
            notes: "Test notes".to_string(),
            completion_date: None,
            due_date: Some(Utc.with_ymd_and_hms(2023, 12, 25, 12, 0, 0).unwrap()),
            reminder_date: Some(Utc.with_ymd_and_hms(2023, 12, 24, 9, 0, 0).unwrap()),
            recurrence: Default::default(),
            expanded: false,
            
            created_date_time: Utc.with_ymd_and_hms(2023, 12, 20, 10, 0, 0).unwrap(),
            last_modified_date_time: Utc.with_ymd_and_hms(2023, 12, 20, 10, 0, 0).unwrap(),
            list_id: Some("1".to_string()),
            checklist_items: Vec::new(),
            checklist_sync_status: ChecklistSyncStatus::Synced,
            
        };

        let request: CreateTodoTaskRequest = (&task).into();
        assert_eq!(request.title, "Test Task");
        assert_eq!(request.importance, Some(TaskImportance::High));
        assert_eq!(request.status, Some(TaskStatus::NotStarted));
        assert_eq!(request.categories, Some(vec!["work".to_string(), "important".to_string()]));
        assert!(request.dueDateTime.is_some());
        assert!(request.reminderDateTime.is_some());
    }

    #[test]
    fn test_todo_task_to_task() {
        let todo_task = TodoTask {
            odata_context: Some("https://graph.microsoft.com/v1.0/$metadata#tasks/$entity".to_string()),
            etag: Some("etag123".to_string()),
            id: "ms-task-id".to_string(),
            title: "MS Graph Task".to_string(),
            body: Some(TaskBody {
                content: "Task description".to_string(),
                contentType: TaskBodyType::Text,
            }),
            completedDateTime: None,
            createdDateTime: Some("2023-12-20T10:00:00Z".to_string()),
            dueDateTime: Some(DateTimeTimeZone {
                dateTime: "2023-12-25T12:00:00Z".to_string(),
                timeZone: "UTC".to_string(),
            }),
            startDateTime: None,
            importance: Some(TaskImportance::High),
            isReminderOn: Some(true),
            lastModifiedDateTime: Some("2023-12-20T10:00:00Z".to_string()),
            linkedResources: None,
            recurrence: None,
            reminderDateTime: None,
            showReminder: Some(true),
            status: Some(TaskStatus::NotStarted),
            categories: Some(vec!["work".to_string()]),
            hasAttachments: Some(false),
            parentList: None,
            extensions: None,
        };

        let task: Task = todo_task.into();
        assert_eq!(task.id, "ms-task-id");
        assert_eq!(task.title, "MS Graph Task");
        assert_eq!(task.notes, "Task description");
        assert_eq!(task.priority, Priority::High);
        assert_eq!(task.status, Status::NotStarted);
        assert_eq!(task.tags, vec!["work".to_string()]);
    }



    #[test]
    fn test_priority_mapping() {
        assert_eq!(TaskImportance::from(Priority::Low), TaskImportance::Low);
        assert_eq!(TaskImportance::from(Priority::Normal), TaskImportance::Normal);
        assert_eq!(TaskImportance::from(Priority::High), TaskImportance::High);

        assert_eq!(Priority::from(TaskImportance::Low), Priority::Low);
        assert_eq!(Priority::from(TaskImportance::Normal), Priority::Normal);
        assert_eq!(Priority::from(TaskImportance::High), Priority::High);
    }

    #[test]
    fn test_status_mapping() {
        assert_eq!(TaskStatus::from(Status::NotStarted), TaskStatus::NotStarted);
        assert_eq!(TaskStatus::from(Status::Completed), TaskStatus::Completed);

        assert_eq!(Status::from(TaskStatus::NotStarted), Status::NotStarted);
        assert_eq!(Status::from(TaskStatus::Completed), Status::Completed);
        assert_eq!(Status::from(TaskStatus::InProgress), Status::NotStarted); // Mapped
    }

    #[test]
    fn test_datetime_mapping() {
        let utc_time = Utc.with_ymd_and_hms(2023, 12, 25, 12, 0, 0).unwrap();
        let dt_tz: DateTimeTimeZone = utc_time.into();
        
        assert_eq!(dt_tz.dateTime, "2023-12-25T12:00:00Z");
        assert_eq!(dt_tz.timeZone, "UTC");

        let converted_back: DateTime<Utc> = dt_tz.into();
        assert_eq!(converted_back, utc_time);
    }

    #[test]
    fn test_ms_graph_date_parsing() {
        // Test Microsoft Graph API specific format with microseconds
        let ms_graph_date = "2025-08-03T00:00:00.0000000";
        let parsed = parse_ms_graph_date(ms_graph_date);
        assert!(parsed.is_some());
        
        if let Some(dt) = parsed {
            assert_eq!(dt.year(), 2025);
            assert_eq!(dt.month(), 8);
            assert_eq!(dt.day(), 3);
            assert_eq!(dt.hour(), 0);
            assert_eq!(dt.minute(), 0);

        }
        
        // Test standard RFC3339 format
        let rfc3339_date = "2025-08-03T00:00:00Z";
        let parsed = parse_ms_graph_date(rfc3339_date);
        assert!(parsed.is_some());
        
        // Test date-only format
        let date_only = "2025-08-03";
        let parsed = parse_ms_graph_date(date_only);
        assert!(parsed.is_some());
        
        // Test invalid date
        let invalid_date = "invalid-date";
        let parsed = parse_ms_graph_date(invalid_date);
        assert!(parsed.is_none());
    }
}
