pub use crate::shared::navigation::core::{AppModel, ContextPage, Flags, Message};
use std::collections::HashMap;

use cosmic::{
    app::{self, Core},
    iced::{keyboard::Event as KeyEvent, Event, Subscription},
    widget::{self, calendar::CalendarModel, segmented_button::Entity},
    Application, ApplicationExt, Element,
};

use crate::{
    config::AppConfig,
    features::{
        favorites::{self, FavoritesMarker},
        lists::{content, List},
        reminders::reminder,
        tasks::{self, details},
        trash::{self, TrashMarker},
    },
    fl,
    shared::{
        dialogs::{DialogAction, DialogPage},
        navigation::{nav::NavMenuAction, ui},
    },
};

impl Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = Flags;
    type Message = Message;
    const APP_ID: &'static str = "dev.edfloreshz.Tasks";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, app::Task<Self::Message>) {
        AppModel::init(core, flags)
    }

    fn context_drawer(&self) -> Option<app::context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => app::context_drawer::about(
                &self.about,
                |url| Message::Open(url.to_string()),
                Message::ToggleContextDrawer,
            )
            .title(self.context_page.title()),
            ContextPage::Settings => app::context_drawer::context_drawer(
                crate::features::settings::views::settings(self),
                Message::ToggleContextDrawer,
            )
            .title(self.context_page.title()),
            ContextPage::TaskDetails => app::context_drawer::context_drawer(
                self.details.view().map(Message::Details),
                Message::ToggleContextDrawer,
            )
            .title(self.context_page.title()),
        })
    }

    fn dialog(&self) -> Option<Element<'_, Message>> {
        let dialog_page = self.dialog_pages.front()?;
        let dialog = dialog_page.view(&self.dialog_text_input);
        Some(dialog.into())
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        vec![ui::menu::menu_bar(&self)]
    }

    fn nav_context_menu(&self) -> Option<Vec<widget::menu::Tree<cosmic::Action<Self::Message>>>> {
        let items = self.nav.iter().map(|entity| {
            let favorites_index_opt = self.nav.data::<FavoritesMarker>(entity);
            let trash_index_opt = self.nav.data::<TrashMarker>(entity);
            let mut items: Vec<widget::menu::Item<NavMenuAction, String>> = Vec::with_capacity(7);

            if trash_index_opt.is_some() {
                items.push(cosmic::widget::menu::Item::Button(
                    fl!("restore-all"),
                    Some(
                        widget::icon::from_name("edit-undo-symbolic")
                            .size(14)
                            .handle(),
                    ),
                    NavMenuAction::TrashRestoreAll,
                ));
                items.push(cosmic::widget::menu::Item::Button(
                    fl!("empty-trash"),
                    Some(
                        widget::icon::from_name("user-trash-full-symbolic")
                            .size(14)
                            .handle(),
                    ),
                    NavMenuAction::TrashEmptyAll,
                ));
            } else if favorites_index_opt.is_some() {
                return items;
            } else {
                items.push(cosmic::widget::menu::Item::Button(
                    fl!("rename"),
                    Some(widget::icon::from_name("edit-symbolic").size(14).handle()),
                    NavMenuAction::Rename(entity),
                ));
                items.push(cosmic::widget::menu::Item::Button(
                    fl!("icon"),
                    Some(
                        widget::icon::from_name("face-smile-big-symbolic")
                            .size(14)
                            .handle(),
                    ),
                    NavMenuAction::SetIcon(entity),
                ));
                items.push(cosmic::widget::menu::Item::Button(
                    fl!("export"),
                    Some(
                        widget::icon::from_name("emblem-shared-symbolic")
                            .size(18)
                            .handle(),
                    ),
                    NavMenuAction::Export(entity),
                ));
                items.push(cosmic::widget::menu::Item::Button(
                    fl!("delete"),
                    Some(
                        widget::icon::from_name("user-trash-full-symbolic")
                            .size(14)
                            .handle(),
                    ),
                    NavMenuAction::Delete(entity),
                ));
            }
            items
        });

        Some(cosmic::widget::menu::nav_context(
            &HashMap::new(),
            items.collect(),
        ))
    }

    fn nav_model(&self) -> Option<&widget::segmented_button::SingleSelectModel> {
        Some(&self.nav)
    }

    fn on_escape(&mut self) -> app::Task<Self::Message> {
        if self.dialog_pages.pop_front().is_some() {
            return app::Task::none();
        }

        self.core.window.show_context = false;

        app::Task::none()
    }

    fn on_nav_select(&mut self, entity: Entity) -> app::Task<Self::Message> {
        let mut tasks = vec![];
        self.nav.activate(entity);

        if self.nav.data::<FavoritesMarker>(entity).is_some() {
            let _ = self.update(Message::Content(content::Message::SetList(None)));
            return self.update(Message::Favorites(favorites::favorites::Message::Load));
        }

        if self.nav.data::<TrashMarker>(entity).is_some() {
            return app::Task::batch(vec![
                self.update(Message::Content(content::Message::SetList(None))),
                self.update(Message::Trash(trash::trash::Message::Load)),
            ]);
        }

        let location_opt = self.nav.data::<List>(entity);

        if let Some(list) = location_opt {
            if let Err(err) = self.config.set_last_list_id(&self.handler, Some(list.id)) {
                tracing::error!("{err}");
            }
            let message = Message::Content(content::Message::SetList(Some(list.clone())));
            let window_title = format!("{} - {}", list.name, fl!("tasks"));
            if let Some(window_id) = self.core.main_window_id() {
                tasks.push(self.set_window_title(window_title, window_id));
            }
            return self.update(message);
        }

        app::Task::batch(tasks)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let mut subscriptions = vec![
            self.core()
                .watch_config::<AppConfig>(Self::APP_ID)
                .map(|update| {
                    for why in update.errors {
                        tracing::error!(?why, "app config error");
                    }

                    Message::UpdateConfig(update.config)
                }),
            cosmic::iced::event::listen_with(|event, _status, _window_id| match event {
                Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => Some(
                    Message::Application(ui::ApplicationAction::Key(modifiers, key)),
                ),
                Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => Some(
                    Message::Application(ui::ApplicationAction::Modifiers(modifiers)),
                ),
                _ => None,
            }),
        ];

        if self.trash.has_pending_deletion() {
            subscriptions.push(
                cosmic::iced::time::every(std::time::Duration::from_secs(1))
                    .map(|_| Message::Trash(trash::trash::Message::TaskDeletionTick)),
            );
        }

        subscriptions.push(
            cosmic::iced::time::every(std::time::Duration::from_secs(30))
                .map(|_| Message::Reminder(reminder::ReminderMessage::Tick)),
        );

        subscriptions.push(crate::shared::store::watcher::subscription(
            self.store.base_dir().to_path_buf(),
        ));

        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Self::Message) -> app::Task<Self::Message> {
        match message {
            Message::UpdateConfig(config) => {
                self.config = config;
                return cosmic::task::message(Message::Content(content::Message::SetConfig(
                    self.config.clone(),
                )));
            }
            Message::Open(url) => {
                if let Err(err) = open::that_detached(url) {
                    tracing::error!("{err}")
                }
            }
            Message::Content(message) => {
                if let Some(output) = self.content.update(message) {
                    match output {
                        content::Output::Focus(id) => return cosmic::widget::text_input::focus(id),
                        content::Output::OpenTaskDetails(key, id) => {
                            let Some(list_id) = self.nav.active_data::<List>().map(|list| list.id)
                            else {
                                tracing::error!("No active list found for task details");
                                return app::Task::none();
                            };

                            let task = self.store.tasks(list_id).get(id).unwrap_or_else(|err| {
                                tracing::error!("Failed to load task details: {err}");
                                tasks::task::Task::default()
                            });

                            let tasks = vec![
                                cosmic::task::message(Message::Details(details::Message::SetTask(
                                    key, task, list_id,
                                ))),
                                cosmic::task::message(Message::ToggleContextPage(
                                    ContextPage::TaskDetails,
                                )),
                            ];
                            return app::Task::batch(tasks);
                        }
                        content::Output::TaskDeleted {
                            task_id,
                            list_id,
                            title,
                        } => {
                            let mut tasks = vec![
                                cosmic::task::message(Message::Trash(trash::trash::Message::Load)),
                                self.toasts
                                    .push(
                                        widget::Toast::new(fl!(
                                            "task-moved-to-trash",
                                            title = title.as_str()
                                        ))
                                        .action(
                                            fl!("undo"),
                                            move |_id| {
                                                Message::Content(content::Message::RestoreTask(
                                                    task_id, list_id,
                                                ))
                                            },
                                        ),
                                    )
                                    .map(cosmic::Action::App),
                            ];
                            if self.core.window.show_context
                                && self.context_page == ContextPage::TaskDetails
                            {
                                tasks.push(cosmic::task::message(Message::ToggleContextPage(
                                    ContextPage::TaskDetails,
                                )));
                            }
                            return app::Task::batch(tasks);
                        }
                    }
                }
            }
            Message::Details(message) => {
                if let Some(output) = self.details.update(message) {
                    match output {
                        details::Output::DeleteTask(key) => {
                            let mut tasks: Vec<cosmic::Task<Message>> =
                                vec![cosmic::task::message(Message::Content(
                                    content::Message::OpenTaskDeletionDialog(key),
                                ))];
                            if self.core.window.show_context
                                && self.context_page == ContextPage::TaskDetails
                            {
                                tasks.push(cosmic::task::message(Message::ToggleContextPage(
                                    ContextPage::TaskDetails,
                                )));
                            }
                            return cosmic::task::batch(tasks);
                        }
                        details::Output::OpenCalendarDialog => {
                            return cosmic::task::message(Message::Dialog(DialogAction::Open(
                                DialogPage::Calendar(CalendarModel::now()),
                            )));
                        }
                        details::Output::OpenReminderDialog => {
                            let (cal, hour, minute) =
                                if let Some(ts) = self.details.task.reminder_date {
                                    let zoned = ts.to_zoned(jiff::tz::TimeZone::system());
                                    let date = zoned.date();
                                    let h = zoned.hour() as u32;
                                    let m = zoned.minute() as u32;
                                    (CalendarModel::new(date, date), h, m)
                                } else {
                                    let now = jiff::Timestamp::now()
                                        .to_zoned(jiff::tz::TimeZone::system());
                                    let date = now.date();
                                    (
                                        CalendarModel::new(date, date),
                                        now.hour() as u32,
                                        now.minute() as u32,
                                    )
                                };
                            return cosmic::task::message(Message::Dialog(DialogAction::Open(
                                DialogPage::ReminderDateTime {
                                    calendar: cal,
                                    hour,
                                    minute,
                                },
                            )));
                        }
                        details::Output::RefreshTask(task) => {
                            return cosmic::task::message(Message::Content(
                                content::Message::RefreshTask(task.clone()),
                            ));
                        }
                    }
                }
            }
            Message::Trash(msg) => {
                let output = self.trash.update(msg);
                self.refresh_trash_nav_icon();
                match output {
                    Some(trash::trash::Output::EmptyTrashRequested) => {
                        return cosmic::task::message(Message::Dialog(DialogAction::Open(
                            DialogPage::EmptyTrash,
                        )));
                    }
                    Some(trash::trash::Output::DeleteTaskRequested(task_id, title)) => {
                        return cosmic::task::message(Message::Dialog(DialogAction::Open(
                            DialogPage::DeleteTaskPermanently(task_id, title),
                        )));
                    }
                    Some(trash::trash::Output::RestoreListRequested(list_id)) => {
                        return cosmic::task::message(Message::Tasks(
                            crate::shared::navigation::nav::TasksAction::RestoreList(list_id),
                        ));
                    }
                    Some(trash::trash::Output::DeleteListRequested(list_id, title)) => {
                        return cosmic::task::message(Message::Dialog(DialogAction::Open(
                            DialogPage::DeleteListPermanently(list_id, title),
                        )));
                    }
                    Some(trash::trash::Output::RestoreTaskFromListRequested(list_id, task_id)) => {
                        return cosmic::task::message(Message::Tasks(
                            crate::shared::navigation::nav::TasksAction::RestoreTaskFromList(
                                list_id, task_id,
                            ),
                        ));
                    }
                    Some(trash::trash::Output::DeleteTaskFromListRequested(
                        list_id,
                        task_id,
                        title,
                    )) => {
                        return cosmic::task::message(Message::Dialog(DialogAction::Open(
                            DialogPage::DeleteTaskFromListPermanently(list_id, task_id, title),
                        )));
                    }
                    None => {}
                }
            }
            Message::CloseToast(id) => {
                self.toasts.remove(id);
            }
            Message::Reminder(msg) => {
                use crate::features::reminders::reminder::ReminderMessage;
                match msg {
                    ReminderMessage::Tick => {
                        let now = jiff::Timestamp::now();
                        let window_start = now
                            .checked_sub(jiff::SignedDuration::from_secs(30))
                            .unwrap_or(now);
                        let notified = reminder::check_and_notify(
                            &self.store,
                            now,
                            window_start,
                            &self.sent_reminders,
                        );
                        for key in notified {
                            self.sent_reminders.insert(key);
                        }
                    }
                }
            }
            Message::Favorites(msg) => {
                if let Some(output) = self.favorites.update(msg) {
                    match output {
                        favorites::favorites::Output::OpenTask { task, list_id } => {
                            let entity = self.nav.iter().find(|e| {
                                self.nav.data::<List>(*e).is_some_and(|l| l.id == list_id)
                            });
                            let Some(entity) = entity else {
                                tracing::error!("Nav entity not found for list {list_id}");
                                return app::Task::none();
                            };
                            self.nav.activate(entity);

                            let mut tasks = vec![cosmic::task::message(
                                Message::ToggleContextPage(ContextPage::TaskDetails),
                            )];

                            if let Some(list) = self.nav.data::<List>(entity) {
                                tasks.push(self.update(Message::Content(
                                    content::Message::SetList(Some(list.clone())),
                                )));
                            }

                            let Some(key) = self.content.find_task_key(task.id) else {
                                tracing::error!("Task key not found after loading list");
                                return app::Task::none();
                            };

                            tasks.push(cosmic::task::message(Message::Details(
                                details::Message::SetTask(key, task, list_id),
                            )));

                            return app::Task::batch(tasks);
                        }
                    }
                }
            }
            Message::Tasks(action) => {
                return self.update_tasks(action);
            }
            Message::Application(action) => {
                return self.update_application(action);
            }
            Message::Menu(action) => {
                return self.update_menu(action);
            }
            Message::Dialog(action) => {
                return self.update_dialog(action);
            }
            Message::ToggleContextPage(page) => {
                if self.context_page == page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = page;
                    self.core.window.show_context = true;
                }
            }
            Message::NavMenu(action) => {
                return self.update_nav_menu(action);
            }
            Message::ToggleContextDrawer => {
                self.core.window.show_context = !self.core.window.show_context;
            }
        }

        app::Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content = if self.nav.active_data::<TrashMarker>().is_some() {
            self.trash.view().map(Message::Trash)
        } else if self.nav.active_data::<FavoritesMarker>().is_some() {
            self.favorites.view().map(Message::Favorites)
        } else {
            self.content.view().map(Message::Content)
        };

        widget::toaster(&self.toasts, content)
    }
}
