use slotmap::DefaultKey;

use crate::pages::content::Message;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TaskAction {
    Add,
    Complete(DefaultKey, bool),
    Expand(DefaultKey),
    Edit(DefaultKey),
    Delete(DefaultKey),
    AddSubTask(DefaultKey),
    ToggleEditMode(DefaultKey, bool),
    TitleSubmit(DefaultKey),
    TitleUpdate(DefaultKey, String),
}

impl cosmic::widget::menu::Action for TaskAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        Message::Task(*self)
    }
}
