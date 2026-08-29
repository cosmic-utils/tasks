use std::collections::BTreeSet;

use accounts::models::{Account, Service};
use accounts::AccountsClient;
use jiff::Timestamp;
use uuid::Uuid;

use crate::features::lists::list::List;
use crate::features::tasks::task::Task;
use crate::shared::store::source::TaskSource;
use crate::shared::store::Store;

use super::google::GoogleTasksProvider;
use super::microsoft::MicrosoftTodoProvider;
use super::model::RemoteTaskDraft;
use super::provider::{ProviderError, RemoteTaskProvider};

#[derive(Debug, Clone)]
pub enum SyncStatus {
    Syncing,
    Idle { at: Timestamp, had_errors: bool },
}

#[derive(Debug, Default, Clone)]
pub struct SyncReport {
    pub accounts_synced: usize,
    pub lists_pulled: usize,
    pub tasks_pulled: usize,
    pub tasks_pushed: usize,
    pub errors: Vec<String>,
}

impl SyncReport {
    pub fn had_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

fn provider_for(id: &str) -> Option<Box<dyn RemoteTaskProvider>> {
    match id {
        "google" => Some(Box::new(GoogleTasksProvider::default())),
        "microsoft" => Some(Box::new(MicrosoftTodoProvider::default())),
        _ => None,
    }
}

fn make_source(provider_id: &str, account_id: Uuid, remote_id: String) -> TaskSource {
    match provider_id {
        "google" => TaskSource::Google {
            account_id,
            remote_id,
        },
        _ => TaskSource::Microsoft {
            account_id,
            remote_id,
        },
    }
}

fn matches_account(source: &TaskSource, account_id: Uuid) -> bool {
    source.account_id() == Some(account_id)
}

/// Moves every list (and its tasks) mirrored from an account that is no
/// longer "active" — deleted, disabled, or with the Todo capability turned
/// off in the system Accounts app — into trash. `active_account_ids` should
/// contain exactly the accounts currently eligible to sync (account exists,
/// enabled, Todo capability on); anything remote-sourced outside that set is
/// considered orphaned. This does not consult Tasks' own local on/off
/// switch (`AppConfig::disabled_accounts`): turning an account off within
/// Tasks only pauses sync, it never deletes the local mirror.
pub fn purge_orphaned_accounts(store: &Store, active_account_ids: &BTreeSet<Uuid>) -> usize {
    let lists = match store.lists().load_all() {
        Ok(lists) => lists,
        Err(err) => {
            tracing::error!("Failed to load lists for orphan cleanup: {err}");
            return 0;
        }
    };

    let mut purged = 0;
    for list in lists {
        let Some(account_id) = list.source.account_id() else {
            continue;
        };
        if !active_account_ids.contains(&account_id) {
            match store.trash().trash_list(list.id) {
                Ok(()) => purged += 1,
                Err(err) => tracing::error!("Failed to trash orphaned list: {err}"),
            }
        }
    }
    purged
}

/// Runs a full two-way sync pass against every account with the Tasks service
/// enabled, excluding any the user has turned off within Tasks (`disabled`).
pub async fn run_sync(
    store: Store,
    mut accounts: AccountsClient,
    disabled: &BTreeSet<Uuid>,
) -> SyncReport {
    let mut report = SyncReport::default();

    let enabled = match accounts.list_enabled_accounts(Service::Tasks).await {
        Ok(accounts) => accounts,
        Err(err) => {
            report
                .errors
                .push(format!("Failed to list accounts: {err}"));
            return report;
        }
    };

    for account in enabled.into_iter().filter(|a| !disabled.contains(&a.id)) {
        let Some(provider) = provider_for(&account.provider) else {
            continue;
        };

        if let Err(err) = accounts.ensure_credentials(&account.id).await {
            report.errors.push(format!(
                "{}: failed to refresh credentials: {err}",
                account.display_name
            ));
            continue;
        }

        let token = match accounts
            .get_access_token(&account.id, &Service::Tasks)
            .await
        {
            Ok((token, _expires_at)) => token,
            Err(err) => {
                report.errors.push(format!(
                    "{}: failed to get access token: {err}",
                    account.display_name
                ));
                continue;
            }
        };

        match sync_account(&store, provider.as_ref(), &account, &token).await {
            Ok(account_report) => {
                report.lists_pulled += account_report.lists_pulled;
                report.tasks_pulled += account_report.tasks_pulled;
                report.tasks_pushed += account_report.tasks_pushed;
                report.errors.extend(account_report.errors);
                report.accounts_synced += 1;
            }
            Err(err) => {
                report
                    .errors
                    .push(format!("{}: {err}", account.display_name));
            }
        }
    }

    report
}

async fn sync_account(
    store: &Store,
    provider: &dyn RemoteTaskProvider,
    account: &Account,
    token: &str,
) -> Result<SyncReport, ProviderError> {
    let mut report = SyncReport::default();

    let remote_lists = provider.list_lists(token).await?;
    let local_lists = store.lists().load_all().unwrap_or_default();

    // Pull: new/renamed remote lists, and detect remote deletions.
    for remote_list in &remote_lists {
        let existing = local_lists.iter().find(|l| {
            matches_account(&l.source, account.id)
                && l.source.remote_id() == Some(remote_list.remote_id.as_str())
        });

        let list_id = match existing {
            Some(list) => {
                if list.name != remote_list.title {
                    if let Err(err) = store
                        .lists()
                        .update(list.id, |l| l.name = remote_list.title.clone())
                    {
                        tracing::error!("Failed to update list name: {err}");
                    }
                }
                list.id
            }
            None => {
                let mut list = List::new(&remote_list.title);
                list.source =
                    make_source(&account.provider, account.id, remote_list.remote_id.clone());
                if let Err(err) = store.lists().save(&list) {
                    report.errors.push(format!("Failed to save list: {err}"));
                    continue;
                }
                report.lists_pulled += 1;
                list.id
            }
        };

        match sync_list_tasks(
            store,
            provider,
            account,
            token,
            list_id,
            &remote_list.remote_id,
        )
        .await
        {
            Ok(list_report) => {
                report.tasks_pulled += list_report.tasks_pulled;
                report.tasks_pushed += list_report.tasks_pushed;
                report.errors.extend(list_report.errors);
            }
            Err(err) => report.errors.push(format!("{}: {err}", remote_list.title)),
        }
    }

    // Detect remote list deletions: local lists tracking this account whose
    // remote_id is no longer present get moved to trash (non-destructive).
    for local_list in &local_lists {
        if !matches_account(&local_list.source, account.id) {
            continue;
        }
        let still_exists = remote_lists
            .iter()
            .any(|r| Some(r.remote_id.as_str()) == local_list.source.remote_id());
        if !still_exists {
            if let Err(err) = store.trash().trash_list(local_list.id) {
                tracing::error!("Failed to trash remotely-deleted list: {err}");
            }
        }
    }

    // Push: local lists in this account with no remote_id yet (created
    // locally, e.g. copied) aren't auto-created remotely in this pass to
    // avoid surprising list creation; only tasks within already-linked lists
    // are pushed (see `sync_list_tasks`).

    Ok(report)
}

async fn sync_list_tasks(
    store: &Store,
    provider: &dyn RemoteTaskProvider,
    account: &Account,
    token: &str,
    list_id: Uuid,
    list_remote_id: &str,
) -> Result<SyncReport, ProviderError> {
    let mut report = SyncReport::default();
    let task_store = store.tasks(list_id);

    let remote_tasks = provider.list_tasks(token, list_remote_id).await?;
    let local_tasks = task_store.load_all().unwrap_or_default();

    let now = Timestamp::now();

    // Pull + push per matched remote task.
    for remote in &remote_tasks {
        let local = local_tasks.iter().find(|t| {
            matches_account(&t.source, account.id)
                && t.source.remote_id() == Some(remote.remote_id.as_str())
        });

        match local {
            Some(local) if local.dirty => {
                let draft = RemoteTaskDraft {
                    title: local.title.clone(),
                    notes: local.notes.clone(),
                    due_date: local.due_date,
                    completed: local.is_completed(),
                };
                if let Err(err) = provider
                    .update_task(token, list_remote_id, &remote.remote_id, &draft)
                    .await
                {
                    report
                        .errors
                        .push(format!("Failed to push task '{}': {err}", local.title));
                    continue;
                }
                let mut updated = local.clone();
                updated.dirty = false;
                updated.remote_updated_at = Some(now);
                updated.last_synced_at = Some(now);
                if let Err(err) = task_store.save_synced(&updated) {
                    tracing::error!("Failed to save synced task: {err}");
                }
                report.tasks_pushed += 1;
            }
            Some(local) => {
                let remote_newer = match local.remote_updated_at {
                    Some(local_updated) => remote.updated_at.is_none_or(|r| r > local_updated),
                    None => true,
                };
                if remote_newer {
                    apply_remote_to_local(&task_store, local.id, remote, now);
                    report.tasks_pulled += 1;
                }
            }
            None => {
                let mut task = Task::new(&remote.title);
                task.notes = remote.notes.clone();
                task.due_date = remote.due_date;
                if remote.completed {
                    task.completion_date = Some(now);
                }
                task.source = make_source(&account.provider, account.id, remote.remote_id.clone());
                task.remote_updated_at = remote.updated_at.or(Some(now));
                task.last_synced_at = Some(now);
                if let Err(err) = task_store.save_synced(&task) {
                    report
                        .errors
                        .push(format!("Failed to save pulled task: {err}"));
                    continue;
                }
                report.tasks_pulled += 1;
            }
        }
    }

    // Push newly-created local tasks (no remote id yet) that live in a
    // remote-linked list.
    for local in &local_tasks {
        if !local.source.is_local() {
            continue;
        }
        let draft = RemoteTaskDraft {
            title: local.title.clone(),
            notes: local.notes.clone(),
            due_date: local.due_date,
            completed: local.is_completed(),
        };
        match provider.create_task(token, list_remote_id, &draft).await {
            Ok(created) => {
                let mut updated = local.clone();
                updated.source = make_source(&account.provider, account.id, created.remote_id);
                updated.dirty = false;
                updated.remote_updated_at = Some(now);
                updated.last_synced_at = Some(now);
                if let Err(err) = task_store.save_synced(&updated) {
                    tracing::error!("Failed to save pushed task: {err}");
                }
                report.tasks_pushed += 1;
            }
            Err(err) => {
                report
                    .errors
                    .push(format!("Failed to push new task '{}': {err}", local.title));
            }
        }
    }

    // Remote deletions: local tasks tracking this list/account whose
    // remote_id is no longer present get moved to trash.
    for local in &local_tasks {
        if !matches_account(&local.source, account.id) || local.source.is_local() {
            continue;
        }
        let still_exists = remote_tasks
            .iter()
            .any(|r| Some(r.remote_id.as_str()) == local.source.remote_id());
        if !still_exists {
            let trashed = crate::features::tasks::task::TrashedTask::new(local.clone(), list_id);
            if store.trash().save(&trashed).is_ok() {
                let _ = task_store.delete(local.id);
            }
        }
    }

    Ok(report)
}

fn apply_remote_to_local(
    task_store: &crate::shared::store::store::TaskStore<'_>,
    task_id: Uuid,
    remote: &super::model::RemoteTask,
    now: Timestamp,
) {
    let mut task = match task_store.get(task_id) {
        Ok(task) => task,
        Err(err) => {
            tracing::error!("Failed to load task for sync: {err}");
            return;
        }
    };
    task.title = remote.title.clone();
    task.notes = remote.notes.clone();
    task.due_date = remote.due_date;
    task.completion_date = if remote.completed {
        task.completion_date.or(Some(now))
    } else {
        None
    };
    task.remote_updated_at = remote.updated_at.or(Some(now));
    task.last_synced_at = Some(now);
    task.dirty = false;
    if let Err(err) = task_store.save_synced(&task) {
        tracing::error!("Failed to apply remote task update: {err}");
    }
}
