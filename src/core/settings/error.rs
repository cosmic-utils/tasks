use cosmic::{
    app::Settings,
    iced::{alignment::Horizontal, Color, Limits, Size},
    widget, Application, ApplicationExt, Core,
};

use crate::{
    core::{config::TasksConfig, icons},
    fl,
};

#[derive(Debug)]
pub struct Flags {
    pub error: crate::LocalStorageError,
}

pub struct View {
    core: Core,
    error: crate::LocalStorageError,
    cause: Option<String>,
}

impl Application for View {
    type Executor = cosmic::iced::executor::Default;
    type Flags = crate::settings::error::Flags;
    type Message = ();

    const APP_ID: &'static str = "dev.edfloreshz.Tasks";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, flags: Self::Flags) -> (Self, cosmic::app::Task<Self::Message>) {
        let cause = flags.error.cause();
        let mut app = Self {
            core,
            error: flags.error,
            cause,
        };

        let mut tasks = vec![];

        if let Some(window_id) = app.core.main_window_id() {
            tasks.push(app.set_window_title(fl!("error-title"), window_id));
        }

        (app, cosmic::app::Task::batch(tasks))
    }

    fn view(&self) -> cosmic::Element<'_, Self::Message> {
        widget::column()
            .push(widget::icon(icons::get_handle("sad-computer-symbolic", 32)).size(32))
            .push(
                widget::text(fl!("oops-something-wrong"))
                    .size(22)
                    .class(cosmic::style::Text::Color(Color::from_rgb(0.8, 0.1, 0.1))),
            )
            .push(widget::text(self.error.to_string()).size(16))
            .push(
                widget::text_input(
                    "",
                    self.cause
                        .as_deref()
                        .unwrap_or("No additional information available."),
                )
                .label(fl!("cause"))
                .size(14)
                .on_input(|_| ()),
            )
            .padding(20)
            .spacing(16)
            .align_x(Horizontal::Center)
            .into()
    }
}

pub fn settings() -> Settings {
    Settings::default()
        .antialiasing(true)
        .client_decorations(true)
        .theme(TasksConfig::config().app_theme.theme())
        .size_limits(Limits::NONE.min_width(500.0).min_height(300.0))
        .size(Size::new(500.0, 300.0))
        .debug(false)
}

pub fn flags(error: crate::LocalStorageError) -> Flags {
    Flags { error }
}
