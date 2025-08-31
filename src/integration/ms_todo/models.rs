use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Todo Lists (todoTaskList)
// ============================================================================

/// Represents a Microsoft Todo list (todoTaskList)
/// Based on: https://learn.microsoft.com/en-us/graph/api/todotasklist-get?view=graph-rest-1.0&tabs=http
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct TodoTaskList {
    #[serde(rename = "@odata.type")]
    pub odata_type: Option<String>,
    pub id: String,
    pub displayName: String,
    pub isOwner: Option<bool>,
    pub isShared: Option<bool>,
    pub wellknownListName: Option<String>,
    /// Tasks when using $expand=tasks
    pub tasks: Option<Vec<TodoTask>>,
    /// Count of non-completed tasks when using $count=true
    #[serde(rename = "@odata.count")]
    pub odata_count: Option<u32>,
}

/// Collection of todo lists
#[derive(Debug, Deserialize, Serialize)]
pub struct TodoTaskListCollection {
    #[serde(rename = "@odata.context")]
    pub context: Option<String>,
    pub value: Vec<TodoTaskList>,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}

// ============================================================================
// Todo Tasks (todoTask)
// ============================================================================

/// Represents a Microsoft Todo task (todoTask)
/// Based on: https://learn.microsoft.com/en-us/graph/api/todotask-get?view=graph-rest-1.0&tabs=http
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct TodoTask {
    #[serde(rename = "@odata.context")]
    pub odata_context: Option<String>,
    #[serde(rename = "@odata.etag")]
    pub etag: Option<String>,
    pub id: String,
    pub title: String,
    pub body: Option<TaskBody>,
    pub completedDateTime: Option<DateTimeTimeZone>,
    pub createdDateTime: Option<String>,
    pub dueDateTime: Option<DateTimeTimeZone>,
    pub startDateTime: Option<DateTimeTimeZone>,
    pub importance: Option<TaskImportance>,
    pub isReminderOn: Option<bool>,
    pub lastModifiedDateTime: Option<String>,
    pub linkedResources: Option<Vec<LinkedResource>>,
    pub recurrence: Option<PatternedRecurrence>,
    pub reminderDateTime: Option<DateTimeTimeZone>,
    pub showReminder: Option<bool>,
    pub status: Option<TaskStatus>,
    pub categories: Option<Vec<String>>,
    pub hasAttachments: Option<bool>,
    pub parentList: Option<TodoTaskList>,
    pub extensions: Option<Vec<Extension>>,
}

/// Task body content
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct TaskBody {
    pub content: String,
    pub contentType: TaskBodyType,
}

/// Task body content type
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum TaskBodyType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "html")]
    Html,
}

/// Date and time with timezone information
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct DateTimeTimeZone {
    pub dateTime: String,
    pub timeZone: String,
}

/// Task importance level
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum TaskImportance {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "high")]
    High,
}

/// Task status
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum TaskStatus {
    #[serde(rename = "notStarted")]
    NotStarted,
    #[serde(rename = "inProgress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "waitingOnOthers")]
    WaitingOnOthers,
    #[serde(rename = "deferred")]
    Deferred,
}

/// Collection of todo tasks
#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct TodoTaskCollection {
    #[serde(rename = "@odata.context")]
    pub context: Option<String>,
    pub value: Vec<TodoTask>,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}

// ============================================================================
// Linked Resources
// ============================================================================

/// Represents a linked resource (file, link, etc.)
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct LinkedResource {
    pub id: String,
    pub webUrl: Option<String>,
    pub applicationName: Option<String>,
    pub displayName: Option<String>,
    pub externalId: Option<String>,
    pub properties: Option<HashMap<String, serde_json::Value>>,
}

// ============================================================================
// Recurrence
// ============================================================================

/// Patterned recurrence for recurring tasks
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PatternedRecurrence {
    pub pattern: RecurrencePattern,
    pub range: RecurrenceRange,
}

