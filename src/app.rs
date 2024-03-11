use done_core::models::list::List;
use done_core::models::task::Task;
use done_core::service::Service;
use std::any::TypeId;
use std::collections::HashMap;
use std::error::Error;
use std::{env, process};

use cosmic::app::{message, Core, Message as CosmicMessage};
use cosmic::iced::{window, Alignment, Length, Subscription};
use cosmic::widget::segmented_button;
use cosmic::{
    cosmic_config, cosmic_theme, executor, theme, widget, Application, ApplicationExt, Command,
    Element,
};

use crate::config::{AppTheme, CONFIG_VERSION};
use crate::content::Content;
use crate::details::Details;
use crate::key_bind::{key_binds, KeyBind};
use crate::{content, details, fl, menu};

pub struct App {
    core: Core,
    nav_model: segmented_button::SingleSelectModel,
    content: Content,
    details: Details,
    config_handler: Option<cosmic_config::Config>,
    config: crate::config::Config,
    app_themes: Vec<String>,
    context_page: ContextPage,
    key_binds: HashMap<KeyBind, Action>,
    selected_list: Option<List>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ContentMessage(content::Message),
    DetailsMessage(details::Message),
    ToggleContextPage(ContextPage),
    LaunchUrl(String),
    PopulateLists(Vec<List>),
    WindowClose,
    WindowNew,
    AppTheme(AppTheme),
    SystemThemeModeChange(cosmic_theme::ThemeMode),
    AddList,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextPage {
    About,
    TaskDetails,
    Settings,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::About => String::new(),
            Self::Settings => fl!("settings"),
            Self::TaskDetails => "Details".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: crate::config::Config,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    About,
    ItemDown,
    ItemUp,
    Settings,
    WindowClose,
    WindowNew,
}

impl Action {
    pub fn message(self) -> Message {
        match self {
            Action::About => Message::ToggleContextPage(ContextPage::About),
            Action::ItemDown => Message::ContentMessage(content::Message::ItemDown),
            Action::ItemUp => Message::ContentMessage(content::Message::ItemUp),
            Action::Settings => Message::ToggleContextPage(ContextPage::Settings),
            Action::WindowClose => Message::WindowClose,
            Action::WindowNew => Message::WindowNew,
        }
    }
}

impl App {
    fn update_config(&mut self) -> Command<CosmicMessage<Message>> {
        cosmic::app::command::set_theme(self.config.app_theme.theme())
    }

    fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
        let repository = "https://github.com/edfloreshz/cosmic-todo";
        let hash = env!("VERGEN_GIT_SHA");
        let short_hash: String = hash.chars().take(7).collect();
        let date = env!("VERGEN_GIT_COMMIT_DATE");
        widget::column::with_children(vec![
            widget::svg(widget::svg::Handle::from_memory(
                &include_bytes!(
                    "../res/icons/hicolor/128x128/apps/com.system76.CosmicTodo.svg"
                )[..],
            ))
                .into(),
            widget::text::title3(fl!("cosmic-todo")).into(),
            widget::button::link(repository)
                .on_press(Message::LaunchUrl(repository.to_string()))
                .padding(0)
                .into(),
            widget::button::link(fl!(
                    "git-description",
                    hash = short_hash.as_str(),
                    date = date
                ))
                .on_press(Message::LaunchUrl(format!("{}/commits/{}", repository, hash)))
                .padding(0)
                .into(),
        ])
        .align_items(Alignment::Center)
        .spacing(space_xxs)
        .width(Length::Fill)
        .into()
    }

    fn settings(&self) -> Element<Message> {
        let app_theme_selected = match self.config.app_theme {
            AppTheme::Dark => 1,
            AppTheme::Light => 2,
            AppTheme::System => 0,
        };
        widget::settings::view_column(vec![widget::settings::view_section(fl!("appearance"))
            .add(
                widget::settings::item::builder(fl!("theme")).control(widget::dropdown(
                    &self.app_themes,
                    Some(app_theme_selected),
                    move |index| {
                        Message::AppTheme(match index {
                            1 => AppTheme::Dark,
                            2 => AppTheme::Light,
                            _ => AppTheme::System,
                        })
                    },
                )),
            )
            .into()])
        .into()
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = Flags;
    type Message = Message;
    const APP_ID: &'static str = "com.system76.CosmicTodo";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(mut core: Core, flags: Self::Flags) -> (Self, Command<CosmicMessage<Self::Message>>) {
        core.nav_bar_toggle_condensed();
        let nav_model = segmented_button::ModelBuilder::default();

        let app = App {
            core,
            nav_model: nav_model.build(),
            content: Content::new(),
            details: Details::new(),
            config_handler: flags.config_handler,
            config: flags.config,
            app_themes: vec![fl!("match-desktop"), fl!("dark"), fl!("light")],
            context_page: ContextPage::Settings,
            key_binds: key_binds(),
            selected_list: None,
        };

        let commands = vec![Command::perform(fetch_lists(), |result| match result {
            Ok(data) => message::app(Message::PopulateLists(data)),
            Err(_) => message::none(),
        })];

        (app, Command::batch(commands))
    }

