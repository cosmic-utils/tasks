use cosmic::{widget::menu::Action, widget::segmented_button};

use crate::features::lists::List;

#[derive(Clone, Debug)]
pub enum TasksAction {
    AddList(List),
    DeleteList(Option<segmented_button::Entity>),
    FetchLists,
    NavSelect(segmented_button::Entity),
    PopulateLists(Vec<List>),
    RestoreList(uuid::Uuid),
    RestoreTaskFromList(uuid::Uuid, uuid::Uuid),
    SyncFromDisk,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavMenuAction {
    Delete(segmented_button::Entity),
    Export(segmented_button::Entity),
    Rename(segmented_button::Entity),
    SetIcon(segmented_button::Entity),
    TrashEmptyAll,
    TrashRestoreAll,
}

impl Action for NavMenuAction {
    type Message = cosmic::Action<crate::app::Message>;

    fn message(&self) -> Self::Message {
        cosmic::Action::App(crate::app::Message::NavMenu(*self))
    }
}
