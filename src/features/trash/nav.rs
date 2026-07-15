use crate::{app::AppModel, fl};

pub struct TrashMarker;

impl AppModel {
    pub fn show_trash_nav_item(&mut self) {
        self.trash_entity = self
            .nav
            .insert()
            .text(fl!("trash"))
            .icon(self.trash.icon())
            .data(TrashMarker)
            .id();
        self.reposition_special_items();
    }

    pub fn hide_trash_nav_item(&mut self) {
        self.nav.remove(self.trash_entity);
        self.reposition_special_items();
    }

    pub fn refresh_trash_nav_icon(&mut self) {
        if self.nav.data::<TrashMarker>(self.trash_entity).is_some() {
            self.nav
                .icon_set(self.trash_entity, self.trash.icon().icon());
        }
    }
}
