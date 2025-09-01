use crate::{
    app::{
        context::ContextPage,
        dialog::{DialogAction, DialogPage},
        Message,
    },
    storage::models::{List, Task, ChecklistItem},
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
    ToggleHideCompleted(bool),
    SortByNameAsc,
    SortByNameDesc,
    SortByDateAsc,
    SortByDateDesc,
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
}

#[derive(Debug, Clone)]
pub enum TasksAction {
    PopulateLists(Vec<List>),
    AddList(List),
    DeleteList(Option<segmented_button::Entity>),
    FetchLists,
    
    // New async variants
    FetchListsAsync,                    // Trigger async list fetch
    ListsFetched(Result<Vec<List>, String>), // Lists result
    CreateTaskAsync(Task),              // Create remote task
    TaskCreated(Result<Task, String>),  // Creation result
    UpdateTaskAsync(Task),              // Update remote task
    TaskUpdated(Result<(), String>),    // Update result
    DeleteTaskAsync(Task),              // Delete remote task
    TaskDeleted(Result<(), String>),    // Deletion result
    DeleteListAsync(List),              // Delete remote list
    ListDeleted(Result<(), String>),    // List deletion result
    
    FetchTasksAsync(List),              // Fetch tasks for a list
    TasksFetched(Result<Vec<Task>, String>), // Tasks result
    
    // Checklist actions
    AddChecklistItemAsync(String),           // item title
    ChecklistItemAdded(Result<ChecklistItem, String>), // Creation result
    UpdateChecklistItemAsync(String, String), // item_id, new_title
    ChecklistItemUpdated(Result<ChecklistItem, String>), // Update result
    ToggleChecklistItemAsync(String),        // item_id
    ChecklistItemToggled(Result<ChecklistItem, String>), // Toggle result
    DeleteChecklistItemAsync(String),        // item_id
    ChecklistItemDeleted(Result<String, String>), // Deletion result
    FetchChecklistItemsAsync(String),        // task_id
    ChecklistItemsFetched(Result<Vec<ChecklistItem>, String>), // Fetch result
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
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NavMenuAction {
    Rename(segmented_button::Entity),
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
