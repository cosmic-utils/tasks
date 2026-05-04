use cosmic::{app, Application};

use crate::{
    app::{
        core::{AppModel, Message},
        dialogs::{DialogAction, DialogPage},
        navigation::TasksAction,
        ContextPage,
    },
    pages::content,
};

use super::NavMenuAction;

impl AppModel {
    pub fn update_tasks(&mut self, action: TasksAction) -> app::Task<Message> {
        match action {
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
                let Some(entity) = self.nav.iter().next() else {
                    return app::Task::none();
                };
                self.nav.activate(entity);
                return self.on_nav_select(entity);
            }
            TasksAction::AddList(list) => {
                self.create_nav_item(&list);
                let Some(entity) = self.nav.iter().last() else {
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
            TasksAction::DeleteTask(id, list_id, task_id) => {
                if let Err(err) = self.store.tasks(list_id).delete(task_id) {
                    tracing::error!("Error deleting task: {err}");
                }

                let mut tasks: Vec<cosmic::Task<Message>> = vec![cosmic::task::message(
                    Message::Content(content::Message::TaskDelete(id)),
                )];

                if self.core.window.show_context {
                    tasks.push(cosmic::task::message(Message::ToggleContextPage(
                        ContextPage::TaskDetails,
                    )));
                }

                return cosmic::task::batch(tasks);
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
                    DialogPage::DeleteList(Some(entity)),
                )));
            }
        }

        app::Task::none()
    }
}
