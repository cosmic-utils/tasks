use crate::{
    app::Message,
    context::ContextPage,
    core::models::List,
    dialog::{DialogAction, DialogPage},
};
use cosmic::{
    iced::keyboard::{Key, Modifiers},
    widget::{self, menu::Action as MenuAction, segmented_button},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    About,
    Settings,
    WindowClose,
    WindowNew,
    NewList,
    DeleteList,
    RenameList,
    Icon,
    ToggleHideCompleted(bool),
}

#[derive(Debug, Clone)]
pub enum ApplicationAction {
    WindowClose,
    WindowNew,
    Key(Modifiers, Key),
    Modifiers(Modifiers),
    AppTheme(usize),
    SystemThemeModeChange,
    Focus(widget::Id),
    NavMenuAction(NavMenuAction),
    Dialog(DialogAction),
    ToggleContextDrawer,
    ToggleContextPage(ContextPage),
    ToggleHideCompleted(bool),
}

#[derive(Debug, Clone)]
pub enum TasksAction {
    PopulateLists(Vec<List>),
    AddList(List),
    DeleteList(Option<segmented_button::Entity>),
    FetchLists,
}

impl MenuAction for Action {
    type Message = Message;
    fn message(&self) -> Self::Message {
        match self {
            Action::About => {
                Message::Application(ApplicationAction::ToggleContextPage(ContextPage::About))
            }
            Action::Settings => {
                Message::Application(ApplicationAction::ToggleContextPage(ContextPage::Settings))
            }
            Action::WindowClose => Message::Application(ApplicationAction::WindowClose),
            Action::WindowNew => Message::Application(ApplicationAction::WindowNew),
            Action::NewList => Message::Application(ApplicationAction::Dialog(DialogAction::Open(
                DialogPage::New(String::new()),
            ))),
            Action::Icon => Message::Application(ApplicationAction::Dialog(DialogAction::Open(
                DialogPage::Icon(None, String::new(), String::new()),
            ))),
            Action::RenameList => Message::Application(ApplicationAction::Dialog(
                DialogAction::Open(DialogPage::Rename(None, String::new())),
            )),
            Action::DeleteList => Message::Application(ApplicationAction::Dialog(
                DialogAction::Open(DialogPage::Delete(None)),
            )),
            Action::ToggleHideCompleted(value) => {
                Message::Application(ApplicationAction::ToggleHideCompleted(*value))
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NavMenuAction {
    Rename(segmented_button::Entity),
    SetIcon(segmented_button::Entity),
    Export(segmented_button::Entity),
    Delete(segmented_button::Entity),
}

impl MenuAction for NavMenuAction {
    type Message = cosmic::Action<Message>;

    fn message(&self) -> Self::Message {
        cosmic::Action::App(Message::Application(ApplicationAction::NavMenuAction(
            *self,
        )))
    }
}
