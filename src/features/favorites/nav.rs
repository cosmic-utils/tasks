use cosmic::widget;

use crate::{app::AppModel, fl};

pub struct FavoritesMarker;

impl AppModel {
    pub fn show_favorites_nav_item(&mut self) {
        let icon = widget::icon::from_name("starred-symbolic").size(16);
        self.favorites_entity = self
            .nav
            .insert()
            .text(fl!("favorites"))
            .icon(icon)
            .data(FavoritesMarker)
            .id();
        self.reposition_special_items();
    }

    pub fn hide_favorites_nav_item(&mut self) {
        self.nav.remove(self.favorites_entity);
        self.reposition_special_items();
    }
}
