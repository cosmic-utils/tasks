use crate::{
    config::AppConfig,
    features::{
        favorites::favorites, lists::content, reminders::reminder::ReminderMessage, tasks::details,
        trash::trash,
    },
    shared::{
        dialogs::DialogAction,
        navigation::{
            nav::{NavMenuAction, TasksAction},
            ui::{ApplicationAction, MenuAction},
        },
    },
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
    Trash(trash::Message),
    Favorites(favorites::Message),
    Reminder(ReminderMessage),
    CloseToast(cosmic::widget::ToastId),
}
