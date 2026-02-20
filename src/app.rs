// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{config::Config, context::ContextPage, menu::MenuAction},
    fl,
    model::List,
    pages::content::{self, Content, SortType},
    services::store::Store,
};
use cosmic::{
    app::context_drawer,
    cosmic_config::{self, CosmicConfigEntry},
    iced::{
        Alignment, Length, Subscription,
        alignment::{Horizontal, Vertical},
    },
    prelude::*,
    widget::{self, about::About, menu::KeyBind, nav_bar},
};
use directories::ProjectDirs;
use std::collections::HashMap;
use uuid::Uuid;

pub mod config;
pub mod context;
pub mod error;
pub mod menu;
pub mod theme;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

pub struct AppModel {
    core: cosmic::Core,
    context_page: ContextPage,
    about: About,
    nav: nav_bar::Model,
    key_binds: HashMap<KeyBind, MenuAction>,
    config: Config,
    store: Store,
    selected_list: Option<Uuid>,
    content: Content,
    app_themes: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    UpdateConfig(Config),
    ToggleContextDrawer,
    Content(content::Message),
    AppTheme(usize),
    Menu(MenuAction),
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "dev.edfloreshz.Tasks";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Determine the project directories for the application.
        let project = ProjectDirs::from("dev", "edfloreshz", "Tasks")
            .expect("Failed to determine project directories");

        let store = Store::open(project.data_dir()).expect("Failed to open data store");

        // Create a nav bar with three page items.
        let mut nav = nav_bar::Model::default();

        // Create the about widget
        let about = About::default()
            .name(fl!("app-title"))
            .icon(widget::icon::from_svg_bytes(APP_ICON))
            .version(env!("CARGO_PKG_VERSION"))
            .links([(fl!("repository"), REPOSITORY)])
            .license(env!("CARGO_PKG_LICENSE"));

        let config = cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .map(|context| match Config::get_entry(&context) {
                Ok(config) => config,
                Err((errors, config)) => {
                    for why in errors {
                        tracing::error!(%why, "error loading app config");
                    }
                    config
                }
            })
            .unwrap_or_default();

