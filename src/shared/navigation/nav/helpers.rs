use std::cmp::Ordering;
use std::collections::BTreeMap;

use cosmic::widget::segmented_button::Entity;
use uuid::Uuid;

use crate::{
    app::AppModel,
    config::ListSortBy,
    features::{accounts::AccountHeaderMarker, lists::List},
};

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

        let list_entities: Vec<Entity> = self
            .nav
            .iter()
            .filter(|e| self.nav.data::<List>(*e).is_some())
            .collect();

        // Group by account (`None` = local lists), local group always first.
        let mut groups: BTreeMap<Option<Uuid>, Vec<Entity>> = BTreeMap::new();
        for entity in list_entities {
            let account_id = self
                .nav
                .data::<List>(entity)
                .and_then(|l| l.source.account_id());
            groups.entry(account_id).or_default().push(entity);
        }

        let mut ordered: Vec<(Option<Uuid>, String, Vec<Entity>)> = groups
            .into_iter()
            .map(|(account_id, entities)| {
                let label = account_id
                    .map(|id| crate::features::accounts::account_label(&self.remote_accounts, id))
                    .unwrap_or_default();
                (account_id, label, entities)
            })
            .collect();
        ordered.sort_by(|a, b| match (a.0, b.0) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(_), Some(_)) => a.1.cmp(&b.1),
        });

        // Drop headers for accounts that no longer have a group; the rest are updated in place.
        let current_account_ids: std::collections::HashSet<Uuid> = ordered
            .iter()
            .filter_map(|(account_id, _, _)| *account_id)
            .collect();
        let stale_headers: Vec<Uuid> = self
            .account_header_entities
            .keys()
            .filter(|id| !current_account_ids.contains(id))
            .copied()
            .collect();
        for account_id in stale_headers {
            if let Some(entity) = self.account_header_entities.remove(&account_id) {
                self.nav.remove(entity);
            }
        }

        let mut pos = first_list_pos;
        for (account_id, label, mut entities) in ordered {
            self.sort_list_entities(&mut entities);

            if let Some(account_id) = account_id {
                let provider = self
                    .remote_accounts
                    .iter()
                    .find(|a| a.id == account_id)
                    .map(|a| a.provider.as_str())
                    .unwrap_or_default();

                let header = match self.account_header_entities.get(&account_id) {
                    Some(&entity) => {
                        self.nav.text_set(entity, label.clone());
                        entity
                    }
                    None => {
                        let entity = self
                            .nav
                            .insert()
                            .text(label.clone())
                            .data(AccountHeaderMarker)
                            .id();
                        self.nav.enable(entity, false);
                        self.account_header_entities.insert(account_id, entity);
                        entity
                    }
                };
                // Refreshed every call so a provider icon is picked up once its fetch completes.
                let icon = crate::features::accounts::provider_icon(
                    &self.providers,
                    &self.provider_icons,
                    provider,
                )
                .size(16);
                self.nav.icon_set(header, icon);

                self.nav.position_set(header, pos);
                pos += 1;
            }

            for entity in entities {
                self.nav.position_set(entity, pos);
                pos += 1;
            }
        }

        // Dividers: above the first list-area entity (separating it from
        // favorites/trash), and above every account header (separating each
        // group from the one before it).
        let entities: Vec<Entity> = self.nav.iter().collect();
        for (i, entity) in entities.iter().enumerate() {
            let is_header = self.nav.data::<AccountHeaderMarker>(*entity).is_some();
            self.nav
                .divider_above_set(*entity, is_header || i == first_list_pos as usize);
        }
    }

    fn sort_list_entities(&self, entities: &mut [Entity]) {
        match self.config.list_sort_by {
            ListSortBy::NameAsc | ListSortBy::NameDesc => {
                entities.sort_by_key(|e| {
                    self.nav
                        .data::<List>(*e)
                        .map(|l| l.name.to_lowercase())
                        .unwrap_or_default()
                });
                if self.config.list_sort_by == ListSortBy::NameDesc {
                    entities.reverse();
                }
            }
            ListSortBy::Manual => {
                entities.sort_by_key(|e| self.nav.data::<List>(*e).map(|l| l.created_at));
            }
        }
    }
}