/// Recurrence pattern
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct RecurrencePattern {
    #[serde(rename = "type")]
    pub pattern_type: RecurrencePatternType,
    pub interval: Option<i32>,
    pub month: Option<i32>,
    pub dayOfMonth: Option<i32>,
    pub daysOfWeek: Option<Vec<DayOfWeek>>,
    pub firstDayOfWeek: Option<DayOfWeek>,
    pub index: Option<WeekIndex>,
}

/// Recurrence pattern type
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum RecurrencePatternType {
    #[serde(rename = "daily")]
    Daily,
    #[serde(rename = "weekly")]
    Weekly,
    #[serde(rename = "absoluteMonthly")]
    AbsoluteMonthly,
    #[serde(rename = "relativeMonthly")]
    RelativeMonthly,
    #[serde(rename = "absoluteYearly")]
    AbsoluteYearly,
    #[serde(rename = "relativeYearly")]
    RelativeYearly,
}

/// Days of the week
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum DayOfWeek {
    #[serde(rename = "sunday")]
    Sunday,
    #[serde(rename = "monday")]
    Monday,
    #[serde(rename = "tuesday")]
    Tuesday,
    #[serde(rename = "wednesday")]
    Wednesday,
    #[serde(rename = "thursday")]
    Thursday,
    #[serde(rename = "friday")]
    Friday,
    #[serde(rename = "saturday")]
    Saturday,
}

/// Week index for relative patterns
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum WeekIndex {
    #[serde(rename = "first")]
    First,
    #[serde(rename = "second")]
    Second,
    #[serde(rename = "third")]
    Third,
    #[serde(rename = "fourth")]
    Fourth,
    #[serde(rename = "last")]
    Last,
}

/// Recurrence range
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct RecurrenceRange {
    #[serde(rename = "type")]
    pub range_type: RecurrenceRangeType,
    pub startDate: Option<String>,
    pub endDate: Option<String>,
    pub numberOfOccurrences: Option<i32>,
}

/// Recurrence range type
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum RecurrenceRangeType {
    #[serde(rename = "endDate")]
    EndDate,
    #[serde(rename = "noEnd")]
    NoEnd,
    #[serde(rename = "numbered")]
    Numbered,
}

// ============================================================================
// Extensions
// ============================================================================

/// Extension for additional task properties
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Extension {
    pub id: String,
    #[serde(rename = "@odata.type")]
    pub odata_type: Option<String>,
    #[serde(flatten)]
    pub additional_data: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Checklist Items (checklistItem)
// ============================================================================

/// Represents a Microsoft Todo checklist item (checklistItem)
/// Based on: https://learn.microsoft.com/en-us/graph/api/resources/checklistitem?view=graph-rest-1.0
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct ChecklistItem {
    pub id: String,
    pub displayName: String,
    pub isChecked: bool,
    pub createdDateTime: String,
    pub checkedDateTime: Option<String>,
}

/// Collection of checklist items
#[derive(Debug, Deserialize, Serialize)]
pub struct ChecklistItemCollection {
    #[serde(rename = "@odata.context")]
    pub context: Option<String>,
    pub value: Vec<ChecklistItem>,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}

/// Request model for creating a new checklist item
#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct CreateChecklistItemRequest {
    pub displayName: String,
    pub isChecked: Option<bool>,
}

/// Request model for updating a checklist item
#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct UpdateChecklistItemRequest {
    pub displayName: Option<String>,
    pub isChecked: Option<bool>,
}

// ============================================================================
// Request/Response Models
// ============================================================================

/// Request model for creating a new todo list
#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct CreateTodoTaskListRequest {
    pub displayName: String,
    pub isOwner: Option<bool>,
    pub isShared: Option<bool>,
}

/// Request model for updating a todo list
#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct UpdateTodoTaskListRequest {
    pub displayName: Option<String>,
}

