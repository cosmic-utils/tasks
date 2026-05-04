use crate::model::List;
use cosmic::{widget::menu::Action, widget::segmented_button};

#[derive(Debug, Clone)]
pub enum TasksAction {
    PopulateLists(Vec<List>),
    AddList(List),
    DeleteList(Option<segmented_button::Entity>),
    FetchLists,
    NavSelect(segmented_button::Entity),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NavMenuAction {
    Rename(segmented_button::Entity),
    SetIcon(segmented_button::Entity),
    Export(segmented_button::Entity),
    Delete(segmented_button::Entity),
}

impl Action for NavMenuAction {
    type Message = cosmic::Action<crate::app::core::Message>;

    fn message(&self) -> Self::Message {
        cosmic::Action::App(crate::app::core::Message::NavMenu(*self))
    }
}
