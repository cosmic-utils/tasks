use std::collections::{HashMap, HashSet, VecDeque};

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
        favorites::favorites::Favorites, lists::content::Content, tasks::details::Details,
        trash::trash::Trash,
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
}
