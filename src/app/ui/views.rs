use cosmic::{widget, Element};

use crate::{
    app::{
        core::{AppModel, Message},
        ui::{ApplicationAction, MenuAction},
    },
    fl,
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
                fl!("hide-completed"),
                widget::toggler(app.config.hide_completed)
                    .on_toggle(|val| Message::Menu(MenuAction::ToggleHideCompleted(val))),
            )),
    )
    .into()
}
