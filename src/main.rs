mod actions;
mod app;
mod content;
mod context;
mod core;
mod details;
mod dialog;

use app::settings;
pub use core::Error;

pub fn main() -> cosmic::iced::Result {
    settings::init();
    cosmic::app::run::<app::Tasks>(settings::settings(), settings::flags())
}
