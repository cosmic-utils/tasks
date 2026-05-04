use cosmic::{app, Application};

use crate::{
    app::{
        core::{AppModel, Message},
        dialogs::{DialogAction, DialogPage},
        navigation::TasksAction,
    },
    pages::content,
};

use super::NavMenuAction;

impl AppModel {
    pub fn update_tasks(&mut self, action: TasksAction) -> app::Task<Message> {
        match action {
            TasksAction::NavSelect(entity) => {
                return self.on_nav_select(entity);
            }
            TasksAction::FetchLists => match self.store.lists().load_all() {
                Ok(lists) => {
                    return self.update(Message::Tasks(TasksAction::PopulateLists(lists)));
                }
                Err(err) => {
                    tracing::error!("Error fetching lists: {err}");
                }
            },
            TasksAction::PopulateLists(lists) => {
                for list in lists {
                    self.create_nav_item(&list);
                }
                self.reposition_special_items();
                let Some(entity) = self.nav.iter().next() else {
                    return app::Task::none();
                };
                self.nav.activate(entity);
                return cosmic::task::message(Message::Tasks(TasksAction::NavSelect(entity)));
            }
            TasksAction::AddList(list) => {
                self.create_nav_item(&list);
                self.reposition_special_items();
                // Select the newly added list, which is now second-to-last
                // (last is always trash). Find it by its data.
                let entity = self
                    .nav
                    .iter()
                    .find(|e| {
                        self.nav
                            .data::<crate::model::List>(*e)
                            .is_some_and(|l| l.id == list.id)
                    })
                    .or_else(|| self.nav.iter().last());
                let Some(entity) = entity else {
                    return app::Task::none();
                };
                return self.on_nav_select(entity);
            }
            TasksAction::DeleteList(entity) => {
                let entity = if let Some(entity) = entity {
                    entity
                } else {
                    self.nav.active()
                };

                if let Some(list) = self.nav.data::<crate::model::List>(entity) {
                    if let Err(err) = self.store.lists().delete(list.id) {
                        tracing::error!("Error deleting list: {err}");
                    }

                    self.nav.remove(entity);

                    return cosmic::task::message(Message::Content(content::Message::SetList(
                        None,
                    )));
                }
                self.nav.remove(self.nav.active());
            }
        }

        app::Task::none()
    }

    pub fn update_nav_menu(&mut self, action: NavMenuAction) -> app::Task<Message> {
        match action {
            NavMenuAction::Rename(entity) => {
                return cosmic::task::message(Message::Dialog(DialogAction::Open(
                    DialogPage::RenameList(Some(entity), String::new()),
                )))
            }
            NavMenuAction::SetIcon(entity) => {
                return cosmic::task::message(Message::Dialog(DialogAction::Open(
                    DialogPage::SetListIcon(Some(entity), String::new(), String::new()),
                )))
            }
            NavMenuAction::Export(entity) => {
                if let Some(list) = self.nav.data::<crate::model::List>(entity) {
                    match self.store.tasks(list.id).load_all() {
                        Ok(data) => {
                            let exported_markdown = Self::export_list(list, &data);
                            return cosmic::task::message(Message::Dialog(DialogAction::Open(
                                DialogPage::Export(exported_markdown),
                            )));
                        }
                        Err(err) => {
                            tracing::error!("Error fetching tasks: {err}");
                        }
                    }
                }
            }
            NavMenuAction::Delete(entity) => {
                return cosmic::task::message(Message::Dialog(DialogAction::Open(
                    DialogPage::DeleteList(Some(entity), String::new()),
                )));
            }
            NavMenuAction::TrashEmptyAll => {
                return cosmic::task::message(Message::Trash(
                    crate::pages::trash::Message::EmptyTrash,
                ));
            }
            NavMenuAction::TrashRestoreAll => {
                return cosmic::task::message(Message::Trash(
                    crate::pages::trash::Message::RestoreAll,
                ));
            }
        }

        app::Task::none()
    }
}
