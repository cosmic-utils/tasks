use std::collections::HashMap;

use thiserror::Error;
use url::Url;

use crate::core::config::TasksConfig;
use crate::storage::models::{List, Task};
use crate::storage::LocalStorage;

use super::caldav::{parse_vtodo, task_to_vtodo, vtodo_to_task, CalDavClient, CalDavError};

const REMOTE_MARKER: &str = "caldav:";

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("CalDAV error: {0}")]
    CalDav(#[from] CalDavError),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Sync is not configured")]
    NotConfigured,
    #[error("Invalid remote URL: {0}")]
    Url(#[from] url::ParseError),
}

pub fn is_configured(config: &TasksConfig) -> bool {
    !config.sync_server_url.trim().is_empty()
        && !config.sync_username.trim().is_empty()
        && !config.sync_password.is_empty()
}

pub fn make_client(config: &TasksConfig) -> Result<CalDavClient, SyncError> {
    if !is_configured(config) {
        return Err(SyncError::NotConfigured);
    }
    Ok(CalDavClient::new(
        config.sync_server_url.trim(),
        config.sync_username.trim(),
        &config.sync_password,
    )?)
}

#[derive(Debug, Clone, Default)]
pub struct SyncReport {
    pub lists_pulled: usize,
    pub tasks_pulled: usize,
    pub tasks_pushed: usize,
}

/// Identify the remote URL bound to a local list, if any.
fn list_remote_url(list: &List) -> Option<Url> {
    let line = list
        .description
        .lines()
        .find(|l| l.trim().starts_with(REMOTE_MARKER))?;
    let raw = line.trim().trim_start_matches(REMOTE_MARKER).trim();
    Url::parse(raw).ok()
}

fn set_list_remote_url(list: &mut List, url: &Url) {
    let url_str = url.as_str();
    let mut kept: Vec<&str> = list
        .description
        .lines()
        .filter(|l| !l.trim().starts_with(REMOTE_MARKER))
        .collect();
    let line = format!("{REMOTE_MARKER}{url_str}");
    kept.push(&line);
    list.description = kept.join("\n");
}

/// Bidirectional sync. v1 semantics:
///   - Discover remote VTODO calendars; create matching local lists if missing.
///   - For each linked list: pull remote VTODOs into local, push local-only tasks.
///   - Conflicts: last_modified_date_time wins (no per-side tombstones, so deletes
///     are not propagated yet).
pub async fn sync(
    storage: &LocalStorage,
    config: &TasksConfig,
) -> Result<SyncReport, SyncError> {
    let client = make_client(config)?;
    let mut report = SyncReport::default();

    let mut local_lists = storage.lists().map_err(|e| SyncError::Storage(e.to_string()))?;
    let remote_calendars = client.list_task_calendars().await?;

    // Index local lists by their bound remote URL.
    let mut by_remote: HashMap<String, usize> = HashMap::new();
    for (i, l) in local_lists.iter().enumerate() {
        if let Some(u) = list_remote_url(l) {
            by_remote.insert(u.to_string(), i);
        }
    }

    // Ensure a local list exists for every remote calendar.
    for cal in &remote_calendars {
        let key = cal.url.to_string();
        if by_remote.contains_key(&key) {
            continue;
        }
        let mut list = List::new(&cal.display_name);
        set_list_remote_url(&mut list, &cal.url);
        let created = storage
            .create_list(&list)
            .map_err(|e| SyncError::Storage(e.to_string()))?;
        report.lists_pulled += 1;
        by_remote.insert(key, local_lists.len());
        local_lists.push(created);
    }

    // Sync each linked list.
    for cal in &remote_calendars {
        let Some(&idx) = by_remote.get(cal.url.as_str()) else {
            continue;
        };
        let list = local_lists[idx].clone();
        let local_tasks = storage
            .tasks(&list)
            .map_err(|e| SyncError::Storage(e.to_string()))?;
        let remote_todos = client.fetch_todos(&cal.url).await?;

        let mut remote_by_uid: HashMap<String, (Url, Option<String>, String)> = HashMap::new();
        for r in remote_todos {
            let todo = match parse_vtodo(&r.ical) {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!("skipping VTODO at {}: {e}", r.href);
                    continue;
                }
            };
            let uid = icalendar::Component::get_uid(&todo).unwrap_or("").to_string();
            if uid.is_empty() {
                continue;
            }
            remote_by_uid.insert(uid, (r.href, r.etag, r.ical));
        }

        let local_by_uid: HashMap<String, Task> = local_tasks
            .iter()
            .map(|t| (t.id.clone(), t.clone()))
            .collect();

        // Pull: write/update local from remote where remote is newer or local missing.
        for (uid, (_href, _etag, ical)) in &remote_by_uid {
            let Ok(todo) = parse_vtodo(ical) else { continue };
            let remote_task = vtodo_to_task(&todo, list.tasks_path());
            match local_by_uid.get(uid) {
                None => {
                    if let Err(e) = storage.create_task(&remote_task) {
                        tracing::warn!("create_task {uid} failed: {e}");
                    } else {
                        report.tasks_pulled += 1;
                    }
                }
                Some(local) => {
                    if remote_task.last_modified_date_time > local.last_modified_date_time {
                        if let Err(e) = storage.replace_task(&remote_task) {
                            tracing::warn!("replace_task {uid} failed: {e}");
                        } else {
                            report.tasks_pulled += 1;
                        }
                    }
                }
            }
        }

        // Push: PUT local-only tasks, and locally-newer tasks.
        for (uid, local) in &local_by_uid {
            let ical = task_to_vtodo(local);
            let target = cal.url.join(&format!("{uid}.ics"))?;
            match remote_by_uid.get(uid) {
                None => match client.put_todo(&target, &ical, None).await {
                    Ok(_) => report.tasks_pushed += 1,
                    Err(e) => tracing::warn!("PUT {uid} failed: {e}"),
                },
                Some((href, etag, _)) => {
                    let remote_task = parse_vtodo(&remote_by_uid[uid].2)
                        .ok()
                        .map(|t| vtodo_to_task(&t, list.tasks_path()));
                    let push = remote_task
                        .map(|r| local.last_modified_date_time > r.last_modified_date_time)
                        .unwrap_or(false);
                    if push {
                        match client.put_todo(href, &ical, etag.as_deref()).await {
                            Ok(_) => report.tasks_pushed += 1,
                            Err(e) => tracing::warn!("PUT update {uid} failed: {e}"),
                        }
                    }
                }
            }
        }
    }

    Ok(report)
}

pub async fn test_connection(config: &TasksConfig) -> Result<(), SyncError> {
    let client = make_client(config)?;
    client.test_connection().await?;
    Ok(())
}
