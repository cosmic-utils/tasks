use std::collections::{HashMap, VecDeque};

use cosmic::{
    app::{self, Core},
    iced::core::keyboard::key::Named,
    iced::keyboard::Key,
    iced::keyboard::Modifiers,
    widget,
    widget::menu::key_bind::{KeyBind, Modifier},
    ApplicationExt,
};

use crate::{
    app::{navigation::TasksAction, ui::MenuAction},
    fl,
    pages::{content::Content, details::Details, trash::Trash},
};

use super::{context::ContextPage, flags::Flags, message::Message, model::AppModel};

impl AppModel {
    pub fn init(core: Core, flags: Flags) -> (Self, app::Task<Message>) {
        let nav_model = widget::segmented_button::ModelBuilder::default().build();

        let about = Self::about();

        let mut app = AppModel {
            core,
            context_page: ContextPage::Settings,
            about,
            nav: nav_model,
            key_binds: key_binds(),
            handler: flags.handler,
            config: flags.config.clone(),
            store: flags.store.clone(),
            content: Content::new(flags.store.clone(), flags.config),
            details: Details::new(flags.store.clone()),
            trash: Trash::new(flags.store),
            modifiers: Modifiers::empty(),
            dialog_pages: VecDeque::new(),
            dialog_text_input: widget::Id::unique(),
            trash_entity: widget::segmented_button::Entity::default(),
        };

        let mut tasks = vec![cosmic::task::message(Message::Tasks(
            TasksAction::FetchLists,
        ))];

        if let Some(id) = app.core.main_window_id() {
            tasks.push(app.set_window_title(fl!("tasks"), id));
        }

        app.core.nav_bar_toggle_condensed();

        // Insert the trash nav item and remember its entity so we can always
        // reposition it to the bottom after new lists are added.
        let trash_icon = widget::icon::from_name("user-trash-full-symbolic").size(16);
        app.trash_entity = app
            .nav
            .insert()
            .text(fl!("trash"))
            .icon(trash_icon)
            .data(crate::model::TrashMarker)
            .id();

        (app, app::Task::batch(tasks))
    }

    fn about() -> widget::about::About {
        widget::about::About::default()
            .name(fl!("tasks"))
            .icon(widget::icon::from_name("dev.edfloreshz.Tasks"))
            .version("0.2.2")
            .author("Eduardo Flores")
            .license("GPL-3.0-only")
            .links([
                (fl!("repository"), "https://github.com/cosmic-utils/tasks"),
                (
                    fl!("support"),
                    "https://github.com/cosmic-utils/tasks/issues",
                ),
                (fl!("website"), "https://tasks.edfloreshz.dev"),
            ])
            .developers([("Eduardo Flores", "edfloreshz@proton.me")])
    }
}

pub fn key_binds() -> HashMap<KeyBind, MenuAction> {
    let mut key_binds = HashMap::new();

    macro_rules! bind {
        ([$($modifier:ident),* $(,)?], $key:expr, $action:ident) => {{
            key_binds.insert(
                KeyBind {
                    modifiers: vec![$(Modifier::$modifier),*],
                    key: $key,
                },
                MenuAction::$action,
            );
        }};
    }

    bind!([Ctrl], Key::Character("n".into()), NewList);
    bind!([Ctrl], Key::Named(Named::Delete), DeleteList);
    bind!([Ctrl], Key::Character("r".into()), RenameList);
    bind!([Ctrl], Key::Character("I".into()), Icon);
    bind!([Ctrl], Key::Character("w".into()), WindowClose);
    bind!([Ctrl, Shift], Key::Character("n".into()), WindowNew);
    bind!([Ctrl], Key::Character(",".into()), Settings);
    bind!([Ctrl], Key::Character("i".into()), About);

    key_binds
}
