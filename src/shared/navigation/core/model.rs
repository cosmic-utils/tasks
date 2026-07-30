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
    /// Provider id -> decoded icon `Handle`, for providers whose manifest
    /// icon is a URL. Populated asynchronously as fetches complete.
    ///
    /// Deliberately caches the constructed `Handle`, not the raw bytes:
    /// `iced::widget::image::Handle::from_bytes` stamps every call with a
    /// fresh globally-unique `Id` (not a content hash), so decoding the same
    /// bytes again on every nav rebuild produced a *different* `Handle` each
    /// time — the renderer could never recognize it as the same image,
    /// forcing a full re-decode/re-upload (visible as flicker, and the
    /// reason the nav bar felt slow) on every single call. Building the
    /// `Handle` once here and reusing it keeps its `Id` stable so repeated
    /// `icon_set` calls with the same image are free.
    pub(crate) provider_icons: HashMap<String, cosmic::widget::icon::Handle>,
    /// Provider ids whose icon fetch has already been attempted this
    /// session (success or failure), so we don't refetch on every reload.
    pub(crate) provider_icon_fetch_attempted: HashSet<String>,
    /// Account id -> its nav bar group-header entity. Reused across
    /// `reposition_special_items()` calls instead of tearing the header
    /// down and recreating it every time (which was both slow and caused
    /// the header image to visibly flicker on every nav rebuild).
    pub(crate) account_header_entities: HashMap<Uuid, nav_bar::Id>,
}
