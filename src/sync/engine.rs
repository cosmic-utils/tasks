use std::collections::HashMap;

use thiserror::Error;
use url::Url;

use crate::storage::LocalStorage;
use crate::storage::models::{List, Task};

use super::caldav::{CalDavClient, CalDavError, parse_vtodo, task_to_vtodo, vtodo_to_task};

/// Legacy marker used in v0.2 to embed the CalDAV URL inside the list
/// description. Kept only for read-side migration into `List::remote_url`.
const LEGACY_REMOTE_MARKER: &str = "caldav:";

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

#[derive(Debug, Clone)]
pub struct SyncCredentials {
    pub server_url: String,
    pub username: String,
    pub password: String,
}

pub fn is_configured(creds: &SyncCredentials) -> bool {
    !creds.server_url.trim().is_empty()
        && !creds.username.trim().is_empty()
        && !creds.password.is_empty()
}

pub fn make_client(creds: &SyncCredentials) -> Result<CalDavClient, SyncError> {
    if !is_configured(creds) {
        return Err(SyncError::NotConfigured);
    }
    Ok(CalDavClient::new(
        creds.server_url.trim(),
        creds.username.trim(),
        &creds.password,
    )?)
}

#[derive(Debug, Clone, Default)]
pub struct SyncReport {
    pub lists_pulled: usize,
    pub tasks_pulled: usize,
    pub tasks_pushed: usize,
    pub tasks_failed: usize,
}

/// Identify the remote URL bound to a local list, if any.
///
/// Reads `List::remote_url` first; falls back to the legacy `caldav:URL`
/// marker that v0.2 stored in `description`.
fn list_remote_url(list: &List) -> Option<Url> {
    if let Some(raw) = list.remote_url.as_deref() {
        if let Ok(url) = Url::parse(raw) {
            return Some(url);
        }
    }
    let line = list
        .description
        .lines()
        .find(|l| l.trim().starts_with(LEGACY_REMOTE_MARKER))?;
    let raw = line.trim().trim_start_matches(LEGACY_REMOTE_MARKER).trim();
    Url::parse(raw).ok()
}

fn set_list_remote_url(list: &mut List, url: &Url) {
    list.remote_url = Some(url.as_str().to_string());
    // Strip any legacy marker line from the description.
    let kept: Vec<&str> = list
        .description
        .lines()
        .filter(|l| !l.trim().starts_with(LEGACY_REMOTE_MARKER))
        .collect();
    list.description = kept.join("\n");
}

/// Bidirectional sync. v1 semantics:
///   - Discover remote VTODO calendars; create matching local lists if missing.
///   - For each linked list: pull remote VTODOs into local, push local-only tasks.
///   - Conflicts: last_modified_date_time wins (no per-side tombstones, so deletes
///     are not propagated yet).
pub async fn sync(
    storage: &LocalStorage,
    creds: &SyncCredentials,
) -> Result<SyncReport, SyncError> {
    let client = make_client(creds)?;
    let mut report = SyncReport::default();

    let mut local_lists = storage
        .lists()
        .map_err(|e| SyncError::Storage(e.to_string()))?;
    let remote_calendars = client.list_task_calendars().await?;

    // Index local lists by their bound remote URL, migrating legacy
    // description-encoded markers into `List::remote_url` on the way.
    let mut by_remote: HashMap<String, usize> = HashMap::new();
    for (i, l) in local_lists.iter_mut().enumerate() {
        let Some(u) = list_remote_url(l) else {
            continue;
        };
        if l.remote_url.as_deref() != Some(u.as_str()) {
            set_list_remote_url(l, &u);
            if let Err(e) = storage.update_list(l) {
                tracing::warn!("migrating legacy remote_url for {}: {e}", l.id);
            }
        }
        by_remote.insert(u.to_string(), i);
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
            let uid = icalendar::Component::get_uid(&todo)
                .unwrap_or("")
                .to_string();
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
            let Ok(todo) = parse_vtodo(ical) else {
                continue;
            };
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
                    Err(e) => {
                        tracing::warn!("PUT {uid} failed: {e}");
                        report.tasks_failed += 1;
                    }
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
                            Err(e) => {
                                tracing::warn!("PUT update {uid} failed: {e}");
                                report.tasks_failed += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(report)
}

pub async fn test_connection(creds: &SyncCredentials) -> Result<(), SyncError> {
    let client = make_client(creds)?;
    client.test_connection().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_list() -> List {
        List::new("test")
    }

    #[test]
    fn legacy_marker_in_description_is_recognized() {
        let mut list = empty_list();
        list.description = "notes\ncaldav:https://example.com/dav/cal/".into();
        let url = list_remote_url(&list).expect("legacy marker should parse");
        assert_eq!(url.as_str(), "https://example.com/dav/cal/");
    }

    #[test]
    fn remote_url_field_takes_precedence() {
        let mut list = empty_list();
        list.description = "caldav:https://old.example.com/".into();
        list.remote_url = Some("https://new.example.com/".into());
        let url = list_remote_url(&list).unwrap();
        assert_eq!(url.as_str(), "https://new.example.com/");
    }

    #[test]
    fn set_remote_url_strips_legacy_marker() {
        let mut list = empty_list();
        list.description = "first line\ncaldav:https://x/\nlast".into();
        let url = Url::parse("https://example.com/cal/").unwrap();
        set_list_remote_url(&mut list, &url);
        assert_eq!(list.remote_url.as_deref(), Some("https://example.com/cal/"));
        assert!(!list.description.contains("caldav:"));
        assert!(list.description.contains("first line"));
        assert!(list.description.contains("last"));
    }

    #[test]
    fn is_configured_requires_all_fields() {
        let blank = SyncCredentials {
            server_url: String::new(),
            username: String::new(),
            password: String::new(),
        };
        assert!(!is_configured(&blank));
        let full = SyncCredentials {
            server_url: "https://x/".into(),
            username: "u".into(),
            password: "p".into(),
        };
        assert!(is_configured(&full));
    }
}
