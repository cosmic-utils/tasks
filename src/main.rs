mod app;
mod content;
mod core;
mod details;
mod todo;

use app::settings::{flags, settings};
pub use core::Error;

pub fn main() -> cosmic::iced::Result {
    cosmic::app::run::<app::Tasks>(settings(), flags())
}
