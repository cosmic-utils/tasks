use std::collections::HashSet;

use cosmic::{app, widget, Application};

use crate::{
    app::{AppModel, Message},
    features::{lists::content, lists::List},
    shared::{
        dialogs::{DialogAction, DialogPage},
        navigation::nav::TasksAction,
    },
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
                let entity = self
                    .config
                    .last_list_id
                    .and_then(|id| {
                        self.nav
                            .iter()
                            .find(|e| self.nav.data::<List>(*e).is_some_and(|l| l.id == id))
                    })
                    .or_else(|| self.nav.iter().next());
                let Some(entity) = entity else {
                    return app::Task::none();
                };
                self.nav.activate(entity);
                return cosmic::task::message(Message::Tasks(TasksAction::NavSelect(entity)));
            }
            TasksAction::AddList(list) => {
                self.create_nav_item(&list);
                self.reposition_special_items();
                let entity = self
                    .nav
                    .iter()
                    .find(|e| self.nav.data::<List>(*e).is_some_and(|l| l.id == list.id))
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

                if let Some(list) = self.nav.data::<List>(entity) {
                    let list_id = list.id;
                    let title = list.name.clone();
                    if let Err(err) = self.store.trash().trash_list(list_id) {
                        tracing::error!("Error moving list to trash: {err}");
                    }

                    self.nav.remove(entity);

                    let mut tasks = vec![
                        cosmic::task::message(Message::Content(content::Message::SetList(None))),
                        cosmic::task::message(Message::Trash(
                            crate::features::trash::trash::Message::Load,
                        )),
                        self.toasts
                            .push(
                                cosmic::widget::Toast::new(crate::fl!(
                                    "list-moved-to-trash",
                                    title = title.as_str()
                                ))
                                .action(crate::fl!("undo"), move |_id| {
                                    Message::Tasks(TasksAction::RestoreList(list_id))
                                }),
                            )
                            .map(cosmic::Action::App),
                    ];
                    return app::Task::batch(std::mem::take(&mut tasks));
                }
                self.nav.remove(self.nav.active());
            }
            TasksAction::RestoreList(list_id) => match self.store.trash().restore_list(list_id) {
                Ok(list) => {
                    self.create_nav_item(&list);
                    self.reposition_special_items();
                    return cosmic::task::message(Message::Trash(
                        crate::features::trash::trash::Message::Load,
                    ));
                }
                Err(err) => {
                    tracing::error!("Error restoring list from trash: {err}");
                }
            },
            TasksAction::RestoreTaskFromList(list_id, task_id) => {
                match self.store.trash().restore_task_from_list(list_id, task_id) {
                    Ok(list) => {
                        let exists = self
                            .nav
                            .iter()
                            .any(|e| self.nav.data::<List>(e).is_some_and(|l| l.id == list.id));
                        if !exists {
                            self.create_nav_item(&list);
                            self.reposition_special_items();
                        }
                        return cosmic::task::message(Message::Trash(
                            crate::features::trash::trash::Message::Load,
                        ));
                    }
                    Err(err) => {
                        tracing::error!("Error restoring task from trashed list: {err}");
                    }
                }
            }
            TasksAction::SyncFromDisk => {
                return self.sync_from_disk();
            }
        }

        app::Task::none()
    }

    fn sync_from_disk(&mut self) -> app::Task<Message> {
        let disk_lists = match self.store.lists().load_all() {
            Ok(lists) => lists,
            Err(err) => {
                tracing::error!("Error syncing lists: {err}");
                return app::Task::none();
            }
        };

        let disk_ids: HashSet<uuid::Uuid> = disk_lists.iter().map(|l| l.id).collect();

        let stale: Vec<_> = self
            .nav
            .iter()
            .filter(|e| {
                self.nav
                    .data::<List>(*e)
                    .is_some_and(|l| !disk_ids.contains(&l.id))
            })
            .collect();
        let removed_active = stale.contains(&self.nav.active());
        for entity in stale {
            self.nav.remove(entity);
        }

        for list in &disk_lists {
            let existing = self
                .nav
                .iter()
                .find(|e| self.nav.data::<List>(*e).is_some_and(|l| l.id == list.id));
            match existing {
                Some(entity) => {
                    self.nav.text_set(entity, list.name.clone());
                    let icon = widget::icon::from_name(
                        list.icon.as_deref().unwrap_or("view-list-symbolic"),
                    )
                    .size(16);
                    self.nav.icon_set(entity, icon.into());
                    self.nav.data_set(entity, list.clone());
                }
                None => {
                    self.create_nav_item(list);
                }
            }
        }
        self.reposition_special_items();

        let mut tasks = vec![cosmic::task::message(Message::Trash(
            crate::features::trash::trash::Message::Load,
        ))];

        if removed_active {
            tasks.push(cosmic::task::message(Message::Content(
                content::Message::SetList(None),
            )));
        } else if let Some(active_list) = self.nav.active_data::<List>().cloned() {
            match self.store.tasks(active_list.id).load_all() {
                Ok(disk_tasks) => {
                    tasks.push(cosmic::task::message(Message::Content(
                        content::Message::SyncTasks(disk_tasks),
                    )));
                }
                Err(err) => tracing::error!("Error syncing tasks: {err}"),
            }
        }

        app::Task::batch(tasks)
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
                if let Some(list) = self.nav.data::<List>(entity) {
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
                if !self.trash.is_empty() {
                    return cosmic::task::message(Message::Dialog(DialogAction::Open(
                        DialogPage::EmptyTrash,
                    )));
                }
            }
            NavMenuAction::TrashRestoreAll => {
                return cosmic::task::message(Message::Trash(
                    crate::features::trash::trash::Message::RestoreAll,
                ));
            }
        }

        app::Task::none()
    }
}
