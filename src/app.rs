pub mod core;
pub mod dialogs;
pub mod navigation;
pub mod ui;

pub use core::{AppModel, ContextPage, Flags, Message};

use cosmic::{
    app::{self, Core},
    iced::{keyboard::Event as KeyEvent, Event, Subscription},
    widget::{self, calendar::CalendarModel, segmented_button::Entity},
    Application, ApplicationExt, Element,
};

use crate::{
    app::{
        dialogs::{DialogAction, DialogPage},
        ui::ApplicationAction,
    },
    config::AppConfig,
    fl,
    model::List,
    pages::{content, details},
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
                ui::views::settings(self),
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

    fn nav_context_menu(
        &self,
        id: widget::nav_bar::Id,
    ) -> Option<Vec<widget::menu::Tree<cosmic::Action<Self::Message>>>> {
        navigation::nav_context_menu(id)
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
        let location_opt = self.nav.data::<List>(entity);

        if let Some(list) = location_opt {
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
        let subscriptions = vec![
            self.core()
                .watch_config::<AppConfig>(Self::APP_ID)
                .map(|update| {
                    for why in update.errors {
                        tracing::error!(?why, "app config error");
                    }

                    Message::UpdateConfig(update.config)
                }),
            cosmic::iced::event::listen_with(|event, _status, _window_id| match event {
                Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => {
                    Some(Message::Application(ApplicationAction::Key(modifiers, key)))
                }
                Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => Some(
                    Message::Application(ApplicationAction::Modifiers(modifiers)),
                ),
                _ => None,
            }),
        ];

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
                            let Some(list_id) = self
                                .nav
                                .active_data::<crate::model::List>()
                                .map(|list| list.id)
                            else {
                                tracing::error!("No active list found for task details");
                                return app::Task::none();
                            };

                            let task = self.store.tasks(list_id).get(id).unwrap_or_else(|err| {
                                tracing::error!("Failed to load task details: {err}");
                                crate::model::Task::default()
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
                        content::Output::OpenTaskDeletionDialog(id, list_id, task_id) => {
                            return cosmic::task::message(Message::Dialog(DialogAction::Open(
                                DialogPage::DeleteTask(id, list_id, task_id),
                            )));
                        }
                        content::Output::ToggleHideCompleted(list) => {
                            if let Some(data) = self.nav.active_data_mut::<crate::model::List>() {
                                data.hide_completed = list.hide_completed;
                            }
                        }
                    }
                }
            }
            Message::Details(message) => {
                if let Some(output) = self.details.update(message) {
                    match output {
                        details::Output::OpenTaskDeletionDialog(id, list_id, task_id) => {
                            return cosmic::task::message(Message::Dialog(DialogAction::Open(
                                DialogPage::DeleteTask(id, list_id, task_id),
                            )));
                        }
                        details::Output::OpenCalendarDialog => {
                            return cosmic::task::message(Message::Dialog(DialogAction::Open(
                                DialogPage::Calendar(CalendarModel::now()),
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
        self.content.view().map(Message::Content)
    }
}
