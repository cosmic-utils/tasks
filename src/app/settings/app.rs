use cosmic::{
    app::Settings,
    cosmic_config,
    iced::{Limits, Size},
    Application,
};
use std::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{app::{
    config::{self, TasksConfig},
    icons::{IconCache, ICON_CACHE},
    localize::localize,
    Tasks,
}, core::storage::LocalStorage};
use crate::core::migration::{migrate_data, migrate_data_dir};

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: config::TasksConfig,
    pub storage: LocalStorage,
}

pub fn init() {
    localize();
    icons();
    tracing();
    migrate_data_dir(&["com.system76.CosmicTasks", "dev.edfloreshz.Orderly"]);
    match migrate_data() {
        Ok(()) => tracing::info!("Data migration completed successfully."),
        Err(error) => tracing::error!("Data migration failed: {:?}", error)
    }
}

pub fn storage() -> Result<LocalStorage, crate::LocalStorageError> {
    LocalStorage::new(Tasks::APP_ID)
}

pub fn settings() -> Settings {
    Settings::default()
        .antialiasing(true)
        .client_decorations(true)
        .theme(TasksConfig::config().app_theme.theme())
        .size_limits(Limits::NONE.min_width(350.0).min_height(180.0))
        .size(Size::new(850.0, 700.0))
        .debug(false)
}

pub fn flags(storage: LocalStorage) -> Flags {
    let (config_handler, config) = (TasksConfig::config_handler(), TasksConfig::config());

    Flags {
        config_handler,
        config,
        storage,
    }
}

pub fn tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub fn icons() {
    ICON_CACHE.get_or_init(|| Mutex::new(IconCache::new()));
    crate::app::icons::cache_all_icons_in_background(vec![14, 16, 18, 20, 32]);
}
