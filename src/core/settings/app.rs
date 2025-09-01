use cosmic::{
    app::Settings,
    iced::{Limits, Size},
    Application,
};
use std::sync::Mutex;
use tracing_subscriber::EnvFilter;

use crate::{
    app::TasksApp,
    core::{
        config::TasksConfig,
        icons::{IconCache, ICON_CACHE},
        localize::localize,
    },
    storage::{
        
        LocalStorage,
    },
};

pub fn init() {
    localize();
    icons();
    tracing();
    
}

pub fn storage() -> Result<LocalStorage, crate::LocalStorageError> {
    LocalStorage::new(TasksApp::APP_ID)
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

pub fn tracing() {
    let filter = EnvFilter::from_default_env()
    .add_directive("wgpu_core=error".parse().unwrap())
    .add_directive("naga=error".parse().unwrap())
    .add_directive("cosmic_text=error".parse().unwrap())
    .add_directive("sctk=error".parse().unwrap())
    .add_directive("wgpu_hal=error".parse().unwrap())
.add_directive("iced_wgpu=error".parse().unwrap());

tracing_subscriber::fmt()
.with_env_filter(filter)
.init();
}

pub fn icons() {
    ICON_CACHE.get_or_init(|| Mutex::new(IconCache::new()));
}
