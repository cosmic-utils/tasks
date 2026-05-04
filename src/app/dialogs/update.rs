use cli_clipboard::{ClipboardContext, ClipboardProvider};

use cosmic::{app, widget};

use crate::{
    app::{
        core::{AppModel, Message},
        navigation::TasksAction,
    },
    model::List,
    pages::{content, details},
};

use super::{DialogAction, DialogPage};

impl AppModel {
    pub fn update_dialog(&mut self, action: DialogAction) -> app::Task<Message> {
        match action {
            DialogAction::Open(page) => {
                match page {
                    DialogPage::RenameList(entity, _) => {
                        let data = if let Some(entity) = entity {
                            self.nav.data::<crate::model::List>(entity)
                        } else {
                            self.nav.active_data::<crate::model::List>()
                        };
                        if let Some(list) = data {
                            self.dialog_pages
                                .push_back(DialogPage::RenameList(entity, list.name.clone()));
                        }
                    }
                    page => self.dialog_pages.push_back(page),
                }
                return cosmic::widget::text_input::focus(self.dialog_text_input.clone());
            }
            DialogAction::Update(dialog_page) => {
                self.dialog_pages[0] = dialog_page;
            }
            DialogAction::Close => {
                self.dialog_pages.pop_front();
            }
            DialogAction::Complete => {
                if let Some(dialog_page) = self.dialog_pages.pop_front() {
                    match dialog_page {
                        DialogPage::NewList(name) => {
                            let list = List::new(&name);
                            match self.store.lists().save(&list) {
                                Ok(_) => {
                                    return cosmic::task::message(Message::Tasks(
                                        TasksAction::AddList(list),
                                    ));
                                }
                                Err(err) => {
                                    tracing::error!("Error updating list: {err}");
                                }
                            }
                        }
                        DialogPage::RenameList(entity, name) => {
                            let data = if let Some(entity) = entity {
                                self.nav.data_mut::<List>(entity)
                            } else {
                                self.nav.active_data_mut::<List>()
                            };

                            if let Some(list) = data {
                                match self
                                    .store
                                    .lists()
                                    .update(list.id, |l| l.name = name.clone())
                                {
                                    Ok(_) => {
                                        list.name.clone_from(&name.clone());
                                        let list = list.clone();
                                        self.nav.text_set(self.nav.active(), name.clone());
                                        return cosmic::task::message(Message::Content(
                                            content::Message::SetList(Some(list)),
                                        ));
                                    }
                                    Err(err) => {
                                        tracing::error!("Error updating list: {err}");
                                    }
                                }
                            }
                        }
                        DialogPage::DeleteList(entity) => {
                            return cosmic::task::message(Message::Tasks(TasksAction::DeleteList(
                                entity,
                            )));
                        }
                        DialogPage::SetListIcon(entity, name, _) => {
                            let data = if let Some(entity) = entity {
                                self.nav.data::<List>(entity)
                            } else {
                                self.nav.active_data::<List>()
                            };
                            if let Some(list) = data {
                                let entity = self.nav.active();
                                self.nav.text_set(entity, list.name.clone());
                                self.nav.icon_set(
                                    entity,
                                    widget::icon::from_name(name.clone()).size(16).icon(),
                                );
                            }
                            if let Some(list) = self.nav.active_data_mut::<List>() {
                                list.icon = Some(name);
                                let list = list.clone();
                                if let Err(err) = self
                                    .store
                                    .lists()
                                    .update(list.id, |l| l.icon = list.icon.clone())
                                {
                                    tracing::error!("Error updating list: {err}");
                                }
                                return cosmic::task::message(Message::Content(
                                    content::Message::SetList(Some(list)),
                                ));
                            }
                        }
                        DialogPage::Calendar(date) => {
                            self.details
                                .update(details::Message::SetDueDate(date.selected));
                        }
                        DialogPage::Export(content) => {
                            let Ok(mut clipboard) = ClipboardContext::new() else {
                                tracing::error!("Clipboard is not available");
                                return app::Task::none();
                            };
                            if let Err(error) = clipboard.set_contents(content) {
                                tracing::error!("Error setting clipboard contents: {error}");
                            }
                        }
                    }
                }
            }
            DialogAction::None => (),
        }

        app::Task::none()
    }
}
