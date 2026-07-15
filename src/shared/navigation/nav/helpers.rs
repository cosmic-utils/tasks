use crate::{app::AppModel, config::ListSortBy, features::lists::List};

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

        let mut list_entities: Vec<_> = self
            .nav
            .iter()
            .filter(|e| self.nav.data::<List>(*e).is_some())
            .collect();
        match self.config.list_sort_by {
            ListSortBy::NameAsc | ListSortBy::NameDesc => {
                list_entities.sort_by_key(|e| {
                    self.nav
                        .data::<List>(*e)
                        .map(|l| l.name.to_lowercase())
                        .unwrap_or_default()
                });
                if self.config.list_sort_by == ListSortBy::NameDesc {
                    list_entities.reverse();
                }
            }
            ListSortBy::Manual => {
                list_entities.sort_by_key(|e| self.nav.data::<List>(*e).map(|l| l.created_at));
            }
        }
        for (i, entity) in list_entities.iter().enumerate() {
            self.nav.position_set(*entity, first_list_pos + i as u16);
        }

        let entities: Vec<_> = self.nav.iter().collect();
        for (i, entity) in entities.iter().enumerate() {
            self.nav
                .divider_above_set(*entity, i == first_list_pos as usize);
        }
    }
}
