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

    /// Pin the trash entity to position 0 and place a divider below it
    /// (i.e. divider_above on the item at position 1) so it is visually
    /// separated from the list items beneath it.
    pub fn reposition_trash(&mut self) {
        self.nav.position_set(self.trash_entity, 0);
        // Sweep all entities: only the item immediately after trash (index 1)
        // gets a divider_above so the separator appears between trash and lists.
        let entities: Vec<_> = self.nav.iter().collect();
        for (i, entity) in entities.iter().enumerate() {
            self.nav.divider_above_set(*entity, i == 1);
        }
    }

    pub fn export_list(list: &List, tasks: &[crate::model::Task]) -> String {
        let markdown = list.markdown();
        let tasks_markdown: String = tasks.iter().map(Markdown::markdown).collect();
        format!("{markdown}\n{tasks_markdown}")
    }
}
