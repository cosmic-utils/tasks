mod app;
mod content;
mod core;
mod details;
mod todo;

use app::settings;
pub use core::Error;

pub fn main() -> cosmic::iced::Result {
    settings::init();
    cosmic::app::run::<app::Tasks>(settings::settings(), settings::flags())
}
