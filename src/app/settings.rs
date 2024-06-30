use crate::app::icon_cache::{IconCache, ICON_CACHE};
use crate::app::Flags;
use cosmic::app::Settings;
use cosmic::iced::{Limits, Size};
use cosmic::Application;
use std::sync::Mutex;

use super::config::TasksConfig;
use super::localize::set_localization;
use super::Tasks;

pub fn init() -> (Settings, Flags) {
    set_localization();
    set_icon_cache();
    set_logger();
    migrate("com.system76.CosmicTasks");
    migrate("dev.edfloreshz.Orderly");
    let settings = get_app_settings();
    let flags = get_flags();
    (settings, flags)
}

pub fn get_app_settings() -> Settings {
    let config = TasksConfig::config();

    let mut settings = Settings::default();
    settings = settings.theme(config.app_theme.theme());
    settings = settings.size_limits(Limits::NONE.min_width(400.0).min_height(180.0));
    settings = settings.size(Size::new(800.0, 800.0));
    settings = settings.debug(false);
    settings
}

pub fn set_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
}

pub fn set_icon_cache() {
    ICON_CACHE.get_or_init(|| Mutex::new(IconCache::new()));
}

pub fn get_flags() -> Flags {
    let (config_handler, config) = (TasksConfig::config_handler(), TasksConfig::config());

    let flags = Flags {
        config_handler,
        config,
    };
    flags
}

pub fn migrate(prev_app_id: &str) {
    let prev = dirs::data_local_dir().unwrap().join(prev_app_id);
    let new = dirs::data_local_dir().unwrap().join(Tasks::APP_ID);
    if prev.exists() {
        match std::fs::rename(prev, new) {
            Ok(_) => log::info!("migrated data to new directory"),
            Err(err) => log::error!("error migrating data: {:?}", err),
        }
    }
}
