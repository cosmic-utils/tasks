use std::{env, process};
use std::collections::HashMap;

use cosmic::{Application, ApplicationExt, Command, cosmic_config, cosmic_theme, Element, executor, theme, widget};
use cosmic::app::{Core, Message as CosmicMessage};
use cosmic::iced::{Alignment, window};
use cosmic::widget::segmented_button;

use crate::{content, fl, menu};
use crate::content::Content;
use crate::key_bind::{key_binds, KeyBind};

pub struct App {
    core: Core,
    nav_model: segmented_button::SingleSelectModel,
    content_model: segmented_button::Model<segmented_button::SingleSelect>,
    config_handler: Option<cosmic_config::Config>,
    config: crate::config::Config,
    app_themes: Vec<String>,
    context_page: ContextPage,
    key_binds: HashMap<KeyBind, Action>,
}

#[derive(Debug, Clone)]
pub enum Message {
    About,
    ContentMessage(Option<segmented_button::Entity>, crate::content::Message),
    ToggleContextPage(ContextPage),
    AppTheme(crate::config::AppTheme),
    LaunchUrl(String),
    WindowClose,
    WindowNew,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextPage {
    About,
    Settings,
    Properties,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::About => String::new(),
            Self::Settings => fl!("settings"),
            Self::Properties => fl!("properties")
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
    Properties,
    Settings,
    WindowClose,
    WindowNew,
}

impl Action {
    pub fn message(self, entity_opt: Option<segmented_button::Entity>) -> Message {
        match self {
            Action::About => Message::ToggleContextPage(ContextPage::About),
            Action::ItemDown => Message::ContentMessage(entity_opt, content::Message::ItemDown),
            Action::ItemUp => Message::ContentMessage(entity_opt, content::Message::ItemUp),
            Action::Properties => Message::ToggleContextPage(ContextPage::Properties),
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
                    "../res/icons/hicolor/128x128/apps/com.system76.CosmicFiles.svg"
                )[..],
            ))
                .into(),
            widget::text::title3(fl!("cosmic-files")).into(),
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
            .into()
    }

    pub fn properties(&self) -> Element<Message> {
        widget::settings::view_column(vec![]).into()
    }

    fn settings(&self) -> Element<Message> {
        widget::settings::view_column(vec![
            widget::settings::view_section(crate::fl!("appearance"))
                .add({
                    let app_theme_selected = match self.config.app_theme {
                        crate::config::AppTheme::Dark => 1,
                        crate::config::AppTheme::Light => 2,
                        crate::config::AppTheme::System => 0,
                    };
                    widget::settings::item::builder(crate::fl!("theme")).control(widget::dropdown(
                        &self.app_themes,
                        Some(app_theme_selected),
                        move |index| {
                            Message::AppTheme(match index {
                                1 => crate::config::AppTheme::Dark,
                                2 => crate::config::AppTheme::Light,
                                _ => crate::config::AppTheme::System,
                            })
                        },
                    ))
                })
                .into(),
        ]).into()
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
        let mut nav_model = segmented_button::ModelBuilder::default();
        let app_themes = vec![crate::fl!("match-desktop"), crate::fl!("dark"), crate::fl!("light")];

        // TODO: Fetch local lists and append them to the model.

        let app = App {
            core,
            nav_model: nav_model.build(),
            content_model: segmented_button::ModelBuilder::default().build(),
            config_handler: flags.config_handler,
            config: flags.config,
            app_themes,
            context_page: ContextPage::Settings,
            key_binds: key_binds(),
        };

        let commands = Vec::new();

        (app, Command::batch(commands))
    }

    fn nav_model(&self) -> Option<&segmented_button::SingleSelectModel> {
        Some(&self.nav_model)
    }

    fn on_nav_select(&mut self, entity: segmented_button::Entity) -> Command<CosmicMessage<Self::Message>> {
        let location_opt = self.nav_model.data::<crate::content::List>(entity);

        if let Some(list) = location_opt {
            let message = Message::ContentMessage(None, crate::content::Message::List(list.clone()));
            return self.update(message);
        }

        Command::none()
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![menu::menu_bar(&self.key_binds)]
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

        match message {
            Message::About => {}
            Message::ContentMessage(entity_opt, content_message) => {
                let entity = entity_opt.unwrap_or_else(|| self.content_model.active());

                let tab_commands = match self.content_model.data_mut::<Content>(entity) {
                    Some(content) => content.update(content_message),
                    _ => Vec::new(),
                };
            }
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
                self.set_context_title(context_page.title());
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
        }
        Command::none()
    }

    fn context_drawer(&self) -> Option<Element<Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => self.about(),
            ContextPage::Settings => self.settings(),
            ContextPage::Properties => self.properties()
        })
    }

    fn view(&self) -> Element<Self::Message> {
        let mut content_column = widget::column::with_capacity(1);

        let entity = self.content_model.active();
        match self.content_model.data::<Content>(entity) {
            Some(content) => {
                let content_view = content
                    .view()
                    .map(move |message| Message::ContentMessage(Some(entity), message));
                content_column = content_column.push(content_view);
            }
            None => {
                //TODO
            }
        }

        let content: Element<_> = content_column.into();

        content
    }
}