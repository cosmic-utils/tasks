use cosmic::{
    iced::keyboard::{Key, Modifiers},
    widget::menu::Action,
};

use crate::app::core::{ContextPage, Message};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    Settings,
    WindowClose,
    WindowNew,
    NewList,
    DeleteList,
    RenameList,
    Icon,
    ToggleHideCompleted(bool),
    SortByNameAsc,
    SortByNameDesc,
    SortByDateAsc,
    SortByDateDesc,
}

#[derive(Debug, Clone)]
pub enum ApplicationAction {
    Key(Modifiers, Key),
    Modifiers(Modifiers),
    AppTheme(usize),
}

impl Action for MenuAction {
    type Message = Message;
    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::Settings => Message::ToggleContextPage(ContextPage::Settings),
            action => Message::Menu(*action),
        }
    }
}
