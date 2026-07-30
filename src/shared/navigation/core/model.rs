use std::collections::{HashMap, HashSet, VecDeque};

use accounts::models::{Account, DbusProviderInfo};
use accounts::AccountsClient;
use cosmic::{
    app::Core,
    cosmic_config,
    iced::keyboard::Modifiers,
    widget::{about::About, menu::key_bind::KeyBind, nav_bar},
};

use uuid::Uuid;

use crate::{
    config,
    features::{
        favorites::favorites::Favorites, lists::content::Content, search::search::Search,
        tasks::details::Details, trash::trash::Trash,
    },
    shared::{dialogs::DialogPage, navigation::ui::MenuAction, store::Store},
};

pub struct AppModel {
    pub(crate) core: Core,
    pub(crate) context_page: super::context::ContextPage,
    pub(crate) about: About,
    pub(crate) nav: nav_bar::Model,
    pub(crate) key_binds: HashMap<KeyBind, MenuAction>,
    pub(crate) handler: cosmic_config::Config,
    pub(crate) config: config::AppConfig,
    pub(crate) modifiers: Modifiers,
    pub(crate) dialog_pages: VecDeque<DialogPage>,
    pub(crate) dialog_text_input: cosmic::widget::Id,
    pub(crate) store: Store,
    pub(crate) content: Content,
    pub(crate) details: Details,
    pub(crate) trash: Trash,
    pub(crate) trash_entity: nav_bar::Id,
    pub(crate) favorites: Favorites,
    pub(crate) favorites_entity: nav_bar::Id,
    pub(crate) sent_reminders: HashSet<(Uuid, i64)>,
    pub(crate) toasts: cosmic::widget::Toasts<super::message::Message>,
    pub(crate) search: Search,
    /// `None` until we know whether accounts-daemon is reachable; still `None`
    /// afterwards if it isn't installed/running, in which case account sync
    /// features are hidden and the app behaves as fully local-only.
    pub(crate) accounts: Option<AccountsClient>,
    pub(crate) accounts_daemon_checked: bool,
    /// Set once the account list has been loaded at least once, so the
    /// "an account just became active" auto-enable logic doesn't treat
    /// every already-active account as newly active on the very first load
    /// and wipe the user's saved local on/off preferences.
    pub(crate) accounts_loaded_once: bool,
    pub(crate) remote_accounts: Vec<Account>,
    pub(crate) sync_status: Option<crate::shared::providers::sync::SyncStatus>,
    /// Provider capability list from `AccountsClient::list_providers()`,
    /// carrying each provider's declared icon (URL / path / theme name).
    pub(crate) providers: Vec<DbusProviderInfo>,
    /// Provider id -> downloaded icon image bytes, for providers whose
    /// manifest icon is a URL. Populated asynchronously as fetches complete.
    pub(crate) provider_icons: HashMap<String, Vec<u8>>,
    /// Provider ids whose icon fetch has already been attempted this
    /// session (success or failure), so we don't refetch on every reload.
    pub(crate) provider_icon_fetch_attempted: HashSet<String>,
}
