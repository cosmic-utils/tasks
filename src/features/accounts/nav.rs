use std::collections::HashMap;

use accounts::models::{Account, DbusProviderInfo};
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

/// The three shapes a provider manifest's `icon` string can take. Mirrors
/// `accounts-ui`'s own classification (`accounts-ui/src/app.rs`): pending
/// upstream is a shared `IconSource`/`DbusProviderInfo::icon_source()` helper
/// in the `accounts` crate itself (not yet on its published `main`), which
/// this should be replaced with once that lands, to avoid the two consumers
/// classifying the same string independently.
enum IconSource<'a> {
    Url,
    Path(&'a str),
    ThemeName(&'a str),
}

fn classify_icon(icon: &str) -> IconSource<'_> {
    if icon.starts_with("http://") || icon.starts_with("https://") {
        IconSource::Url
    } else if icon.starts_with('/') {
        IconSource::Path(icon)
    } else {
        IconSource::ThemeName(icon)
    }
}

/// Logo icon representing an account's provider in the nav bar.
///
/// `list_providers()` (`AccountsClient`) exposes each provider's manifest
/// icon (a URL, an absolute path, or a freedesktop icon-theme name) via
/// `DbusProviderInfo::icon` to any D-Bus consumer — that's the source of
/// truth, not a Tasks-specific choice:
/// - `Url`: reuse the `Handle` already decoded once in `icon_cache` by the
///   async fetch (kicked off in `update_accounts` on `ProvidersLoaded`).
///   Must not re-decode the bytes here on every call — see the
///   `AppModel::provider_icons` field doc for why that caused flicker.
/// - `Path`: load directly from disk (stable, path-hashed `Handle::Id`, no
///   caching needed).
/// - `ThemeName`: resolve through the platform icon theme (same as above).
///
/// Falls back to a generic symbolic icon if the provider is unknown,
/// declares no icon, or its URL hasn't resolved yet.
pub fn provider_icon(
    providers: &[DbusProviderInfo],
    icon_cache: &HashMap<String, widget::icon::Handle>,
    provider_id: &str,
) -> widget::Icon {
    let icon = providers
        .iter()
        .find(|p| p.id == provider_id)
        .and_then(|p| p.icon.as_deref());

    if let Some(icon) = icon {
        match classify_icon(icon) {
            IconSource::Url => {
                if let Some(handle) = icon_cache.get(provider_id) {
                    return widget::icon::icon(handle.clone());
                }
            }
            IconSource::Path(path) => {
                return widget::icon::icon(widget::icon::from_path(std::path::PathBuf::from(path)));
            }
            IconSource::ThemeName(name) => {
                return widget::icon::from_name(name.to_string()).icon();
            }
        }
    }

    widget::icon::from_name("weather-clouds-symbolic").icon()
}
