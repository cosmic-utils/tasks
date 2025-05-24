mod actions;
mod app;
mod content;
mod context;
mod core;
mod details;
mod dialog;
mod error;

use app::settings;
pub use error::*;

pub fn main() -> cosmic::iced::Result {
    settings::app::init();
    match settings::app::storage() {
        Ok(storage) => {
            cosmic::app::run::<app::Tasks>(settings::app::settings(), settings::app::flags(storage))
        }
        Err(error) => cosmic::app::run::<settings::error::View>(
            settings::error::settings(),
            settings::error::flags(error),
        ),
    }
}
