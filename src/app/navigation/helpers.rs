use cosmic::widget::{
    self,
    segmented_button::{EntityMut, SingleSelect},
};

use crate::{
    app::{core::AppModel, ui::Markdown},
    fl,
    model::List,
};

impl AppModel {
    pub fn create_nav_item(&mut self, list: &List) -> EntityMut<'_, SingleSelect> {
        let icon =
            widget::icon::from_name(list.icon.as_deref().unwrap_or("view-list-symbolic")).size(16);
        self.nav
            .insert()
            .text(list.name.clone())
            .icon(icon)
            .data(list.clone())
    }

    /// Insert the favorites nav item and store its entity.
    pub fn show_favorites_nav_item(&mut self) {
        let icon = widget::icon::from_name("starred-symbolic").size(16);
        self.favorites_entity = self
            .nav
            .insert()
            .text(fl!("favorites"))
            .icon(icon)
            .data(crate::model::FavoritesMarker)
            .id();
        self.reposition_special_items();
    }

    /// Remove the favorites nav item.
    pub fn hide_favorites_nav_item(&mut self) {
        self.nav.remove(self.favorites_entity);
        self.reposition_special_items();
    }

    /// Pin the special nav items to the top of the nav bar and place a single
    /// divider below them, above the first list item. When favorites is hidden
    /// trash sits at position 0; when shown, favorites is 0 and trash is 1.
    pub fn reposition_special_items(&mut self) {
        let first_list_pos: u16 = if self.config.show_favorites {
            self.nav.position_set(self.favorites_entity, 0);
            self.nav.position_set(self.trash_entity, 1);
            2
        } else {
            self.nav.position_set(self.trash_entity, 0);
            1
        };
        let entities: Vec<_> = self.nav.iter().collect();
        for (i, entity) in entities.iter().enumerate() {
            self.nav
                .divider_above_set(*entity, i == first_list_pos as usize);
        }
    }

    pub fn export_list(list: &List, tasks: &[crate::model::Task]) -> String {
        let markdown = list.markdown();
        let tasks_markdown: String = tasks.iter().map(Markdown::markdown).collect();
        format!("{markdown}\n{tasks_markdown}")
    }
}
