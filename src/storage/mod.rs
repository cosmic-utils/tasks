pub mod models;


use crate::{
    app::markdown::Markdown,
    storage::models::{List, Task},
    Error, LocalStorageError, TasksError,
};



use tracing::{debug, error, info};

use crate::integration::ms_todo::{http_client::MsTodoHttpClient, models::*};

use crate::auth::ms_todo_auth::MsTodoAuth;



#[derive(Debug, Clone)]
pub struct LocalStorage {
    http_client: MsTodoHttpClient,
    auth: MsTodoAuth,
}


impl LocalStorage {
    pub fn new(_application_id: &str) -> Result<Self, LocalStorageError> {
        // For MS Graph, we need to get the auth token
        // This is a simplified approach - in practice, you'd want to get this from the auth system
        let auth = MsTodoAuth::new().map_err(|e| {
            LocalStorageError::LocalStorageDirectoryCreationFailed(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        if !auth.has_valid_tokens() {
            return Err(LocalStorageError::LocalStorageDirectoryCreationFailed(
                std::io::Error::new(std::io::ErrorKind::Other, "No valid tokens"),
            ));
        }

        Ok(Self {
            http_client: MsTodoHttpClient::new(),
            auth: auth,
        })
    }

    /// Get a valid access token, refreshing if necessary
    fn get_valid_token(&self) -> Result<String, LocalStorageError> {
        self.auth.get_access_token().map_err(|e| {
            LocalStorageError::LocalStorageDirectoryCreationFailed(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })
    }

    pub async fn tasks(&self, list: &List) -> Result<Vec<Task>, Error> {
        let auth_token = self
            .get_valid_token()
            .map_err(|_e| Error::Tasks(TasksError::TaskNotFound))?;
        let url =if list.hide_completed {
            format!("/me/todo/lists/{}/tasks?$filter=status ne 'completed'&$orderby=createdDateTime desc", list.id)
        } else {
            format!("/me/todo/lists/{}/tasks?$orderby=createdDateTime desc", list.id)
        };

        let response: TodoTaskCollection = self
            .http_client
            .get(
                &url,
                &format!("Bearer {}", auth_token),
            )
            .await
            .map_err(|_e| {

                error!("❌ Failed to get tasks via API: {}   for url {}", _e, url);
                Error::Tasks(TasksError::ApiError)
            })?;

        // Convert TodoTask[] → Task[] with proper path construction
        let tasks: Vec<Task> = response.value
            .into_iter()
            .map(|todo_task| {
                crate::integration::ms_todo::mapping::todo_task_to_task_with_path(
                    todo_task, &list.id,
                )
            })
            .collect();

        Ok(tasks)
    }
    pub async fn get_active_tasks(&self, list: &List) -> Result<Vec<Task>, Error> {
        let auth_token = self
            .get_valid_token()
            .map_err(|_e| Error::Tasks(TasksError::TaskNotFound))?;

        let response: TodoTaskCollection = self
            .http_client
            .get(
                &format!("/me/todo/lists/{}/tasks?$filter=status ne 'completed'", list.id),
                &format!("Bearer {}", auth_token),
            )
            .await
            .map_err(|_e| Error::Tasks(TasksError::TaskNotFound))?;

        // Convert TodoTask[] → Task[] with proper path construction
        let tasks: Vec<Task> = response.value
            .into_iter()
            .map(|todo_task| {
                crate::integration::ms_todo::mapping::todo_task_to_task_with_path(
                    todo_task, &list.id,
                )
            })
            .collect();

        Ok(tasks)
    }

    #[allow(dead_code)]
    pub fn sub_tasks(_task: &Task) -> Result<Vec<Task>, Error> {
        // Skip sub-tasks for now as requested
        Ok(Vec::new())
    }

    pub async fn lists(&mut self) -> Result<Vec<List>, Error> {

        let auth_token = self
            .get_valid_token()
            .map_err(|_e| Error::Tasks(TasksError::ListNotFound))?;

        // Use $expand to get tasks and $count for non-completed task count
        let response: TodoTaskListCollection = self
            .http_client
            .get(
                "/me/todo/lists",
                &format!("Bearer {}", auth_token),
            )
            .await
            .map_err(|_e| Error::Tasks(TasksError::ListNotFound))?;

        // Convert TodoTaskList[] → List[]
        debug!("Response: {:?}", response);
        let mut lists = Vec::new();
        for tl in response.value {
            let mut l = tl.into();
            let tasks = self.get_active_tasks(&l).await.unwrap_or_default();

            l.number_of_tasks = tasks.len() as u32;

            lists.push(l);
        }
        lists.sort_by(|a, b| a.well_known_list_name.cmp(&b.well_known_list_name).then(a.name.cmp(&b.name)));
        Ok(lists)
    }

    pub async fn create_task(&self, task: &Task) -> Result<Task, Error> {
        let list_id = task.list_id.clone().unwrap_or_default();
        let auth_token = self
            .get_valid_token()
            .map_err(|_e| Error::Tasks(TasksError::ExistingTask))?;

        info!("🔧 Creating task '{}' in list '{}'", task.title, list_id);

        info!("🔧 Task ID: {}", task.id);

        let request = CreateTodoTaskRequest::from(task);
        debug!(
            "Calling API to create task , {}",
            &format!("/me/todo/lists/{}/tasks", list_id)
        );
        let response: TodoTask = self
            .http_client
            .post(
                &format!("/me/todo/lists/{}/tasks", list_id),
                &request,
                &format!("Bearer {}", auth_token),
            )
            .await
            .map_err(|e| {
                error!("❌ Failed to create task via API: {}", e);
                Error::Tasks(TasksError::ApiError)
            })?;

        info!("✅ Task created successfully via API, ID: {}", response.id);

        // Convert TodoTask → Task with proper path construction
        let new_task =
            crate::integration::ms_todo::mapping::todo_task_to_task_with_path(response, &list_id);

        Ok(new_task)
    }

    pub async fn update_task(&self, task: &Task) -> Result<(), Error> {
        let list_id = task.list_id.clone().unwrap_or_default();
        let auth_token = self
            .get_valid_token()
            .map_err(|_e| Error::Tasks(TasksError::TaskNotFound))?;

        let request = UpdateTodoTaskRequest::from(task);
        info!("Request Json: {}", serde_json::to_string_pretty(&request).unwrap());
        info!("Request url: {}", &format!("/me/todo/lists/{}/tasks/{}", list_id, task.id));

        // PATCH returns the updated TodoTask
        let _: TodoTask = self
            .http_client
            .patch::<UpdateTodoTaskRequest, TodoTask>(
                &format!("/me/todo/lists/{}/tasks/{}", list_id, task.id),
                &request,
                &format!("Bearer {}", auth_token),
            )
            .await
            .map_err(|_e| Error::Tasks(TasksError::TaskNotFound))?;

        Ok(())
    }

    pub async fn delete_task(&self, task: &Task) -> Result<(), Error> {
        let list_id = task.list_id.clone().unwrap_or_default();
        let auth_token = self
            .get_valid_token()
            .map_err(|_e| Error::Tasks(TasksError::TaskNotFound))?;

        self.http_client
            .delete(
                &format!("/me/todo/lists/{}/tasks/{}", list_id, task.id),
                &format!("Bearer {}", auth_token),
            )
            .await
            .map_err(|_e| Error::Tasks(TasksError::TaskNotFound))?;

        Ok(())
    }

    pub async fn create_list(&self, list: &List) -> Result<List, Error> {
        let auth_token = self
            .get_valid_token()
            .map_err(|_e| Error::Tasks(TasksError::ExistingList))?;

        let request = CreateTodoTaskListRequest::from(list);

        let response: TodoTaskList = self
            .http_client
            .post(
                "/me/todo/lists",
                &request,
                &format!("Bearer {}", auth_token),
            )
            .await
            .map_err(|_e| Error::Tasks(TasksError::ExistingList))?;

        // Convert TodoTaskList → List
        Ok(response.into())
    }

    pub async fn update_list(&self, list: &List) -> Result<(), Error> {
        let auth_token = self
            .get_valid_token()
            .map_err(|_e| Error::Tasks(TasksError::ListNotFound))?;

        let request = UpdateTodoTaskListRequest::from(list);

        
        let url = format!("/me/todo/lists/{}", list.id);
        
        // Use TodoTaskList as the response type since PATCH returns the updated list
        let _: TodoTaskList = self
            .http_client
            .patch::<UpdateTodoTaskListRequest, TodoTaskList>(
                &url, 
                &request,
                &format!("Bearer {}", auth_token),
            )
            .await.map_err(|e| {
                error!("❌ Failed to update list via API: {}", e);
                crate::app::error::Error::Tasks(TasksError::ApiError)}
            )?;

        Ok(())
    }

    pub async fn delete_list(&self, list: &List) -> Result<(), Error> {
        let auth_token = self
            .get_valid_token()
            .map_err(|_e| Error::Tasks(TasksError::ListNotFound))?;

        self.http_client
            .delete(
                &format!("/me/todo/lists/{}", list.id),
                &format!("Bearer {}", auth_token),
            )
            .await.map_err(|e| crate::app::error::Error::Tasks(TasksError::ApiError))?;

        Ok(())
    }

    pub fn export_list(list: &List, tasks: &[Task]) -> String {
        let markdown = list.markdown();
        let tasks_markdown: String = tasks.iter().map(Markdown::markdown).collect();
        format!("{markdown}\n{tasks_markdown}")
    }

    // ============================================================================
    // Checklist Operations
    // ============================================================================

    /// Fetch checklist items for a task from MS Graph
    pub async fn fetch_checklist_items(&self, task: &Task) -> Result<Vec<crate::storage::models::ChecklistItem>, Error> {
        let auth_token = self
            .get_valid_token()
            .map_err(|_e| Error::Tasks(TasksError::TaskNotFound))?;

        // Extract list_id from task path or use stored list_id
        let list_id = task.list_id.as_ref()
            .ok_or_else(|| Error::Tasks(TasksError::TaskNotFound))?;

        let url = format!("/me/todo/lists/{}/tasks/{}/checklistItems", list_id, task.id);

        let response: crate::integration::ms_todo::models::ChecklistItemCollection = self
            .http_client
            .get(
                &url,
                &format!("Bearer {}", auth_token),
            )
            .await
            .map_err(|_e| {
                error!("❌ Failed to fetch checklist items via API: {}", _e);
                Error::Tasks(TasksError::ApiError)
            })?;

        // Convert MS Graph ChecklistItem[] → local ChecklistItem[]
        let items: Vec<crate::storage::models::ChecklistItem> = response.value
            .into_iter()
            .map(|item| item.into())
            .collect();

        Ok(items)
    }

    /// Create a new checklist item via MS Graph
    pub async fn create_checklist_item(&self, task: &Task, title: &str) -> Result<crate::storage::models::ChecklistItem, Error> {
        let auth_token = self
            .get_valid_token()
            .map_err(|_e| Error::Tasks(TasksError::TaskNotFound))?;

        let list_id = task.list_id.as_ref()
            .ok_or_else(|| Error::Tasks(TasksError::ApiError))?;

        let request = crate::integration::ms_todo::models::CreateChecklistItemRequest {
            displayName: title.to_string(),
            isChecked: Some(false),
        };

        let url = format!("/me/todo/lists/{}/tasks/{}/checklistItems", list_id, task.id);

        let response: crate::integration::ms_todo::models::ChecklistItem = self
            .http_client
            .post(
                &url,
                &request,
                &format!("Bearer {}", auth_token),
            )
            .await
            .map_err(|_e| {
                error!("❌ Failed to create checklist item via API: {}", _e);
                Error::Tasks(TasksError::ApiError)
            })?;

        Ok(response.into())
    }

    /// Update a checklist item via MS Graph
    pub async fn update_checklist_item(&self, task: &Task, item_id: &str, title: &str, is_checked: bool) -> Result<crate::storage::models::ChecklistItem, Error> {
        let auth_token = self
            .get_valid_token()
            .map_err(|_e| Error::Tasks(TasksError::TaskNotFound))?;

        let list_id = task.list_id.as_ref()
            .ok_or_else(|| Error::Tasks(TasksError::ApiError))?;

        let request = crate::integration::ms_todo::models::UpdateChecklistItemRequest {
            displayName: Some(title.to_string()),
            isChecked: Some(is_checked),
        };

        let url = format!("/me/todo/lists/{}/tasks/{}/checklistItems/{}", list_id, task.id, item_id);

        let response: crate::integration::ms_todo::models::ChecklistItem = self
            .http_client
            .patch(
                &url,
                &request,
                &format!("Bearer {}", auth_token),
            )
            .await
            .map_err(|_e| {
                error!("❌ Failed to update checklist item via API: {}", _e);
                Error::Tasks(TasksError::ApiError)
            })?;

        Ok(response.into())
    }

    /// Delete a checklist item via MS Graph
    pub async fn delete_checklist_item(&self, task: &Task, item_id: &str) -> Result<String, Error> {
        let auth_token = self
            .get_valid_token()
            .map_err(|_e| Error::Tasks(TasksError::TaskNotFound))?;

        let list_id = task.list_id.as_ref()
            .ok_or_else(|| Error::Tasks(TasksError::ApiError))?;

        let url = format!("/me/todo/lists/{}/tasks/{}/checklistItems/{}", list_id, task.id, item_id);

        self.http_client
            .delete(
                &url,
                &format!("Bearer {}", auth_token),
            )
            .await
            .map_err(|_e| {
                error!("❌ Failed to delete checklist item via API: {}", _e);
                Error::Tasks(TasksError::ApiError)
            })?;

        Ok(item_id.to_string())
    }
}
