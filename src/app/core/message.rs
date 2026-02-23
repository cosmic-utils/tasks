use crate::{
    app::{
        dialogs::DialogAction,
        navigation::{NavMenuAction, TasksAction},
        ui::{ApplicationAction, MenuAction},
    },
    config::AppConfig,
    pages::{content, details},
};

use super::ContextPage;

#[derive(Debug, Clone)]
pub enum Message {
    Content(content::Message),
    Details(details::Message),
    Tasks(TasksAction),
    Menu(MenuAction),
    Dialog(DialogAction),
    NavMenu(NavMenuAction),
    Application(ApplicationAction),
    ToggleContextDrawer,
    ToggleContextPage(ContextPage),
    Open(String),
    UpdateConfig(AppConfig),
}
