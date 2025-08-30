use crate::integration::ms_todo::{
    http_client::MsTodoHttpClient,
    models::{
        ChecklistItem, ChecklistItemCollection, CreateChecklistItemRequest,
        CreateTodoTaskListRequest, CreateTodoTaskRequest, TodoTask, TodoTaskList,
        TodoTaskListCollection, TodoTaskCollection, UpdateChecklistItemRequest,
        UpdateTodoTaskListRequest, UpdateTodoTaskRequest,
    },
};
use anyhow::{Context, Result};

/// Microsoft Graph Todo API client
#[derive(Debug, Clone)]
pub struct MsTodoClient {
    http_client: MsTodoHttpClient,
    auth_header: String,
    list_id: String,
}

impl MsTodoClient {
    /// Create a new MS Todo client
    pub fn new(auth_header: String, list_id: String) -> Self {
        Self {
            http_client: MsTodoHttpClient::new(),
            auth_header,
            list_id,
        }
    }

    // ============================================================================
    // Todo Lists Operations
    // ============================================================================

    /// Get all todo lists
    pub async fn get_lists(&self) -> Result<Vec<TodoTaskList>> {
        let response: TodoTaskListCollection = self
            .http_client
            .get("/me/todo/lists", &self.auth_header)
            .await?;
        Ok(response.value)
    }

    /// Create a new todo list
    pub async fn create_list(&self, request: CreateTodoTaskListRequest) -> Result<TodoTaskList> {
        self.http_client
            .post("/me/todo/lists", &request, &self.auth_header)
            .await
    }

    /// Update a todo list
    pub async fn update_list(&self, list_id: &str, request: UpdateTodoTaskListRequest) -> Result<TodoTaskList> {
        let url = format!("/me/todo/lists/{}", list_id);
        self.http_client
            .patch(&url, &request, &self.auth_header)
            .await
    }

    /// Delete a todo list
    pub async fn delete_list(&self, list_id: &str) -> Result<()> {
        let url = format!("/me/todo/lists/{}", list_id);
        self.http_client.delete(&url, &self.auth_header).await
    }

    // ============================================================================
    // Todo Tasks Operations
    // ============================================================================

    /// Get all tasks in a list
    pub async fn get_tasks(&self) -> Result<Vec<TodoTask>> {
        let url = format!("/me/todo/lists/{}/tasks", self.list_id);
        let response: TodoTaskCollection = self
            .http_client
            .get(&url, &self.auth_header)
            .await?;
        Ok(response.value)
    }

    /// Create a new task
    pub async fn create_task(&self, request: CreateTodoTaskRequest) -> Result<TodoTask> {
        let url = format!("/me/todo/lists/{}/tasks", self.list_id);
        self.http_client
            .post(&url, &request, &self.auth_header)
            .await
    }

    /// Update a task
    pub async fn update_task(&self, task_id: &str, request: UpdateTodoTaskRequest) -> Result<TodoTask> {
        let url = format!("/me/todo/lists/{}/tasks/{}", self.list_id, task_id);
        self.http_client
            .patch(&url, &request, &self.auth_header)
            .await
    }

    /// Delete a task
    pub async fn delete_task(&self, task_id: &str) -> Result<()> {
        let url = format!("/me/todo/lists/{}/tasks/{}", self.list_id, task_id);
        self.http_client.delete(&url, &self.auth_header).await
    }

    // ============================================================================
    // Checklist Items Operations
    // ============================================================================

    /// Get all checklist items for a task
    pub async fn get_checklist_items(&self, task_id: &str) -> Result<Vec<ChecklistItem>> {
        let url = format!("/me/todo/lists/{}/tasks/{}/checklistItems", self.list_id, task_id);
        let response: ChecklistItemCollection = self
            .http_client
            .get(&url, &self.auth_header)
            .await?;
        Ok(response.value)
    }

    /// Create a new checklist item
    pub async fn create_checklist_item(
        &self,
        task_id: &str,
        request: CreateChecklistItemRequest,
    ) -> Result<ChecklistItem> {
        let url = format!("/me/todo/lists/{}/tasks/{}/checklistItems", self.list_id, task_id);
        self.http_client
            .post(&url, &request, &self.auth_header)
            .await
    }

    /// Update a checklist item
    pub async fn update_checklist_item(
        &self,
        task_id: &str,
        item_id: &str,
        request: UpdateChecklistItemRequest,
    ) -> Result<ChecklistItem> {
        let url = format!(
            "/me/todo/lists/{}/tasks/{}/checklistItems/{}",
            self.list_id, task_id, item_id
        );
        self.http_client
            .patch(&url, &request, &self.auth_header)
            .await
    }

    /// Delete a checklist item
    pub async fn delete_checklist_item(&self, task_id: &str, item_id: &str) -> Result<()> {
        let url = format!(
            "/me/todo/lists/{}/tasks/{}/checklistItems/{}",
            self.list_id, task_id, item_id
        );
        self.http_client.delete(&url, &self.auth_header).await
    }

    /// Get a specific checklist item
    pub async fn get_checklist_item(&self, task_id: &str, item_id: &str) -> Result<ChecklistItem> {
        let url = format!(
            "/me/todo/lists/{}/tasks/{}/checklistItems/{}",
            self.list_id, task_id, item_id
        );
        self.http_client.get(&url, &self.auth_header).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = MsTodoClient::new("Bearer token".to_string(), "list123".to_string());
        assert_eq!(client.list_id, "list123");
    }

    #[test]
    fn test_checklist_item_creation() {
        let request = CreateChecklistItemRequest {
            displayName: "Test Item".to_string(),
            isChecked: Some(false),
        };
        assert_eq!(request.displayName, "Test Item");
        assert_eq!(request.isChecked, Some(false));
    }

    #[test]
    fn test_checklist_item_update() {
        let request = UpdateChecklistItemRequest {
            displayName: Some("Updated Item".to_string()),
            isChecked: Some(true),
        };
        assert_eq!(request.displayName, Some("Updated Item".to_string()));
        assert_eq!(request.isChecked, Some(true));
    }
}