    fn context_drawer(&self) -> Option<Element<Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => self.about(),
            ContextPage::Settings => self.settings(),
            ContextPage::TaskDetails => self.details.view().map(Message::DetailsMessage),
        })
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![menu::menu_bar(&self.key_binds), ]
    }

    fn header_end(&self) -> Vec<Element<Self::Message>> {
        let add_list_button = widget::button::icon(
            widget::icon::from_name("list-add-symbolic")
                .size(16)
                .handle(),
        ).on_press(Message::AddList);
        vec![add_list_button.into()]
    }

    fn nav_model(&self) -> Option<&segmented_button::SingleSelectModel> {
        Some(&self.nav_model)
    }

    fn on_nav_select(
        &mut self,
        entity: segmented_button::Entity,
    ) -> Command<CosmicMessage<Self::Message>> {
        let location_opt = self.nav_model.data::<List>(entity);

        if let Some(list) = location_opt {
            self.selected_list = Some(list.clone());
            let message = Message::ContentMessage(content::Message::List(list.clone()));
            return self.update(message);
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        struct ConfigSubscription;
        struct ThemeSubscription;

        let mut subscriptions = vec![
            cosmic_config::config_subscription(
                TypeId::of::<ConfigSubscription>(),
                Self::APP_ID.into(),
                CONFIG_VERSION,
            )
            .map(|update| {
                if !update.errors.is_empty() {
                    log::info!(
                        "errors loading config {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                }
                Message::SystemThemeModeChange(update.config)
            }),
            cosmic_config::config_subscription::<_, cosmic_theme::ThemeMode>(
                TypeId::of::<ThemeSubscription>(),
                cosmic_theme::THEME_MODE_ID.into(),
                cosmic_theme::ThemeMode::version(),
            )
            .map(|update| {
                if !update.errors.is_empty() {
                    log::info!(
                        "errors loading theme mode {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                }
                Message::SystemThemeModeChange(update.config)
            }),
        ];

        subscriptions.push(
            self.content
                .subscription()
                .map(|message| Message::ContentMessage(message)),
        );

        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Self::Message) -> Command<CosmicMessage<Self::Message>> {
        // Helper for updating config values efficiently
        macro_rules! config_set {
            ($name: ident, $value: expr) => {
                match &self.config_handler {
                    Some(config_handler) => {
                        match paste::paste! { self.config.[<set_ $name>](config_handler, $value) } {
                            Ok(_) => {}
                            Err(err) => {
                                log::warn!(
                                    "failed to save config {:?}: {}",
                                    stringify!($name),
                                    err
                                );
                            }
                        }
                    }
                    None => {
                        self.config.$name = $value;
                        log::warn!(
                            "failed to save config {:?}: no config handler",
                            stringify!($name)
                        );
                    }
                }
            };
        }

        let mut commands = vec![];

        match message {
            Message::ContentMessage(message) => {
                let content_commands = self.content.update(message);
                for content_command in content_commands {
                    match content_command {
                        content::Command::GetTasks(list_id) => {
                            commands.push(Command::perform(fetch_tasks(list_id), |result| {
                                match result {
                                    Ok(data) => message::app(Message::ContentMessage(
                                        content::Message::SetItems(data),
                                    )),
                                    Err(_) => message::none(),
                                }
                            }));
                        }
                        content::Command::DisplayTask(task) => {
                            let entity =
                                self.details.priority_model.entity_at(task.priority as u16);
                            if let Some(entity) = entity {
                                self.details.priority_model.activate(entity);
                            }

                            self.details.task = Some(task.clone());
                            commands.push(
                                self.update(Message::ToggleContextPage(ContextPage::TaskDetails)),
                            );
                        }
                        content::Command::UpdateTask(task) => {
                            self.details.task = Some(task.clone());
                            let command =
                                Command::perform(update_task(task), |result| match result {
                                    Ok(_) => message::none(),
                                    Err(_) => message::none(),
                                });
                            commands.push(command);
                        }
                        content::Command::Delete(id) => {
                            if let Some(list) = &self.selected_list {
                                let command = Command::perform(
                                    delete_task(list.id.clone(), id.clone()),
                                    |result| match result {
                                        Ok(_) => message::none(),
                                        Err(_) => message::none(),
                                    },
                                );
                                commands.push(command);
                            }
                        }
                        content::Command::CreateTask(task) => {
                            let command =
                                Command::perform(create_task(task), |result| match result {
                                    Ok(_) => message::none(),
                                    Err(_) => message::none(),
                                });
                            commands.push(command);
                        }
                    }
                }
            }
            Message::DetailsMessage(message) => {
                let details_commands = self.details.update(message);
                for details_command in details_commands {
                    match details_command {
                        details::Command::Update(task) => {
                            commands.push(Command::perform(
                                update_task(task),
                                |result| match result {
                                    Ok(_) => message::none(),
                                    Err(_) => message::none(),
                                },
                            ));
                        }
                        details::Command::Rename(id, title) => {
                            commands.push(self.update(Message::ContentMessage(
                                content::Message::Rename(id.clone(), title.clone()),
                            )));
                        }
                        details::Command::Delete(_) => {}
                        details::Command::Complete(_, _) => {}
                        details::Command::Favorite(_, _) => {}
                        details::Command::PriorityActivate(id, priority) => {
                            commands.push(self.update(Message::ContentMessage(
                                content::Message::SetPriority(id.clone(), priority.clone()),
                            )));
                        }
                    }
                }
            }
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page.clone();
                    self.core.window.show_context = true;
                }
                self.set_context_title(context_page.clone().title());
            }
            Message::WindowClose => {
                return window::close(window::Id::MAIN);
            }
            Message::WindowNew => match env::current_exe() {
                Ok(exe) => match process::Command::new(&exe).spawn() {
                    Ok(_child) => {}
                    Err(err) => {
                        eprintln!("failed to execute {:?}: {}", exe, err);
                    }
                },
                Err(err) => {
                    eprintln!("failed to get current executable path: {}", err);
                }
            },
            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    log::warn!("failed to open {:?}: {}", url, err);
                }
            },
            Message::AppTheme(app_theme) => {
                config_set!(app_theme, app_theme);
                return self.update_config();
            }
            Message::SystemThemeModeChange(_) => {
                return self.update_config();
            }
            Message::PopulateLists(lists) => {
                for list in lists {
                    self.nav_model
                        .insert()
                        .text(list.name.clone())
                        .icon(widget::icon::icon(
                            widget::icon::from_name(list.clone().icon.unwrap())
                                .size(16)
                                .handle(),
                        ))
                        .data(list);
                }
            }
            Message::AddList => {
                todo!("Implement add dialog");
            }
        }

        if !commands.is_empty() {
            return Command::batch(commands);
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let mut content_column = widget::column::with_capacity(1);
        let content_view = self
            .content
            .view()
            .map(move |message| Message::ContentMessage(message));
        content_column = content_column.push(content_view);

        let content: Element<_> = content_column.into();

        content
    }
}

async fn create_task(task: Task) -> Result<(), Box<dyn Error>> {
    let mut service = Service::Computer.get_service();
    Ok(service.create_task(task).await?)
}

async fn fetch_lists() -> Result<Vec<List>, Box<dyn Error>> {
    let mut service = Service::Computer.get_service();
    Ok(service.read_lists().await.unwrap_or(vec![]))
}

async fn fetch_tasks(list_id: String) -> Result<Vec<Task>, Box<dyn Error>> {
    let mut service = Service::Computer.get_service();
    Ok(service
        .read_tasks_from_list(list_id)
        .await
        .unwrap_or(vec![]))
}

async fn update_task(task: Task) -> Result<Task, Box<dyn Error>> {
    let mut service = Service::Computer.get_service();
    Ok(service.update_task(task).await?)
}

async fn delete_task(list_id: String, task_id: String) -> Result<(), Box<dyn Error>> {
    let mut service = Service::Computer.get_service();
    Ok(service.delete_task(list_id, task_id).await?)
}
