pub mod http_client;
pub mod models;
pub mod mapping;

#[cfg(test)]
mod tests {
    use super::models::*;

    #[test]
    fn test_basic_models() {
        // Test that we can create basic models matching Microsoft Graph API
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

        // Test that we can create a task matching Microsoft Graph API
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
            categories: Some(vec!["Important".to_string()]),
            hasAttachments: Some(false),
            parentList: None,
            extensions: None,
        };

        assert_eq!(task.title, "Test Task");
        assert_eq!(task.importance, Some(TaskImportance::High));
        assert_eq!(task.status, Some(TaskStatus::NotStarted));

        // Test that we can create checklist items matching Microsoft Graph API
        let checklist_item = ChecklistItem {
            id: "checklist1".to_string(),
            displayName: "Test Checklist Item".to_string(),
            isChecked: false,
            createdDateTime: "2023-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(checklist_item.displayName, "Test Checklist Item");
        assert_eq!(checklist_item.id, "checklist1");
        assert_eq!(checklist_item.isChecked, false);
    }
}
