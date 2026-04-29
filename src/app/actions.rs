use crate::{
    app::{
        Message,
        context::ContextPage,
        dialog::{DialogAction, DialogPage},
    },
    storage::models::List,
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
    ImportList,
    Icon,
    ToggleHideCompleted(bool),
    SortByNameAsc,
    SortByNameDesc,
    SortByDateAsc,
    SortByDateDesc,
    SortByDueAsc,
    SortByDueDesc,
    SyncNow,
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
    SortByNameAsc,
    SortByNameDesc,
    SortByDateAsc,
    SortByDateDesc,
    SortByDueAsc,
    SortByDueDesc,
    ToggleEncryptNotes(bool),
    /// Open the portal file picker to choose a markdown file, then read +
    /// parse it. The result is delivered via `ImportFromFileResult`.
    ImportFromFile,
    /// `(filename, contents)` on success, or a short error string. The
    /// "cancelled" sentinel is treated as a no-op.
    ImportFromFileResult(Result<(String, String), String>),
    /// Open the portal Save dialog for the currently displayed Export
    /// payload and write the markdown to the chosen path.
    SaveExportToFile,
    SaveExportToFileResult(Result<std::path::PathBuf, String>),
    SetSyncServerUrl(String),
    SetSyncUsername(String),
    SetSyncPassword(String),
    TestSyncConnection,
    TestSyncConnectionResult(Result<(), String>),
    SyncNow,
    SyncTick,
    SyncResult(Result<crate::sync::engine::SyncReport, String>),
    SignOut,
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
            Action::ImportList => {
                Message::Application(ApplicationAction::ImportFromFile)
            }
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
            Action::SortByNameAsc => Message::Application(ApplicationAction::SortByNameAsc),
            Action::SortByNameDesc => Message::Application(ApplicationAction::SortByNameDesc),
            Action::SortByDateAsc => Message::Application(ApplicationAction::SortByDateAsc),
            Action::SortByDateDesc => Message::Application(ApplicationAction::SortByDateDesc),
            Action::SortByDueAsc => Message::Application(ApplicationAction::SortByDueAsc),
            Action::SortByDueDesc => Message::Application(ApplicationAction::SortByDueDesc),
            Action::SyncNow => Message::Application(ApplicationAction::SyncNow),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NavMenuAction {
    Rename(segmented_button::Entity),
    SetIcon(segmented_button::Entity),
    Export(segmented_button::Entity),
    Delete(segmented_button::Entity),
    SyncNow,
}

impl MenuAction for NavMenuAction {
    type Message = cosmic::Action<Message>;

    fn message(&self) -> Self::Message {
        cosmic::Action::App(Message::Application(ApplicationAction::NavMenuAction(
            *self,
        )))
    }
}