/// Request model for creating a new todo task
#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct CreateTodoTaskRequest {
    pub title: String,
    pub body: Option<TaskBody>,
    pub dueDateTime: Option<DateTimeTimeZone>,
    pub startDateTime: Option<DateTimeTimeZone>,
    pub importance: Option<TaskImportance>,
    pub isReminderOn: Option<bool>,
    pub reminderDateTime: Option<DateTimeTimeZone>,
    pub showReminder: Option<bool>,
    pub status: Option<TaskStatus>,
    pub categories: Option<Vec<String>>,
}

/// Request model for updating a todo task
#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct UpdateTodoTaskRequest {
    pub title: Option<String>,
    pub body: Option<TaskBody>,
    pub completedDateTime: Option<DateTimeTimeZone>,
    pub dueDateTime: Option<DateTimeTimeZone>,
    pub startDateTime: Option<DateTimeTimeZone>,
    pub importance: Option<TaskImportance>,
    pub isReminderOn: Option<bool>,
    pub reminderDateTime: Option<DateTimeTimeZone>,
    pub showReminder: Option<bool>,
    pub status: Option<TaskStatus>,
    pub categories: Option<Vec<String>>,
}

// ============================================================================
// Error Models
// ============================================================================

/// Microsoft Graph API error response
#[derive(Debug, Deserialize, Serialize)]
pub struct GraphError {
    pub error: GraphErrorDetail,
}

/// Detailed error information
#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct GraphErrorDetail {
    pub code: String,
    pub message: String,
    pub innerError: Option<GraphInnerError>,
}

/// Inner error details
#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct GraphInnerError {
    pub code: String,
    pub message: String,
    pub innerError: Option<Box<GraphInnerError>>,
}

// ============================================================================
// Utility Models
// ============================================================================

/// Result wrapper for API operations
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TodoApiResult<T> {
    pub data: Option<T>,
    pub error: Option<String>,
    pub success: bool,
}

impl<T> TodoApiResult<T> {
    #[allow(dead_code)]
    pub fn success(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
            success: true,
        }
    }

    #[allow(dead_code)]
    pub fn error(error: String) -> Self {
        Self {
            data: None,
            error: Some(error),
            success: false,
        }
    }

    #[allow(dead_code)]
    pub fn is_success(&self) -> bool {
        self.success
    }

    #[allow(dead_code)]
    pub fn is_error(&self) -> bool {
        !self.success
    }
}

// ============================================================================
// Default Implementations
// ============================================================================

impl Default for TaskBodyType {
    fn default() -> Self {
        TaskBodyType::Text
    }
}

impl Default for TaskImportance {
    fn default() -> Self {
        TaskImportance::Normal
    }
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::NotStarted
    }
}

impl Default for RecurrencePatternType {
    fn default() -> Self {
        RecurrencePatternType::Daily
    }
}

