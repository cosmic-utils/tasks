use crate::app::icon_cache::{IconCache, ICON_CACHE};
use crate::app::{App, Flags};
use cosmic::app::Settings;
use cosmic::iced::{Limits, Size};
use cosmic::Application;
use done_core::service::Services;
use std::sync::Mutex;

use super::config::CosmicTasksConfig;
use super::localize::set_localization;

pub fn init() -> (Settings, Flags) {
    set_localization();
    set_icon_cache();
    set_logger();
    start_services();
    let settings = get_app_settings();
    let flags = get_flags();
    (settings, flags)
}

pub fn get_app_settings() -> Settings {
    let config = CosmicTasksConfig::config();

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

pub fn start_services() {
    Services::init(App::APP_ID);
}

pub fn get_flags() -> Flags {
    let (config_handler, config) = (
        CosmicTasksConfig::config_handler(),
        CosmicTasksConfig::config(),
    );

    let flags = Flags {
        config_handler,
        config,
    };
    flags
}