        match store.lists().load_all() {
            Ok(lists) => {
                for list in lists {
                    let name = list.name.clone();
                    let mut entry = nav.insert().text(name);
                    if let Some(icon) = list.icon.as_deref() {
                        entry = entry.icon(widget::icon::from_name(icon));
                    }
                    entry.data(list);
                }
            }
            Err(err) => tracing::error!("failed to fetch lists: {err}"),
        }

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            about,
            nav,
            key_binds: HashMap::new(),
            config,
            store: store.clone(),
            selected_list: None,
            content: Content::new(store),
            app_themes: vec![fl!("match-desktop"), fl!("dark"), fl!("light")],
        };

        let tasks = vec![
            // Update the window title on startup.
            app.update_title(),
        ];

        (app, Task::batch(tasks))
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        vec![crate::app::menu::menu_bar(&self.key_binds, &self.config).into()]
    }

    /// Enables the COSMIC application to create a nav bar with this model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            ),
            ContextPage::Settings => {
                context_drawer::context_drawer(self.settings(), Message::ToggleContextDrawer)
            }
        })
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<'_, Self::Message> {
        let _space_s = cosmic::theme::spacing().space_s;

        let content = if self.selected_list.is_some() {
            self.content.view().into().map(Message::Content)
        } else {
            self.select_list().into()
        };

        widget::column()
            .push(content)
            .width(600)
            .height(Length::Fill)
            .apply(widget::container)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They can be dynamically
    /// stopped and started conditionally based on application state, or persist
    /// indefinitely.
    fn subscription(&self) -> Subscription<Self::Message> {
        // Add subscriptions which are always active.
        let subscriptions = vec![
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    for why in update.errors {
                        tracing::error!(?why, "app config error");
                    }

                    Message::UpdateConfig(update.config)
                }),
        ];

        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Content(content_message) => {
                let output = self.content.update(content_message);
                match output {
                    Some(content_output) => match content_output {
                        content::Output::ToggleHideCompleted(list) => {
                            if let Some(data) = self.nav.active_data_mut::<List>() {
                                data.hide_completed = list.hide_completed;
                                if let Err(err) = self.store.lists().update(list.id, |list| {
                                    list.hide_completed = data.hide_completed
                                }) {
                                    tracing::error!("Error updating list: {err}");
                                }
                            }
                        }
                    },
                    None => {}
                }
            }
            Message::Menu(menu_action) => match menu_action {
                MenuAction::File(file_action) => match file_action {
                    menu::FileAction::WindowNew => match std::env::current_exe() {
                        Ok(exe) => match std::process::Command::new(&exe).spawn() {
                            Ok(_) => {}
                            Err(err) => {
                                tracing::error!("failed to execute {exe:?}: {err}");
                            }
                        },
                        Err(err) => {
                            tracing::error!("failed to get current executable path: {err}");
                        }
                    },
                    menu::FileAction::NewList => todo!(),
                    menu::FileAction::WindowClose => {
                        if let Some(window_id) = self.core.main_window_id() {
                            return Task::batch(vec![cosmic::iced::window::close(window_id)]);
                        }
                    }
                },
                MenuAction::Edit(edit_action) => match edit_action {
                    menu::EditAction::RenameList => todo!(),
                    menu::EditAction::Icon => todo!(),
                    menu::EditAction::DeleteList => todo!(),
                },
                MenuAction::View(view_action) => match view_action {
                    menu::ViewAction::Settings => {
                        return cosmic::task::message(Message::ToggleContextPage(
                            ContextPage::Settings,
                        ));
                    }
                    menu::ViewAction::ToggleHideCompleted(hide_completed) => todo!(),
                    menu::ViewAction::About => {
                        return cosmic::task::message(Message::ToggleContextPage(
                            ContextPage::About,
                        ));
                    }
                },
                MenuAction::Sort(sort_action) => {
                    return cosmic::task::message(Message::Content(content::Message::SetSort(
                        match sort_action {
                            menu::SortAction::SortByNameAsc => SortType::NameAsc,
                            menu::SortAction::SortByNameDesc => SortType::NameDesc,
                            menu::SortAction::SortByDateAsc => SortType::DateAsc,
                            menu::SortAction::SortByDateDesc => SortType::DateDesc,
                        },
                    )));
                }
            },
            Message::AppTheme(theme) => {
                let handler = cosmic::cosmic_config::Config::new(AppModel::APP_ID, 1).ok();

                if let Some(handler) = &handler {
                    if let Err(err) = self.config.set_app_theme(handler, theme.into()) {
                        tracing::error!("{err}")
                    }

                    return Task::batch(vec![cosmic::command::set_theme(
                        self.config.app_theme.theme(),
                    )]);
                }
            }

            Message::ToggleContextDrawer => {
                self.core.window.show_context = !self.core.window.show_context;
            }

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
        }
        Task::none()
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        let mut tasks = vec![];
        // Activate the page in the model.
        self.nav.activate(id);

        tasks.push(self.update_title());
        if let Some(selected_list) = self.nav.active_data::<List>() {
            self.selected_list = Some(selected_list.id);
            tasks.push(cosmic::task::message(Message::Content(
                content::Message::SetSelectedList(Some(selected_list.clone())),
            )));
        }
        Task::batch(tasks)
    }
}

impl AppModel {
    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app-title");

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }

    pub fn settings(&self) -> impl Into<Element<'_, Message>> {
        widget::scrollable(widget::settings::section().title(fl!("appearance")).add(
            widget::settings::item::item(
                fl!("theme"),
                widget::dropdown(
                    &self.app_themes,
                    Some(self.config.app_theme.into()),
                    |theme| Message::AppTheme(theme),
                ),
            ),
        ))
    }

    pub fn select_list(&self) -> impl Into<Element<'_, Message>> {
        let spacing = cosmic::theme::spacing();

        widget::container(
            widget::column()
                .push(widget::icon::from_name("applications-office-symbolic").size(56))
                .push(widget::text::title1(fl!("no-list-selected")))
                .push(widget::text(fl!("no-list-suggestion")))
                .spacing(spacing.space_xs)
                .align_x(Alignment::Center),
        )
        .align_y(Vertical::Center)
        .align_x(Horizontal::Center)
        .height(Length::Fill)
        .width(Length::Fill)
    }
}
