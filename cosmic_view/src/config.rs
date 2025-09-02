//! Configuration management for Cosmic View
//! 
//! This module handles application settings and preferences.

use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, Config, CosmicConfigEntry},
    theme,
};
use serde::{Deserialize, Serialize};

/// Configuration version for compatibility checking
pub const CONFIG_VERSION: u64 = 1;

/// Main application configuration structure
#[derive(Clone, Default, Debug, Eq, PartialEq, Deserialize, Serialize, CosmicConfigEntry)]
pub struct CosmicViewConfig {
    /// Application theme preference
    pub app_theme: AppTheme,
    /// Last opened directory path
    pub last_directory: Option<String>,
    /// Whether to show file names in the viewer
    pub show_filenames: bool,
}

impl CosmicViewConfig {
    /// Get the configuration handler for this app
    pub fn config_handler() -> Option<Config> {
        Config::new("com.github.cosmic-view", CONFIG_VERSION).ok()
    }

    /// Load the current configuration
    pub fn config() -> CosmicViewConfig {
        match Self::config_handler() {
            Some(config_handler) => {
                CosmicViewConfig::get_entry(&config_handler).unwrap_or_else(|(errs, config)| {
                    tracing::info!("errors loading config: {:?}", errs);
                    config
                })
            }
            None => CosmicViewConfig::default(),
        }
    }
}

/// Application theme options
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum AppTheme {
    /// Use system theme preference
    #[default]
    System,
    /// Force dark theme
    Dark,
    /// Force light theme
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
    /// Get the corresponding cosmic theme
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