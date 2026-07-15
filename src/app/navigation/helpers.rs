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

    pub fn show_trash_nav_item(&mut self) {
        let icon = widget::icon::from_name("user-trash-full-symbolic").size(16);
        self.trash_entity = self
            .nav
            .insert()
            .text(fl!("trash"))
            .icon(icon)
            .data(crate::model::TrashMarker)
            .id();
        self.reposition_special_items();
    }

    pub fn hide_trash_nav_item(&mut self) {
        self.nav.remove(self.trash_entity);
        self.reposition_special_items();
    }

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

    pub fn export_list(list: &List, tasks: &[crate::model::Task]) -> String {
        let markdown = list.markdown();
        let tasks_markdown: String = tasks.iter().map(Markdown::markdown).collect();
        format!("{markdown}\n{tasks_markdown}")
    }
}
