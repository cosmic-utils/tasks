// SPDX-License-Identifier: GPL-3.0-only

use cosmic::widget::icon;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

pub(crate) static ICON_CACHE: OnceLock<Mutex<IconCache>> = OnceLock::new();

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct IconCacheKey {
    name: String,
    size: u16,
}

pub struct IconCacheEntry {
    pub handle: icon::Handle,
    pub _bytes: Option<Vec<u8>>,
}

pub struct IconCache {
    cache: HashMap<IconCacheKey, IconCacheEntry>,
    bundled_icons: std::collections::HashSet<String>,
}

impl IconCache {
    pub fn new() -> Self {
        let mut bundled_icons = std::collections::HashSet::new();
        let icons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("res/icons/bundled");
        if let Ok(entries) = fs::read_dir(icons_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if let Some(stripped) = name.strip_suffix(".svg") {
                        bundled_icons.insert(stripped.to_string());
                    }
                }
            }
        }
        Self {
            cache: HashMap::new(),
            bundled_icons,
        }
    }

    fn get_icon(&mut self, name: &str, size: u16) -> icon::Icon {
        let key = IconCacheKey {
            name: name.to_string(),
            size,
        };
        if let Some(entry) = self.cache.get(&key) {
            return icon::icon(entry.handle.clone()).size(size);
        }
        let (handle, bytes) = if self.bundled_icons.contains(name) {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(format!("res/icons/bundled/{}.svg", name));
            let data = fs::read(&path).expect("Failed to read bundled icon");
            let handle = icon::from_svg_bytes(data.clone()).symbolic(true);
            (handle, Some(data))
        } else {
            (icon::from_name(name).size(size).handle(), None)
        };
        self.cache.insert(
            key.clone(),
            IconCacheEntry {
                handle: handle.clone(),
                _bytes: bytes,
            },
        );
        icon::icon(handle).size(size)
    }

}

// Removed enumeration helpers used only by the icon picker dialog

pub fn get_icon(name: &str, size: u16) -> icon::Icon {
    let mut icon_cache = ICON_CACHE.get().unwrap().lock().unwrap();
    icon_cache.get_icon(name, size)
}

pub fn get_handle(name: &str, size: u16) -> icon::Handle {
    let mut icon_cache = ICON_CACHE.get().unwrap().lock().unwrap();
    let key = IconCacheKey {
        name: name.to_string(),
        size,
    };
    if let Some(entry) = icon_cache.cache.get(&key) {
        return entry.handle.clone();
    }
    let (handle, bytes) = if icon_cache.bundled_icons.contains(name) {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("res/icons/bundled/{}.svg", name));
        let data = fs::read(&path).expect("Failed to read bundled icon");
        let handle = icon::from_svg_bytes(data.clone()).symbolic(true);
        (handle, Some(data))
    } else {
        (icon::from_name(name).size(size).handle(), None)
    };
    icon_cache.cache.insert(
        key,
        IconCacheEntry {
            handle: handle.clone(),
            _bytes: bytes,
        },
    );
    handle
}
