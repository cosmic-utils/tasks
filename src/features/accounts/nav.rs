use std::collections::HashMap;

use accounts::models::{Account, DbusProviderInfo, IconSource};
use cosmic::widget;
use uuid::Uuid;

/// Tags a non-selectable nav bar entry used purely as a section label
/// grouping the lists belonging to one remote account.
pub struct AccountHeaderMarker;

/// Account holder's name for a group header, e.g. "Jane Doe". The provider
/// is conveyed by the header's logo instead of repeating it in the text.
/// Falls back to a generic label if the account is no longer known (e.g. a
/// stale group briefly visible mid-purge).
pub fn account_label(accounts: &[Account], id: Uuid) -> String {
    accounts
        .iter()
        .find(|a| a.id == id)
        .map(|a| a.display_name.clone())
        .unwrap_or_else(|| crate::fl!("account"))
}

/// Logo icon representing an account's provider in the nav bar.
///
/// The provider's manifest icon (from `DbusProviderInfo`) can be a URL, an
/// absolute path, or a freedesktop icon-theme name:
/// - `Url`: reuse the `Handle` already decoded in `icon_cache` by the async fetch.
/// - `Path`: load directly from disk.
/// - `ThemeName`: resolve through the platform icon theme.
///
/// Falls back to a generic symbolic icon if the provider is unknown,
/// declares no icon, or its URL hasn't resolved yet.
pub fn provider_icon(
    providers: &[DbusProviderInfo],
    icon_cache: &HashMap<String, widget::icon::Handle>,
    provider_id: &str,
) -> widget::Icon {
    let icon_source = providers
        .iter()
        .find(|p| p.id == provider_id)
        .and_then(|p| p.icon_source());

    if let Some(icon_source) = icon_source {
        match icon_source {
            IconSource::Url(_) => {
                if let Some(handle) = icon_cache.get(provider_id) {
                    return widget::icon::icon(handle.clone());
                }
            }
            IconSource::Path(path) => {
                return widget::icon::icon(widget::icon::from_path(path));
            }
            IconSource::ThemeName(name) => {
                return widget::icon::from_name(name).icon();
            }
        }
    }

    widget::icon::from_name("weather-clouds-symbolic").icon()
}
