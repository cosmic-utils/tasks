use crate::app;
use crate::app::icon_cache::{IconCache, ICON_CACHE};
use crate::app::{App, Flags};
use cosmic::app::Settings;
use cosmic::iced::{Limits, Size};
use cosmic::widget::icon;
use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry},
    theme, Application,
};
use done_core::service::Services;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

pub const CONFIG_VERSION: u64 = 1;

#[derive(Clone, Default, Debug, Eq, PartialEq, Deserialize, Serialize, CosmicConfigEntry)]
pub struct Config {
    pub app_theme: AppTheme,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum AppTheme {
    Dark,
    Light,
    #[default]
    System,
}

impl AppTheme {
    pub fn theme(&self) -> theme::Theme {
        match self {
            Self::Dark => theme::Theme::dark(),
            Self::Light => theme::Theme::light(),
            Self::System => theme::system_preference(),
        }
    }
}

pub fn get_icon(name: &'static str, size: u16) -> icon::Icon {
    let mut icon_cache = ICON_CACHE.get().unwrap().lock().unwrap();
    icon_cache.get(name, size)
}

pub fn config() -> (Settings, Flags) {
    logger();
    app::localize::localize();
    icons();
    services();
    let settings = settings();
    let flags = flags();
    (settings, flags)
}

pub fn flags() -> Flags {
    let (config_handler, config) = get_config();

    let flags = Flags {
        config_handler,
        config,
    };
    flags
}

pub fn settings() -> Settings {
    let (_, config) = get_config();

    let mut settings = Settings::default();
    settings = settings.theme(config.app_theme.theme());
    settings = settings.size_limits(Limits::NONE.min_width(400.0).min_height(180.0));
    settings = settings.size(Size::new(800.0, 800.0));
    settings = settings.debug(false);
    settings
}

pub fn get_config() -> (Option<cosmic_config::Config>, Config) {
    let (config_handler, config) = match cosmic_config::Config::new(App::APP_ID, CONFIG_VERSION) {
        Ok(config_handler) => {
            let config = Config::get_entry(&config_handler).unwrap_or_else(|(errs, config)| {
                log::info!("errors loading config: {:?}", errs);
                config
            });
            (Some(config_handler), config)
        }
        Err(err) => {
            log::error!("failed to create config handler: {}", err);
            (None, Config::default())
        }
    };
    (config_handler, config)
}

pub fn logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
}

pub fn icons() {
    ICON_CACHE.get_or_init(|| Mutex::new(IconCache::new()));
}

pub fn services() {
    Services::init(App::APP_ID);
}
