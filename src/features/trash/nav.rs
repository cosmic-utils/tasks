use cosmic::widget;

use crate::{app::AppModel, fl};

/// Zero-sized marker struct to tag the trash nav item.
pub struct TrashMarker;

impl AppModel {
    pub fn show_trash_nav_item(&mut self) {
        let icon = widget::icon::from_name("user-trash-full-symbolic").size(16);
        self.trash_entity = self
            .nav
            .insert()
            .text(fl!("trash"))
            .icon(icon)
            .data(TrashMarker)
            .id();
        self.reposition_special_items();
    }

    pub fn hide_trash_nav_item(&mut self) {
        self.nav.remove(self.trash_entity);
        self.reposition_special_items();
    }
}
