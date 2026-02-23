mod app;
mod config;
mod error;
mod i18n;
mod migrations;
mod model;
mod pages;
mod services;

use cosmic::cosmic_config::CosmicConfigEntry;
pub use error::*;

use cosmic::Application;
use cosmic::{
    app::Settings,
    cosmic_config::Config,
    iced::{Limits, Size},
};
use directories::ProjectDirs;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    app::AppModel,
    config::{AppConfig, CONFIG_VERSION},
    services::store::Store,
};

pub fn main() -> Result<()> {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    // Initialize tracing for logging and debugging.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("tasks=info")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Determine the project directories for this application.
    let project = ProjectDirs::from("dev", "edfloreshz", "Tasks")
        .expect("Failed to determine project directories");

    // Run migrations to ensure old data is converted to the new format before starting the app.
    let old_base_dir = project
        .data_dir()
        .parent()
        .expect("Failed to determine previous app directory")
        .join(app::AppModel::APP_ID);

    let new_base_dir = project.data_dir();

    migrations::migrate(old_base_dir, new_base_dir)?;

    // Store is used for persistent storage of tasks and app state.
    let store = Store::open(project.data_dir())?;

    tracing::info!("Project data directory: {:?}", project.data_dir());

    // Config handler for managing the app's configuration.
    let handler = Config::new(AppModel::APP_ID, CONFIG_VERSION)?;

    // Load the app's configuration, falling back to defaults if there are errors.
    let config = AppConfig::get_entry(&handler).unwrap_or_else(|(errs, config)| {
        tracing::info!("errors loading config: {:?}", errs);
        config
    });

    // Settings for configuring the application window and iced runtime.
    let settings = Settings::default()
        .theme(config.app_theme.theme())
        .size_limits(Limits::NONE.min_width(350.0).min_height(180.0))
        .size(Size::new(850.0, 700.0))
        .debug(false);

    // Get the application flags, which include the config handler, app config, and store.
    let flags = app::Flags {
        handler,
        config,
        store,
    };

    // Run the application.
    cosmic::app::run::<app::AppModel>(settings, flags).map_err(Error::Iced)
}