impl Default for DayOfWeek {
    fn default() -> Self {
        DayOfWeek::Monday
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todo_list_creation() {
        let list = TodoTaskList {
            odata_type: Some("#microsoft.graph.todoTaskList".to_string()),
            id: "list1".to_string(),
            displayName: "Test List".to_string(),
            isOwner: Some(true),
            isShared: Some(false),
            wellknownListName: Some("defaultList".to_string()),
            tasks: None,
            odata_count: None,
        };

        assert_eq!(list.displayName, "Test List");
        assert_eq!(list.id, "list1");
        assert_eq!(list.isOwner, Some(true));
    }

    #[test]
    fn test_todo_task_creation() {
        let task = TodoTask {
            odata_context: Some("https://graph.microsoft.com/v1.0/$metadata#tasks/$entity".to_string()),
            etag: Some("etag456".to_string()),
            id: "task1".to_string(),
            title: "Test Task".to_string(),
            body: Some(TaskBody {
                content: "Task description".to_string(),
                contentType: TaskBodyType::Text,
            }),
            completedDateTime: None,
            createdDateTime: Some("2023-01-01T00:00:00Z".to_string()),
            dueDateTime: None,
            startDateTime: None,
            importance: Some(TaskImportance::High),
            isReminderOn: Some(false),
            lastModifiedDateTime: Some("2023-01-01T00:00:00Z".to_string()),
            linkedResources: None,
            recurrence: None,
            reminderDateTime: None,
            showReminder: Some(false),
            status: Some(TaskStatus::NotStarted),
            categories: Some(vec!["work".to_string()]),
            hasAttachments: Some(false),
            parentList: None,
            extensions: None,
        };

        assert_eq!(task.title, "Test Task");
        assert_eq!(task.importance, Some(TaskImportance::High));
        assert_eq!(task.status, Some(TaskStatus::NotStarted));
    }

    #[test]
    fn test_task_body_serialization() {
        let body = TaskBody {
            content: "Test content".to_string(),
            contentType: TaskBodyType::Html,
        };

        let serialized = serde_json::to_string(&body).unwrap();
        assert!(serialized.contains("Test content"));
        assert!(serialized.contains("html"));
    }

    #[test]
    fn test_todo_api_result() {
        let success_result = TodoApiResult::success("test data".to_string());
        assert!(success_result.is_success());
        assert!(!success_result.is_error());
        assert_eq!(success_result.data, Some("test data".to_string()));
        assert_eq!(success_result.error, None);

        let error_result = TodoApiResult::<String>::error("test error".to_string());
        assert!(!error_result.is_success());
        assert!(error_result.is_error());
        assert_eq!(error_result.data, None);
        assert_eq!(error_result.error, Some("test error".to_string()));
    }

    #[test]
    fn test_enum_defaults() {
        assert_eq!(TaskBodyType::default(), TaskBodyType::Text);
        assert_eq!(TaskImportance::default(), TaskImportance::Normal);
        assert_eq!(TaskStatus::default(), TaskStatus::NotStarted);
        assert_eq!(RecurrencePatternType::default(), RecurrencePatternType::Daily);
        assert_eq!(DayOfWeek::default(), DayOfWeek::Monday);
    }

    #[test]
    fn test_checklist_item_creation() {
        let item = ChecklistItem {
            id: "item1".to_string(),
            displayName: "Test Checklist Item".to_string(),
            isChecked: false,
            createdDateTime: "2023-01-01T00:00:00Z".to_string(),
            checkedDateTime: None,
        };

        assert_eq!(item.displayName, "Test Checklist Item");
        assert_eq!(item.id, "item1");
        assert_eq!(item.isChecked, false);
    }

    #[test]
    fn test_checklist_item_collection() {
        let items = vec![
            ChecklistItem {
                id: "item1".to_string(),
                displayName: "Item 1".to_string(),
                isChecked: false,
                createdDateTime: "2023-01-01T00:00:00Z".to_string(),
                checkedDateTime: None,
            },
            ChecklistItem {
                id: "item2".to_string(),
                displayName: "Item 2".to_string(),
                isChecked: true,
                createdDateTime: "2023-01-01T00:00:00Z".to_string(),
                checkedDateTime: None,
            },
        ];

        let collection = ChecklistItemCollection {
            context: Some("https://graph.microsoft.com/v1.0/$metadata#checklistItems".to_string()),
            value: items,
            next_link: None,
        };

        assert_eq!(collection.value.len(), 2);
        assert_eq!(collection.value[0].isChecked, false);
        assert_eq!(collection.value[1].isChecked, true);
    }

    #[test]
    fn test_create_checklist_item_request() {
        let request = CreateChecklistItemRequest {
            displayName: "New Item".to_string(),
            isChecked: Some(false),
        };

        assert_eq!(request.displayName, "New Item");
        assert_eq!(request.isChecked, Some(false));
    }

    #[test]
    fn test_update_checklist_item_request() {
        let request = UpdateChecklistItemRequest {
            displayName: Some("Updated Item".to_string()),
            isChecked: Some(true),
        };

        assert_eq!(request.displayName, Some("Updated Item".to_string()));
        assert_eq!(request.isChecked, Some(true));
    }
}
