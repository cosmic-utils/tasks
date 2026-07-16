use cosmic::{
    iced::keyboard::{Key, Modifiers},
    widget::menu::Action,
};

use crate::app::{ContextPage, Message};
use crate::features::lists::content;

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
    ToggleHideCompletedShortcut,
    ToggleSearchBar,
    SortByNameAsc,
    SortByNameDesc,
    SortByDateAsc,
    SortByDateDesc,
    SortByManual,
}

#[derive(Debug, Clone)]
pub enum ApplicationAction {
    Key(Modifiers, Key),
    Modifiers(Modifiers),
    AppTheme(usize),
    ToggleShowFavorites(bool),
    ToggleShowTrash(bool),
    ListSortBy(usize),
}

impl Action for MenuAction {
    type Message = Message;
    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::Settings => Message::ToggleContextPage(ContextPage::Settings),
            MenuAction::ToggleSearchBar => Message::Content(content::Message::ToggleSearchBar),
            action => Message::Menu(*action),
        }
    }
}
