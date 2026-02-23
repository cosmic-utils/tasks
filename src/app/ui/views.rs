use cosmic::{widget, Element};

use crate::{
    app::{
        core::{AppModel, Message},
        ui::ApplicationAction,
    },
    fl,
};

pub fn settings(app: &AppModel) -> Element<'_, Message> {
    widget::scrollable(widget::settings::section().title(fl!("appearance")).add(
        widget::settings::item::item(
            fl!("theme"),
            widget::dropdown(
                vec![fl!("match-desktop"), fl!("dark"), fl!("light")],
                Some(app.config.app_theme.into()),
                |theme| Message::Application(ApplicationAction::AppTheme(theme)),
            ),
        ),
    ))
    .into()
}
