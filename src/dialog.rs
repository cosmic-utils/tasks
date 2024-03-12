// Copyright 2023 System76 <inflist_o@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    app::{self, Command, Core, cosmic::Cosmic},
    Application, ApplicationExt,
    cosmic_theme,
    Element, executor, iced::{
        event,
        Event,
        keyboard::{Event as KeyEvent, Modifiers},
        multi_window::Application as IcedApplication,
        Size, subscription::Subscription, window,
    }, theme, widget,
};
use cosmic::iced::{Alignment, Length};

use crate::fl;

#[derive(Clone, Debug)]
pub struct DialogMessage(app::Message<Message>);

#[derive(Clone, Debug)]
pub enum DialogResult {
    Cancel,
    Ok(String),
}

#[derive(Clone, Debug)]
pub enum DialogKind {
    Confirm(String, String),
}

impl DialogKind {
    pub fn title(&self) -> String {
        match self {
            Self::Confirm(_, _) => fl!("confirm"),
        }
    }
}

pub struct Dialog<M> {
    cosmic: Cosmic<App>,
    mapper: fn(DialogMessage) -> M,
    on_result: Box<dyn Fn(DialogResult) -> M>,
}

impl<M: Send + 'static> Dialog<M> {
    pub fn new(
        kind: DialogKind,
        mapper: fn(DialogMessage) -> M,
        on_result: impl Fn(DialogResult) -> M + 'static,
    ) -> (Self, Command<M>) {
        //TODO: only do this once somehow?
        crate::localize::localize();

        let mut settings = window::Settings::default();
        settings.decorations = false;
        settings.exit_on_close_request = false;
        settings.transparent = true;

        //TODO: allow resize!
        settings.size = Size::new(500.0, 300.0);
        settings.resizable = false;

        #[cfg(target_os = "linux")]
        {
            settings.platform_specific.application_id = App::APP_ID.to_string();
        }

        let (window_id, window_command) = window::spawn(settings);

        let core = Core::default();
        let flags = Flags { kind, window_id };
        let (cosmic, cosmic_command) = <Cosmic<App> as IcedApplication>::new((core, flags));

        (
            Self {
                cosmic,
                mapper,
                on_result: Box::new(on_result),
            },
            Command::batch([window_command, cosmic_command])
                .map(DialogMessage)
                .map(move |message| app::Message::App(mapper(message))),
        )
    }

    pub fn subscription(&self) -> Subscription<M> {
        self.cosmic
            .subscription()
            .map(DialogMessage)
            .map(self.mapper)
    }

    pub fn update(&mut self, message: DialogMessage) -> Command<M> {
        let mapper = self.mapper;
        let command = self
            .cosmic
            .update(message.0)
            .map(DialogMessage)
            .map(move |message| app::Message::App(mapper(message)));
        if let Some(result) = self.cosmic.app.result_opt.take() {
            let on_result_message = (self.on_result)(result);
            Command::batch([
                command,
                Command::perform(async move { app::Message::App(on_result_message) }, |x| x),
            ])
        } else {
            command
        }
    }

    pub fn view(&self, window_id: window::Id) -> Element<M> {
        self.cosmic
            .view(window_id)
            .map(DialogMessage)
            .map(self.mapper)
    }
}

#[derive(Clone, Debug)]
struct Flags {
    kind: DialogKind,
    window_id: window::Id,
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
enum Message {
    Modifiers(Modifiers),
    Cancel,
    Ok,
    InputChanged(String),
}

/// The [`App`] stores application-specific state.
struct App {
    core: Core,
    flags: Flags,
    result_opt: Option<DialogResult>,
    modifiers: Modifiers,
    input: String,
}

impl App {
    fn update_title(&mut self) -> Command<Message> {
        let title = self.flags.kind.title();
        self.set_header_title(title.clone());
        self.set_window_title(title, self.main_window_id())
    }

    fn confirm_view(&self, title: String, message: String) -> Element<Message> {
        let cosmic_theme::Spacing { space_s, .. } = theme::active().cosmic().spacing;

        let body = widget::column::with_children(vec![
            widget::text(title.clone()).size(24).into(),
            widget::text(message.clone()).into(),
        ])
            .align_items(Alignment::Center)
            .spacing(space_s)
            .into();

        let input = widget::text_input("", &self.input)
            .on_input(Message::InputChanged)
            .into();

        let actions = widget::row::with_children(vec![
            widget::button::standard(fl!("cancel"))
                .on_press(Message::Cancel)
                .into(),
            widget::button::suggested(fl!("ok"))
                .on_press(Message::Ok)
                .into(),
        ])
            .height(Length::Fill)
            .align_items(Alignment::Center)
            .spacing(space_s)
            .into();

        widget::column::with_children(vec![body, input, actions])
            .align_items(Alignment::Center)
            .width(Length::Fill)
            .spacing(space_s)
            .padding(space_s)
            .into()
    }
}

/// Implement [`Application`] to integrate with COSMIC.
impl Application for App {
    /// Default async executor to use with the app.
    type Executor = executor::Default;

    /// Argument received
    type Flags = Flags;

    /// Message type specific to our [`App`].
    type Message = Message;

    /// The unique application ID to supply to the window manager.
    const APP_ID: &'static str = "com.system76.CosmicDialog";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Creates the application, and optionally emits command on initialize.
    fn init(mut core: Core, flags: Self::Flags) -> (Self, Command<Message>) {
        core.window.show_maximize = false;
        core.window.show_minimize = false;
        core.nav_bar_toggle_condensed();

        let mut app = App {
            core,
            flags,
            result_opt: None,
            modifiers: Modifiers::default(),
            input: String::new(),
        };

        let commands = Command::batch([app.update_title()]);

        (app, commands)
    }

    fn main_window_id(&self) -> window::Id {
        self.flags.window_id
    }

    fn on_app_exit(&mut self) {
        self.result_opt = Some(DialogResult::Cancel);
    }

    fn on_escape(&mut self) -> Command<Message> {
        self.update(Message::Cancel)
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([event::listen_with(|event, _status| match event {
            Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
                Some(Message::Modifiers(modifiers))
            }
            _ => None,
        })])
    }

    /// Handle application events here.
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Cancel => {
                self.result_opt = Some(DialogResult::Cancel);
                return window::close(self.main_window_id());
            }
            Message::Ok => {
                self.result_opt = Some(DialogResult::Ok(self.input.clone()));
                return window::close(self.main_window_id());
            }
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
            }
            Message::InputChanged(text) => {
                self.input = text;
            }
        }

        Command::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Message> {
        let content = match &self.flags.kind {
            DialogKind::Confirm(title, message) => {
                self.confirm_view(title.clone(), message.clone())
            }
        };

        content
    }
}
