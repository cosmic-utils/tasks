use cosmic::{
    app::Settings,
    iced::{Limits, Size},
    Application,
};
use std::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::app::{
    config::TasksConfig,
    icons::{IconCache, ICON_CACHE},
    localize::localize,
    Flags, Tasks,
};

pub fn init() {
    localize();
    icons();
    tracing();
    migrate(&["com.system76.CosmicTasks", "dev.edfloreshz.Orderly"]);
}

pub fn settings() -> Settings {
    Settings::default()
        .antialiasing(true)
        .client_decorations(true)
        .theme(TasksConfig::config().app_theme.theme())
        .size_limits(Limits::NONE.min_width(350.0).min_height(180.0))
        .size(Size::new(700.0, 750.0))
        .debug(false)
}

pub fn flags() -> Flags {
    let (config_handler, config) = (TasksConfig::config_handler(), TasksConfig::config());

    Flags {
        config_handler,
        config,
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
}

pub fn migrate(prev_app_ids: &[&str]) {
    for prev_app_id in prev_app_ids.iter() {
        let prev = dirs::data_local_dir().unwrap().join(prev_app_id);
        let new = dirs::data_local_dir().unwrap().join(Tasks::APP_ID);
        if prev.exists() {
            match std::fs::rename(prev, new) {
                Ok(()) => tracing::info!("migrated data to new directory"),
                Err(err) => tracing::error!("error migrating data: {:?}", err),
            }
        }
    }
}
