use crate::{
    config::AppConfig,
    features::{
        accounts::AccountsAction, favorites::favorites, lists::content,
        reminders::reminder::ReminderMessage, search::search, tasks::details, trash::trash,
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

#[derive(Clone, Debug)]
pub enum Message {
    Application(ApplicationAction),
    Accounts(AccountsAction),
    CloseToast(cosmic::widget::ToastId),
    Content(content::Message),
    Details(details::Message),
    Dialog(DialogAction),
    Favorites(favorites::Message),
    Menu(MenuAction),
    NavMenu(NavMenuAction),
    Open(String),
    Reminder(ReminderMessage),
    Search(search::Message),
    Tasks(TasksAction),
    ToggleContextDrawer,
    ToggleContextPage(ContextPage),
    Trash(trash::Message),
    UpdateConfig(AppConfig),
}
