use jiff::civil::Date;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use super::model::{RemoteList, RemoteTask, RemoteTaskDraft};
use super::provider::{ProviderError, ProviderResult, RemoteTaskProvider};

const BASE_URL: &str = "https://tasks.googleapis.com/tasks/v1";

pub struct GoogleTasksProvider {
    client: reqwest::Client,
}

impl Default for GoogleTasksProvider {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct TaskListsResponse {
    #[serde(default)]
    items: Vec<GoogleTaskList>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GoogleTaskList {
    id: Option<String>,
    title: String,
    #[serde(default)]
    updated: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TasksResponse {
    #[serde(default)]
    items: Vec<GoogleTask>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GoogleTask {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    title: String,
    #[serde(default)]
    notes: String,
    #[serde(default)]
    due: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    updated: Option<String>,
}

impl From<GoogleTaskList> for RemoteList {
    fn from(value: GoogleTaskList) -> Self {
        RemoteList {
            remote_id: value.id.unwrap_or_default(),
            title: value.title,
        }
    }
}

fn parse_google_date(s: &str) -> Option<Date> {
    // Google's `due` field is an RFC3339 timestamp truncated to a date, e.g.
    // "2026-08-01T00:00:00.000Z".
    s.get(0..10).and_then(|d| d.parse::<Date>().ok())
}

fn parse_google_updated(s: &str) -> Option<Timestamp> {
    s.parse::<Timestamp>().ok()
}

impl From<GoogleTask> for RemoteTask {
    fn from(value: GoogleTask) -> Self {
        RemoteTask {
            remote_id: value.id.unwrap_or_default(),
            title: value.title,
            notes: value.notes,
            due_date: value.due.as_deref().and_then(parse_google_date),
            completed: value.status.as_deref() == Some("completed"),
            updated_at: value.updated.as_deref().and_then(parse_google_updated),
        }
    }
}

fn draft_to_json(draft: &RemoteTaskDraft) -> GoogleTask {
    GoogleTask {
        id: None,
        title: draft.title.clone(),
        notes: draft.notes.clone(),
        due: draft
            .due_date
            .map(|d| format!("{}T00:00:00.000Z", d)),
        status: Some(if draft.completed {
            "completed".to_string()
        } else {
            "needsAction".to_string()
        }),
        updated: None,
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
impl RemoteTaskProvider for GoogleTasksProvider {
    fn name(&self) -> &'static str {
        "google"
    }

    async fn list_lists(&self, token: &str) -> ProviderResult<Vec<RemoteList>> {
        let response = self
            .client
            .get(format!("{BASE_URL}/users/@me/lists"))
            .bearer_auth(token)
            .send()
            .await?;
        let response = check_status("google", response).await?;
        let body: TaskListsResponse = response.json().await?;
        Ok(body.items.into_iter().map(Into::into).collect())
    }

    async fn create_list(&self, token: &str, name: &str) -> ProviderResult<RemoteList> {
        let response = self
            .client
            .post(format!("{BASE_URL}/users/@me/lists"))
            .bearer_auth(token)
            .json(&serde_json::json!({ "title": name }))
            .send()
            .await?;
        let response = check_status("google", response).await?;
        let body: GoogleTaskList = response.json().await?;
        Ok(body.into())
    }

    async fn update_list(&self, token: &str, remote_id: &str, name: &str) -> ProviderResult<()> {
        let response = self
            .client
            .patch(format!("{BASE_URL}/users/@me/lists/{remote_id}"))
            .bearer_auth(token)
            .json(&serde_json::json!({ "title": name }))
            .send()
            .await?;
        check_status("google", response).await?;
        Ok(())
    }

    async fn delete_list(&self, token: &str, remote_id: &str) -> ProviderResult<()> {
        let response = self
            .client
            .delete(format!("{BASE_URL}/users/@me/lists/{remote_id}"))
            .bearer_auth(token)
            .send()
            .await?;
        check_status("google", response).await?;
        Ok(())
    }

    async fn list_tasks(&self, token: &str, list_remote_id: &str) -> ProviderResult<Vec<RemoteTask>> {
        let response = self
            .client
            .get(format!("{BASE_URL}/lists/{list_remote_id}/tasks"))
            .query(&[("showCompleted", "true"), ("showHidden", "true")])
            .bearer_auth(token)
            .send()
            .await?;
        let response = check_status("google", response).await?;
        let body: TasksResponse = response.json().await?;
        Ok(body.items.into_iter().map(Into::into).collect())
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
        let response = check_status("google", response).await?;
        let body: GoogleTask = response.json().await?;
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
        check_status("google", response).await?;
        Ok(())
    }

    async fn delete_task(&self, token: &str, list_remote_id: &str, remote_id: &str) -> ProviderResult<()> {
        let response = self
            .client
            .delete(format!("{BASE_URL}/lists/{list_remote_id}/tasks/{remote_id}"))
            .bearer_auth(token)
            .send()
            .await?;
        check_status("google", response).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_google_task_json() {
        let raw = r#"{"id":"abc","title":"Buy milk","notes":"2%","due":"2026-08-01T00:00:00.000Z","status":"needsAction","updated":"2026-07-30T12:00:00.000Z"}"#;
        let task: GoogleTask = serde_json::from_str(raw).unwrap();
        let remote: RemoteTask = task.into();
        assert_eq!(remote.remote_id, "abc");
        assert_eq!(remote.title, "Buy milk");
        assert!(!remote.completed);
        assert_eq!(remote.due_date, Some("2026-08-01".parse().unwrap()));
        assert!(remote.updated_at.is_some());
    }

    #[test]
    fn maps_completed_status() {
        let raw = r#"{"id":"abc","title":"Done","status":"completed"}"#;
        let task: GoogleTask = serde_json::from_str(raw).unwrap();
        let remote: RemoteTask = task.into();
        assert!(remote.completed);
    }
}
