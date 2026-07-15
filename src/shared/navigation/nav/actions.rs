use crate::features::lists::List;
use cosmic::{widget::menu::Action, widget::segmented_button};

#[derive(Debug, Clone)]
pub enum TasksAction {
    PopulateLists(Vec<List>),
    AddList(List),
    DeleteList(Option<segmented_button::Entity>),
    RestoreList(uuid::Uuid),
    RestoreTaskFromList(uuid::Uuid, uuid::Uuid),
    FetchLists,
    NavSelect(segmented_button::Entity),
    SyncFromDisk,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NavMenuAction {
    Rename(segmented_button::Entity),
    SetIcon(segmented_button::Entity),
    Export(segmented_button::Entity),
    Delete(segmented_button::Entity),
    TrashEmptyAll,
    TrashRestoreAll,
}

impl Action for NavMenuAction {
    type Message = cosmic::Action<crate::app::Message>;

    fn message(&self) -> Self::Message {
        cosmic::Action::App(crate::app::Message::NavMenu(*self))
    }
}
