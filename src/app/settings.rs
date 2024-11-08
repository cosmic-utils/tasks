use crate::app::icon_cache::{IconCache, ICON_CACHE};
use crate::app::Flags;
use cosmic::app::Settings;
use cosmic::iced::{Limits, Size};
use cosmic::Application;
use std::sync::Mutex;

use super::config::TasksConfig;
use super::localize::localize;
use super::Tasks;

pub fn settings() -> Settings {
    localize();
    icons();
    log();
    migrate(&["com.system76.CosmicTasks", "dev.edfloreshz.Orderly"]);

    let config = TasksConfig::config();

    Settings::default()
        .antialiasing(true)
        .client_decorations(true)
        .theme(config.app_theme.theme())
        .size_limits(Limits::NONE.min_width(350.0).min_height(180.0))
        .size(Size::new(700.0, 900.0))
        .debug(false)
}

pub fn flags() -> Flags {
    let (config_handler, config) = (TasksConfig::config_handler(), TasksConfig::config());

    Flags {
        config_handler,
        config,
    }
}

pub fn log() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
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
                Ok(()) => log::info!("migrated data to new directory"),
                Err(err) => log::error!("error migrating data: {:?}", err),
            }
        }
    }
}
