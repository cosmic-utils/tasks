use cosmic::{
    iced::keyboard::{Key, Modifiers},
    widget::menu::Action,
};

use crate::app::{ContextPage, Message};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    DeleteList,
    Icon,
    NewList,
    RenameList,
    Settings,
    SortByDateAsc,
    SortByDateDesc,
    SortByManual,
    SortByNameAsc,
    SortByNameDesc,
    ToggleHideCompleted(bool),
    ToggleHideCompletedShortcut,
    ToggleSearchBar,
    WindowClose,
    WindowNew,
}

#[derive(Clone, Debug)]
pub enum ApplicationAction {
    AppTheme(usize),
    Key(Modifiers, Key),
    Modifiers(Modifiers),
    ListSortBy(usize),
    ToggleShowFavorites(bool),
    ToggleShowTrash(bool),
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
