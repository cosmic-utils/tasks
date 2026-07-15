use crate::app::AppModel;

impl AppModel {
    pub fn reposition_special_items(&mut self) {
        let mut pos: u16 = 0;
        if self.config.show_favorites {
            self.nav.position_set(self.favorites_entity, pos);
            pos += 1;
        }
        if self.config.show_trash {
            self.nav.position_set(self.trash_entity, pos);
            pos += 1;
        }
        let first_list_pos = pos;
        let entities: Vec<_> = self.nav.iter().collect();
        for (i, entity) in entities.iter().enumerate() {
            self.nav
                .divider_above_set(*entity, i == first_list_pos as usize);
        }
    }
}
