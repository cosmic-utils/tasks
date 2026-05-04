use std::{env, process};

use cosmic::{app, widget::menu::Action as _, Application};

use crate::{
    app::{
        core::{AppModel, ContextPage, Message},
        dialogs::{DialogAction, DialogPage},
    },
    pages::content,
};

use super::{ApplicationAction, MenuAction};

impl AppModel {
    pub fn update_application(&mut self, action: ApplicationAction) -> app::Task<Message> {
        match action {
            ApplicationAction::AppTheme(theme) => {
                if let Err(err) = self.config.set_app_theme(&self.handler, theme.into()) {
                    tracing::error!("{err}")
                }
                return cosmic::command::set_theme(self.config.app_theme.theme());
            }
            ApplicationAction::ToggleShowFavorites(show) => {
                if let Err(err) = self.config.set_show_favorites(&self.handler, show) {
                    tracing::error!("{err}");
                }
                if show {
                    self.show_favorites_nav_item();
                } else {
                    // Remove the nav item unconditionally first.
                    self.hide_favorites_nav_item();
                    // If the user was viewing favorites, navigate them to the first list.
                    if self
                        .nav
                        .active_data::<crate::model::FavoritesMarker>()
                        .is_some()
                    {
                        let entity = self
                            .nav
                            .iter()
                            .find(|e| self.nav.data::<crate::model::List>(*e).is_some())
                            .unwrap_or_else(|| self.nav.iter().last().unwrap_or_default());
                        return self.update(crate::app::core::Message::Content(
                            crate::pages::content::Message::SetList(
                                self.nav.data::<crate::model::List>(entity).cloned(),
                            ),
                        ));
                    }
                }
            }
            ApplicationAction::Key(modifiers, key) => {
                for (key_bind, action) in self.key_binds.clone().into_iter() {
                    if key_bind.matches(modifiers, &key) {
                        return cosmic::task::message(action.message());
                    }
                }
            }
            ApplicationAction::Modifiers(modifiers) => {
                self.modifiers = modifiers;
            }
        }

        app::Task::none()
    }

    pub fn update_menu(&mut self, action: MenuAction) -> app::Task<Message> {
        match action {
            MenuAction::About => {
                return cosmic::task::message(Message::ToggleContextPage(ContextPage::About));
            }
            MenuAction::Settings => {
                return cosmic::task::message(Message::ToggleContextPage(ContextPage::Settings));
            }
            MenuAction::WindowClose => {
                if let Some(window_id) = self.core.main_window_id() {
                    return cosmic::iced::window::close(window_id);
                }
            }
            MenuAction::WindowNew => match env::current_exe() {
                Ok(exe) => match process::Command::new(&exe).spawn() {
                    Ok(_) => {}
                    Err(err) => {
                        tracing::error!("failed to execute {exe:?}: {err}");
                    }
                },
                Err(err) => {
                    tracing::error!("failed to get current executable path: {err}");
                }
            },
            MenuAction::NewList => {
                return cosmic::task::message(Message::Dialog(DialogAction::Open(
                    DialogPage::NewList(String::new()),
                )));
            }
            MenuAction::DeleteList => {
                return cosmic::task::message(Message::Dialog(DialogAction::Open(
                    DialogPage::DeleteList(None),
                )));
            }
            MenuAction::RenameList => {
                return cosmic::task::message(Message::Dialog(DialogAction::Open(
                    DialogPage::RenameList(None, String::new()),
                )));
            }
            MenuAction::Icon => {
                return cosmic::task::message(Message::Dialog(DialogAction::Open(
                    DialogPage::SetListIcon(None, String::new(), String::new()),
                )));
            }
            MenuAction::ToggleHideCompleted(completed) => {
                if let Err(err) = self.config.set_hide_completed(&self.handler, completed) {
                    tracing::error!("{err}")
                }
                return cosmic::task::message(Message::Content(content::Message::SetConfig(
                    self.config.clone(),
                )));
            }
            MenuAction::SortByNameAsc => {
                return cosmic::task::message(Message::Content(content::Message::SetSort(
                    content::SortType::NameAsc,
                )));
            }
            MenuAction::SortByNameDesc => {
                return cosmic::task::message(Message::Content(content::Message::SetSort(
                    content::SortType::NameDesc,
                )));
            }
            MenuAction::SortByDateAsc => {
                return cosmic::task::message(Message::Content(content::Message::SetSort(
                    content::SortType::DateAsc,
                )));
            }
            MenuAction::SortByDateDesc => {
                return cosmic::task::message(Message::Content(content::Message::SetSort(
                    content::SortType::DateDesc,
                )));
            }
        }

        app::Task::none()
    }
}
