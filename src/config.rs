use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry},
    theme,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const CONFIG_VERSION: u64 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, CosmicConfigEntry)]
#[version = 1]
pub struct AppConfig {
    pub app_theme: AppTheme,
    pub hide_completed: bool,
    pub show_favorites: bool,
    pub show_trash: bool,
    pub sort_by: SortBy,
    pub last_list_id: Option<Uuid>,
    pub list_sort_by: ListSortBy,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_theme: AppTheme::default(),
            hide_completed: false,
            show_favorites: true,
            show_trash: true,
            sort_by: SortBy::default(),
            last_list_id: None,
            list_sort_by: ListSortBy::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum ListSortBy {
    #[default]
    NameAsc,
    NameDesc,
    Manual,
}

impl From<usize> for ListSortBy {
    fn from(value: usize) -> Self {
        match value {
            1 => ListSortBy::NameDesc,
            2 => ListSortBy::Manual,
            _ => ListSortBy::NameAsc,
        }
    }
}

impl From<ListSortBy> for usize {
    fn from(value: ListSortBy) -> Self {
        match value {
            ListSortBy::NameAsc => 0,
            ListSortBy::NameDesc => 1,
            ListSortBy::Manual => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum SortBy {
    NameAsc,
    NameDesc,
    #[default]
    DateAsc,
    DateDesc,
    Manual,
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
            Self::Dark => theme::Theme::dark(),
            Self::Light => theme::Theme::light(),
            Self::System => theme::system_preference(),
        }
    }
}
