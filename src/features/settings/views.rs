use cosmic::{widget, Element};

use crate::{
    app::{AppModel, Message},
    fl,
    shared::navigation::ui::{ApplicationAction, MenuAction},
};

pub fn settings(app: &AppModel) -> Element<'_, Message> {
    widget::scrollable(
        widget::settings::section()
            .title(fl!("appearance"))
            .add(widget::settings::item::item(
                fl!("theme"),
                widget::dropdown(
                    vec![fl!("match-desktop"), fl!("dark"), fl!("light")],
                    Some(app.config.app_theme.into()),
                    |theme| Message::Application(ApplicationAction::AppTheme(theme)),
                ),
            ))
            .add(widget::settings::item::item(
                fl!("show-favorites"),
                widget::toggler(app.config.show_favorites).on_toggle(|val| {
                    Message::Application(ApplicationAction::ToggleShowFavorites(val))
                }),
            ))
            .add(widget::settings::item::item(
                fl!("show-trash"),
                widget::toggler(app.config.show_trash)
                    .on_toggle(|val| Message::Application(ApplicationAction::ToggleShowTrash(val))),
            ))
            .add(widget::settings::item::item(
                fl!("sort-lists-by"),
                widget::dropdown(
                    vec![
                        fl!("sort-name-asc"),
                        fl!("sort-name-desc"),
                        fl!("sort-manual"),
                    ],
                    Some(app.config.list_sort_by.into()),
                    |sort_by| Message::Application(ApplicationAction::ListSortBy(sort_by)),
                ),
            ))
            .add(widget::settings::item::item(
                fl!("hide-completed"),
                widget::toggler(app.config.hide_completed)
                    .on_toggle(|val| Message::Menu(MenuAction::ToggleHideCompleted(val))),
            )),
    )
    .into()
}
