use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry},
    theme,
};
use serde::{Deserialize, Serialize};

pub const CONFIG_VERSION: u64 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, CosmicConfigEntry)]
pub struct AppConfig {
    pub app_theme: AppTheme,
    pub hide_completed: bool,
    pub show_favorites: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_theme: AppTheme::default(),
            hide_completed: false,
            show_favorites: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum AppTheme {
    #[default]
    System,
    Dark,
    Light,
}

impl From<usize> for AppTheme {
    fn from(value: usize) -> Self {
        match value {
            1 => AppTheme::Dark,
            2 => AppTheme::Light,
            _ => AppTheme::System,
        }
    }
}

impl From<AppTheme> for usize {
    fn from(value: AppTheme) -> Self {
        match value {
            AppTheme::System => 0,
            AppTheme::Dark => 1,
            AppTheme::Light => 2,
        }
    }
}

impl AppTheme {
    pub fn theme(&self) -> theme::Theme {
        match self {
            Self::Dark => {
                let mut t = theme::system_dark();
                t.theme_type.prefer_dark(Some(true));
                t
            }
            Self::Light => {
                let mut t = theme::system_light();
                t.theme_type.prefer_dark(Some(false));
                t
            }
            Self::System => theme::system_preference(),
        }
    }
}
