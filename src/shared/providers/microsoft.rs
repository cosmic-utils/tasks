use jiff::civil::Date;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use super::model::{RemoteList, RemoteTask, RemoteTaskDraft};
use super::provider::{ProviderError, ProviderResult, RemoteTaskProvider};

const BASE_URL: &str = "https://graph.microsoft.com/v1.0/me/todo";

pub struct MicrosoftTodoProvider {
    client: reqwest::Client,
}

impl Default for MicrosoftTodoProvider {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ListsResponse {
    #[serde(default)]
    value: Vec<GraphList>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GraphList {
    #[serde(default)]
    id: Option<String>,
    #[serde(rename = "displayName")]
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct TasksResponse {
    #[serde(default)]
    value: Vec<GraphTask>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GraphBody {
    #[serde(default)]
    content: String,
    #[serde(rename = "contentType", default = "default_content_type")]
    content_type: String,
}

fn default_content_type() -> String {
    "text".to_string()
}

#[derive(Debug, Deserialize, Serialize)]
struct GraphDateTime {
    #[serde(rename = "dateTime")]
    date_time: String,
    #[serde(rename = "timeZone", default = "default_time_zone")]
    time_zone: String,
}

fn default_time_zone() -> String {
    "UTC".to_string()
}

#[derive(Debug, Deserialize, Serialize)]
struct GraphTask {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    title: String,
    #[serde(default)]
    body: Option<GraphBody>,
    #[serde(default)]
    status: Option<String>,
    #[serde(rename = "dueDateTime", default)]
    due_date_time: Option<GraphDateTime>,
    #[serde(rename = "lastModifiedDateTime", default)]
    last_modified_date_time: Option<String>,
}

impl From<GraphList> for RemoteList {
    fn from(value: GraphList) -> Self {
        RemoteList {
            remote_id: value.id.unwrap_or_default(),
            title: value.display_name,
        }
    }
}

fn parse_graph_date(dt: &GraphDateTime) -> Option<Date> {
    dt.date_time.get(0..10).and_then(|d| d.parse::<Date>().ok())
}

fn parse_graph_modified(s: &str) -> Option<Timestamp> {
    s.parse::<Timestamp>().ok()
}

impl From<GraphTask> for RemoteTask {
    fn from(value: GraphTask) -> Self {
        RemoteTask {
            remote_id: value.id.unwrap_or_default(),
            title: value.title,
            notes: value.body.map(|b| b.content).unwrap_or_default(),
            due_date: value.due_date_time.as_ref().and_then(parse_graph_date),
            completed: value.status.as_deref() == Some("completed"),
            updated_at: value
                .last_modified_date_time
                .as_deref()
                .and_then(parse_graph_modified),
        }
    }
}

fn draft_to_json(draft: &RemoteTaskDraft) -> GraphTask {
    GraphTask {
        id: None,
        title: draft.title.clone(),
        body: Some(GraphBody {
            content: draft.notes.clone(),
            content_type: default_content_type(),
        }),
        status: Some(if draft.completed {
            "completed".to_string()
        } else {
            "notStarted".to_string()
        }),
        due_date_time: draft.due_date.map(|d| GraphDateTime {
            date_time: format!("{}T00:00:00.0000000", d),
            time_zone: default_time_zone(),
        }),
        last_modified_date_time: None,
    }
}

async fn check_status(provider: &'static str, response: reqwest::Response) -> ProviderResult<reqwest::Response> {
    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err(ProviderError::Unauthorized);
    }
    if !status.is_success() {
        let message = response.text().await.unwrap_or_default();
        return Err(ProviderError::Api {
            provider,
            status: status.as_u16(),
            message,
        });
    }
    Ok(response)
}

#[async_trait::async_trait]
impl RemoteTaskProvider for MicrosoftTodoProvider {
    fn name(&self) -> &'static str {
        "microsoft"
    }

    async fn list_lists(&self, token: &str) -> ProviderResult<Vec<RemoteList>> {
        let response = self
            .client
            .get(format!("{BASE_URL}/lists"))
            .bearer_auth(token)
            .send()
            .await?;
        let response = check_status("microsoft", response).await?;
        let body: ListsResponse = response.json().await?;
        Ok(body.value.into_iter().map(Into::into).collect())
    }

    async fn create_list(&self, token: &str, name: &str) -> ProviderResult<RemoteList> {
        let response = self
            .client
            .post(format!("{BASE_URL}/lists"))
            .bearer_auth(token)
            .json(&serde_json::json!({ "displayName": name }))
            .send()
            .await?;
        let response = check_status("microsoft", response).await?;
        let body: GraphList = response.json().await?;
        Ok(body.into())
    }

    async fn update_list(&self, token: &str, remote_id: &str, name: &str) -> ProviderResult<()> {
        let response = self
            .client
            .patch(format!("{BASE_URL}/lists/{remote_id}"))
            .bearer_auth(token)
            .json(&serde_json::json!({ "displayName": name }))
            .send()
            .await?;
        check_status("microsoft", response).await?;
        Ok(())
    }

    async fn delete_list(&self, token: &str, remote_id: &str) -> ProviderResult<()> {
        let response = self
            .client
            .delete(format!("{BASE_URL}/lists/{remote_id}"))
            .bearer_auth(token)
            .send()
            .await?;
        check_status("microsoft", response).await?;
        Ok(())
    }

    async fn list_tasks(&self, token: &str, list_remote_id: &str) -> ProviderResult<Vec<RemoteTask>> {
        let response = self
            .client
            .get(format!("{BASE_URL}/lists/{list_remote_id}/tasks"))
            .bearer_auth(token)
            .send()
            .await?;
        let response = check_status("microsoft", response).await?;
        let body: TasksResponse = response.json().await?;
        Ok(body.value.into_iter().map(Into::into).collect())
    }

    async fn create_task(
        &self,
        token: &str,
        list_remote_id: &str,
        task: &RemoteTaskDraft,
    ) -> ProviderResult<RemoteTask> {
        let response = self
            .client
            .post(format!("{BASE_URL}/lists/{list_remote_id}/tasks"))
            .bearer_auth(token)
            .json(&draft_to_json(task))
            .send()
            .await?;
        let response = check_status("microsoft", response).await?;
        let body: GraphTask = response.json().await?;
        Ok(body.into())
    }

    async fn update_task(
        &self,
        token: &str,
        list_remote_id: &str,
        remote_id: &str,
        task: &RemoteTaskDraft,
    ) -> ProviderResult<()> {
        let response = self
            .client
            .patch(format!("{BASE_URL}/lists/{list_remote_id}/tasks/{remote_id}"))
            .bearer_auth(token)
            .json(&draft_to_json(task))
            .send()
            .await?;
        check_status("microsoft", response).await?;
        Ok(())
    }

    async fn delete_task(&self, token: &str, list_remote_id: &str, remote_id: &str) -> ProviderResult<()> {
        let response = self
            .client
            .delete(format!("{BASE_URL}/lists/{list_remote_id}/tasks/{remote_id}"))
            .bearer_auth(token)
            .send()
            .await?;
        check_status("microsoft", response).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_graph_task_json() {
        let raw = r#"{"id":"abc","title":"Buy milk","body":{"content":"2%","contentType":"text"},"status":"notStarted","dueDateTime":{"dateTime":"2026-08-01T00:00:00.0000000","timeZone":"UTC"},"lastModifiedDateTime":"2026-07-30T12:00:00Z"}"#;
        let task: GraphTask = serde_json::from_str(raw).unwrap();
        let remote: RemoteTask = task.into();
        assert_eq!(remote.remote_id, "abc");
        assert_eq!(remote.notes, "2%");
        assert!(!remote.completed);
        assert_eq!(remote.due_date, Some("2026-08-01".parse().unwrap()));
        assert!(remote.updated_at.is_some());
    }

    #[test]
    fn maps_completed_status() {
        let raw = r#"{"id":"abc","title":"Done","status":"completed"}"#;
        let task: GraphTask = serde_json::from_str(raw).unwrap();
        let remote: RemoteTask = task.into();
        assert!(remote.completed);
    }
}
