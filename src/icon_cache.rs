// SPDX-License-Identifier: GPL-3.0-only

use cosmic::widget::icon;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IconCacheKey {
    name: &'static str,
    size: u16,
}

pub struct IconCache {
    cache: HashMap<IconCacheKey, icon::Handle>,
}

impl IconCache {
    pub fn new() -> Self {
        let mut cache = HashMap::new();

        macro_rules! bundle {
            ($name:expr, $size:expr) => {
                let data: &'static [u8] =
                    include_bytes!(concat!("../res/icons/bundled/", $name, ".svg"));
                cache.insert(
                    IconCacheKey {
                        name: $name,
                        size: $size,
                    },
                    icon::from_svg_bytes(data).symbolic(true),
                );
            };
        }

        bundle!("edit-clear-symbolic", 18);
        bundle!("folder-open-symbolic", 18);
        bundle!("go-down-symbolic", 18);
        bundle!("go-next-symbolic", 18);
        bundle!("go-up-symbolic", 18);
        bundle!("list-add-symbolic", 18);
        bundle!("object-select-symbolic", 18);
        bundle!("replace-symbolic", 18);
        bundle!("replace-all-symbolic", 18);
        bundle!("window-close-symbolic", 18);

        bundle!("paper-plane-symbolic", 18);
        bundle!("task-past-due-symbolic", 18);
        bundle!("user-trash-full-symbolic", 18);
        bundle!("mail-send-symbolic", 18);
        bundle!("applications-office-symbolic", 18);

        bundle!("flag-filled-symbolic", 16);
        bundle!("flag-outline-thick-symbolic", 16);
        bundle!("flag-outline-thin-symbolic", 16);

        Self { cache }
    }

    pub fn get(&mut self, name: &'static str, size: u16) -> icon::Icon {
        let handle = self
            .cache
            .entry(IconCacheKey { name, size })
            .or_insert_with(|| icon::from_name(name).size(size).handle())
            .clone();
        icon::icon(handle).size(size)
    }
}
