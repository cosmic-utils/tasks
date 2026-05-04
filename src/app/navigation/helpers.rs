use cosmic::widget::{
    self,
    segmented_button::{EntityMut, SingleSelect},
};

use crate::{
    app::{core::AppModel, ui::Markdown},
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

    /// Pin the special nav items (favorites, trash) to the top of the nav bar
    /// and place a single divider below them, above the first list item.
    /// Called after every list insertion.
    pub fn reposition_special_items(&mut self) {
        self.nav.position_set(self.favorites_entity, 0);
        self.nav.position_set(self.trash_entity, 1);
        // Only the item at position 2 (first list) gets a divider above it.
        let entities: Vec<_> = self.nav.iter().collect();
        for (i, entity) in entities.iter().enumerate() {
            self.nav.divider_above_set(*entity, i == 2);
        }
    }

    pub fn export_list(list: &List, tasks: &[crate::model::Task]) -> String {
        let markdown = list.markdown();
        let tasks_markdown: String = tasks.iter().map(Markdown::markdown).collect();
        format!("{markdown}\n{tasks_markdown}")
    }
}
