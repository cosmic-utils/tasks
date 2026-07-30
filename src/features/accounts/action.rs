use accounts::models::{Account, DbusProviderInfo};
use accounts::AccountsClient;
use uuid::Uuid;

use crate::shared::providers::sync::SyncReport;

#[derive(Debug, Clone)]
pub enum AccountsAction {
    /// Fired once at startup after attempting to connect to accounts-daemon.
    DaemonConnected(AccountsClient),
    DaemonUnavailable,
    /// An account-added/removed/changed signal fired; reload the account list.
    AccountsChanged,
    AccountsLoaded(Vec<Account>),
    ProvidersLoaded(Vec<DbusProviderInfo>),
    /// A provider's icon URL (from its manifest) finished downloading.
    ProviderIconFetched(String, Option<Vec<u8>>),
    /// Opens the system Accounts app for the user to connect/disconnect
    /// accounts; Tasks itself does not manage account creation.
    OpenAccountsApp,
    /// Turns sync for an account on/off within Tasks only. Does not touch
    /// the account's Todo capability, which is managed by the Accounts app.
    SetLocallyEnabled(Uuid, bool),
    /// Manual "Sync now" or a periodic tick.
    SyncNow,
    SyncFinished(SyncReport),
}
