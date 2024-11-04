mod app;
mod content;
mod core;
mod details;
pub use core::Error;
mod todo;

pub fn main() -> cosmic::iced::Result {
    let (settings, flags) = app::settings::init();
    cosmic::app::run::<app::Tasks>(settings, flags)
}
