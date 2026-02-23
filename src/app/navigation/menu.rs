use std::collections::HashMap;

use cosmic::widget;

use crate::{app::core::Message, fl};

use super::NavMenuAction;

pub fn nav_context_menu(
    id: widget::nav_bar::Id,
) -> Option<Vec<widget::menu::Tree<cosmic::Action<Message>>>> {
    Some(cosmic::widget::menu::items(
        &HashMap::new(),
        vec![
            cosmic::widget::menu::Item::Button(
                fl!("rename"),
                Some(widget::icon::from_name("edit-symbolic").size(14).handle()),
                NavMenuAction::Rename(id),
            ),
            cosmic::widget::menu::Item::Button(
                fl!("icon"),
                Some(
                    widget::icon::from_name("face-smile-big-symbolic")
                        .size(14)
                        .handle(),
                ),
                NavMenuAction::SetIcon(id),
            ),
            cosmic::widget::menu::Item::Button(
                fl!("export"),
                Some(
                    widget::icon::from_name("emblem-shared-symbolic")
                        .size(18)
                        .handle(),
                ),
                NavMenuAction::Export(id),
            ),
            cosmic::widget::menu::Item::Button(
                fl!("delete"),
                Some(
                    widget::icon::from_name("user-trash-full-symbolic")
                        .size(14)
                        .handle(),
                ),
                NavMenuAction::Delete(id),
            ),
        ],
    ))
}
