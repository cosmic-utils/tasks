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

// Bundled fallbacks (copied from the `accounts` project's own assets), used
// until a provider's manifest-declared icon URL finishes downloading, or for
// any provider that doesn't declare one at all.
const GOOGLE_LOGO: &[u8] = include_bytes!("../../../res/img/google.png");
const MICROSOFT_LOGO: &[u8] = include_bytes!("../../../res/img/microsoft.png");

fn bundled_fallback_icon(provider_id: &str) -> widget::Icon {
    match provider_id {
        "google" => widget::icon::icon(widget::icon::from_raster_bytes(GOOGLE_LOGO)),
        "microsoft" => widget::icon::icon(widget::icon::from_raster_bytes(MICROSOFT_LOGO)),
        _ => widget::icon::from_name("weather-clouds-symbolic").icon(),
    }
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
/// - `Url`: use the downloaded bytes from `icon_cache` once the async fetch
///   (kicked off in `update_accounts` on `ProvidersLoaded`) completes.
/// - `Path`: load directly from disk.
/// - `ThemeName`: resolve through the platform icon theme.
///
/// Falls back to a bundled logo (or a generic symbolic icon) if the
/// provider is unknown, declares no icon, or its URL hasn't resolved yet.
pub fn provider_icon(
    providers: &[DbusProviderInfo],
    icon_cache: &HashMap<String, Vec<u8>>,
    provider_id: &str,
) -> widget::Icon {
    let icon = providers
        .iter()
        .find(|p| p.id == provider_id)
        .and_then(|p| p.icon.as_deref());

    if let Some(icon) = icon {
        match classify_icon(icon) {
            IconSource::Url => {
                if let Some(bytes) = icon_cache.get(provider_id) {
                    return widget::icon::icon(widget::icon::from_raster_bytes(bytes.clone()));
                }
            }
            IconSource::Path(path) => {
                return widget::icon::icon(widget::icon::from_path(
                    std::path::PathBuf::from(path),
                ));
            }
            IconSource::ThemeName(name) => {
                return widget::icon::from_name(name.to_string()).icon();
            }
        }
    }

    bundled_fallback_icon(provider_id)
}
