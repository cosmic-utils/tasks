use cosmic::widget;
use uuid::Uuid;

use crate::app;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NavMenuAction {
    Rename(Uuid),
    Export(Uuid),
    Delete(Uuid),
}

impl widget::menu::Action for NavMenuAction {
    type Message = cosmic::Action<app::Message>;

    fn message(&self) -> Self::Message {
        cosmic::Action::App(app::Message::NavMenu(*self))
    }
}
