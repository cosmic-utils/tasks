use accounts::{
    models::{Account, DbusProviderInfo},
    AccountsClient,
};
use uuid::Uuid;

use crate::shared::providers::sync::SyncReport;

#[derive(Clone, Debug)]
pub enum AccountsAction {
    /// An account-added/removed/changed signal fired; reload the account list.
    AccountsChanged,
    AccountsLoaded(Vec<Account>),
    /// Fired once at startup after attempting to connect to accounts-daemon.
    DaemonConnected(AccountsClient),
    DaemonUnavailable,
    /// Opens the system Accounts app for the user to connect/disconnect
    /// accounts; Tasks itself does not manage account creation.
    OpenAccountsApp,
    /// A provider's icon URL (from its manifest) finished downloading.
    ProviderIconFetched(String, Option<Vec<u8>>),
    ProvidersLoaded(Vec<DbusProviderInfo>),
    /// Turns sync for an account on/off within Tasks only. Does not touch
    /// the account's Todo capability, which is managed by the Accounts app.
    SetLocallyEnabled(Uuid, bool),
    SyncFinished(SyncReport),
    /// Manual "Sync now" or a periodic tick.
    SyncNow,
}
