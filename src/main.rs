mod app;
mod core;
mod pages;
mod storage;

use core::settings;

pub use app::error::*;

pub fn main() -> cosmic::iced::Result {
    settings::app::init();
    match settings::app::storage() {
        Ok(storage) => {
            cosmic::app::run::<app::Tasks>(settings::app::settings(), app::flags(storage))
        }
        Err(error) => cosmic::app::run::<settings::error::View>(
            settings::error::settings(),
            settings::error::flags(error),
        ),
    }
}
