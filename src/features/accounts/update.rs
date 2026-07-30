use accounts::models::Service;
use cosmic::app;
use std::collections::BTreeSet;

use crate::app::Message;
use crate::shared::navigation::core::AppModel;

use super::AccountsAction;

impl AppModel {
    pub fn update_accounts(&mut self, action: AccountsAction) -> app::Task<Message> {
        match action {
            AccountsAction::DaemonConnected(client) => {
                self.accounts = Some(client.clone());
                self.accounts_daemon_checked = true;
                // `AccountsLoaded` (fired once this resolves) triggers the
                // initial sync itself, so no separate `SyncNow` is needed here.
                let client_for_providers = client.clone();
                let tasks: Vec<app::Task<Message>> = vec![
                    reload_accounts_task(self.accounts.clone()),
                    cosmic::task::future(async move {
                        match client_for_providers.list_providers().await {
                            Ok(providers) => {
                                Message::Accounts(AccountsAction::ProvidersLoaded(providers))
                            }
                            Err(err) => {
                                tracing::error!("Failed to list providers: {err}");
                                Message::Accounts(AccountsAction::ProvidersLoaded(Vec::new()))
                            }
                        }
                    }),
                ];
                return app::Task::batch(tasks);
            }
            AccountsAction::DaemonUnavailable => {
                self.accounts = None;
                self.accounts_daemon_checked = true;
            }
            AccountsAction::AccountsChanged => {
                return reload_accounts_task(self.accounts.clone());
            }
            AccountsAction::AccountsLoaded(accounts) => {
                let previously_active = todo_active_ids(&self.remote_accounts);
                let is_first_load = !self.accounts_loaded_once;
                self.accounts_loaded_once = true;
                self.remote_accounts = accounts.clone();

                // Whatever isn't in this set (account deleted, disabled, or
                // Todo capability turned off) had its local mirror purged,
                // regardless of Tasks' own local on/off switch: that switch
                // only pauses sync, it never governs whether data is kept.
                let active = todo_active_ids(&accounts);

                // An account whose Todo capability just turned on (wasn't
                // active before, is now) should start syncing immediately,
                // without requiring the user to also flip the Tasks-local
                // toggle: clear it from the local disabled set. Skipped on
                // the very first load, where "previously active" is
                // vacuously empty and would otherwise wipe every saved
                // local preference on every app startup.
                if !is_first_load {
                    let newly_active: Vec<uuid::Uuid> =
                        active.difference(&previously_active).copied().collect();
                    if !newly_active.is_empty() {
                        let mut disabled = self.config.disabled_accounts.clone();
                        let mut changed = false;
                        for id in newly_active {
                            changed |= disabled.remove(&id);
                        }
                        if changed {
                            if let Err(err) =
                                self.config.set_disabled_accounts(&self.handler, disabled)
                            {
                                tracing::error!("Failed to save account sync preference: {err}");
                            }
                        }
                    }
                }

                let store = self.store.clone();
                let tasks: Vec<app::Task<Message>> = vec![
                    cosmic::task::future(async move {
                        crate::shared::providers::sync::purge_orphaned_accounts(&store, &active);
                        Message::Tasks(crate::shared::navigation::nav::TasksAction::SyncFromDisk)
                    }),
                    // Pull data for any newly-active account right away,
                    // instead of waiting for the next periodic sync.
                    cosmic::task::message(Message::Accounts(AccountsAction::SyncNow)),
                ];
                return app::Task::batch(tasks);
            }
            AccountsAction::ProvidersLoaded(providers) => {
                self.providers = providers;

                let mut fetches: Vec<app::Task<Message>> = Vec::new();
                for provider in &self.providers {
                    let Some(accounts::models::IconSource::Url(url)) = provider.icon_source()
                    else {
                        continue;
                    };
                    if !self
                        .provider_icon_fetch_attempted
                        .insert(provider.id.clone())
                    {
                        continue;
                    }
                    let provider_id = provider.id.clone();
                    fetches.push(cosmic::task::future(async move {
                        let bytes = reqwest::get(&url)
                            .await
                            .ok()
                            .filter(|r| r.status().is_success());
                        let bytes = match bytes {
                            Some(response) => response.bytes().await.ok().map(|b| b.to_vec()),
                            None => None,
                        };
                        if bytes.is_none() {
                            tracing::error!("Failed to fetch provider icon from {url}");
                        }
                        Message::Accounts(AccountsAction::ProviderIconFetched(provider_id, bytes))
                    }));
                }
                if !fetches.is_empty() {
                    return app::Task::batch(fetches);
                }
            }
            AccountsAction::ProviderIconFetched(provider_id, bytes) => {
                if let Some(bytes) = bytes {
                    // Build the `Handle` once per fetch so its `Id` stays stable.
                    let handle = cosmic::widget::icon::from_raster_bytes(bytes);
                    self.provider_icons.insert(provider_id, handle);
                    // Re-render nav headers now that the real icon is available.
                    self.reposition_special_items();
                }
            }
            AccountsAction::OpenAccountsApp => {
                if let Err(err) = std::process::Command::new("accounts-ui").spawn() {
                    tracing::error!("Failed to launch the Accounts app: {err}");
                    return self
                        .toasts
                        .push(cosmic::widget::Toast::new(crate::fl!(
                            "accounts-app-launch-failed"
                        )))
                        .map(cosmic::Action::App);
                }
            }
            AccountsAction::SetLocallyEnabled(id, enabled) => {
                let mut disabled = self.config.disabled_accounts.clone();
                if enabled {
                    disabled.remove(&id);
                } else {
                    disabled.insert(id);
                }
                if let Err(err) = self.config.set_disabled_accounts(&self.handler, disabled) {
                    tracing::error!("Failed to save account sync preference: {err}");
                }
            }
            AccountsAction::SyncNow => {
                let Some(client) = self.accounts.clone() else {
                    return app::Task::none();
                };
                self.sync_status = Some(crate::shared::providers::sync::SyncStatus::Syncing);
                let store = self.store.clone();
                let disabled = self.config.disabled_accounts.clone();
                return cosmic::task::future(async move {
                    let report =
                        crate::shared::providers::sync::run_sync(store, client, &disabled).await;
                    Message::Accounts(AccountsAction::SyncFinished(report))
                });
            }
            AccountsAction::SyncFinished(report) => {
                let needs_reconnect = report
                    .errors
                    .iter()
                    .any(|err| err.contains("authorization expired"));
                for err in &report.errors {
                    tracing::error!("sync: {err}");
                }
                self.sync_status = Some(crate::shared::providers::sync::SyncStatus::Idle {
                    at: jiff::Timestamp::now(),
                    had_errors: report.had_errors(),
                });
                let mut tasks = vec![cosmic::task::message(Message::Tasks(
                    crate::shared::navigation::nav::TasksAction::SyncFromDisk,
                ))];
                if needs_reconnect {
                    tasks.push(
                        self.toasts
                            .push(cosmic::widget::Toast::new(crate::fl!("reconnect-account")))
                            .map(cosmic::Action::App),
                    );
                }
                return app::Task::batch(tasks);
            }
        }

        app::Task::none()
    }
}

fn todo_active_ids(accounts: &[accounts::models::Account]) -> BTreeSet<uuid::Uuid> {
    accounts
        .iter()
        .filter(|a| a.enabled && a.services.get(&Service::Tasks).copied().unwrap_or(false))
        .map(|a| a.id)
        .collect()
}

fn reload_accounts_task(client: Option<accounts::AccountsClient>) -> app::Task<Message> {
    let Some(client) = client else {
        return app::Task::none();
    };
    cosmic::task::future(async move {
        match client.list_accounts().await {
            Ok(accounts) => Message::Accounts(AccountsAction::AccountsLoaded(accounts)),
            Err(err) => {
                tracing::error!("Failed to list accounts: {err}");
                Message::Accounts(AccountsAction::AccountsLoaded(Vec::new()))
            }
        }
    })
}
