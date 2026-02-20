// SPDX-License-Identifier: GPL-3.0-only

use directories::ProjectDirs;

mod app;
mod config;
mod error;
mod i18n;
mod model;
mod store;

fn main() -> error::Result<()> {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    // Settings for configuring the application window and iced runtime.
    let settings = cosmic::app::Settings::default().size_limits(
        cosmic::iced::Limits::NONE
            .min_width(360.0)
            .min_height(180.0),
    );

    let project = ProjectDirs::from("dev", "edfloreshz", "Tasks")
        .expect("Failed to determine project directories");

    let flags = app::Flags {
        store: store::Store::open(project.data_dir())?,
    };

    // Starts the application's event loop with `()` as the application's flags.
    cosmic::app::run::<app::AppModel>(settings, flags)?;

    Ok(())
}
