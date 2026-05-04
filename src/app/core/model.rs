use std::collections::{HashMap, VecDeque};

use cosmic::{
    app::Core,
    cosmic_config,
    iced::keyboard::Modifiers,
    widget::{about::About, menu::key_bind::KeyBind, nav_bar},
};

use crate::{
    app::{dialogs::DialogPage, ui::MenuAction},
    config,
    pages::{content::Content, details::Details, favorites::Favorites, trash::Trash},
    services::store::Store,
};

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    pub(crate) core: Core,
    /// Display a context drawer with the designated page if defined.
    pub(crate) context_page: super::context::ContextPage,
    /// The about page for this app.
    pub(crate) about: About,
    /// Contains items assigned to the nav bar panel.
    pub(crate) nav: nav_bar::Model,
    /// Key bindings for the application's menu bar.
    pub(crate) key_binds: HashMap<KeyBind, MenuAction>,
    /// Configuration handler for managing app settings.
    pub(crate) handler: cosmic_config::Config,
    /// Application-specific configuration.
    pub(crate) config: config::AppConfig,

    /// Current keyboard modifiers.
    pub(crate) modifiers: Modifiers,
    /// Queue of dialog pages.
    pub(crate) dialog_pages: VecDeque<DialogPage>,
    /// Identifier for the dialog text input widget.
    pub(crate) dialog_text_input: cosmic::widget::Id,

    /// Persistent storage for lists and tasks.
    pub(crate) store: Store,
    /// The main content area of the application.
    pub(crate) content: Content,
    /// The details view for tasks.
    pub(crate) details: Details,
    /// The trash page.
    pub(crate) trash: Trash,
    /// The nav bar entity for the trash item (kept so we can always reposition it last).
    pub(crate) trash_entity: nav_bar::Id,
    /// The favorites page.
    pub(crate) favorites: Favorites,
    /// The nav bar entity for the favorites item.
    pub(crate) favorites_entity: nav_bar::Id,
}
