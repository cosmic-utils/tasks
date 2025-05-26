// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use cosmic::{
    widget::menu::{items, key_bind::KeyBind, root, Item, ItemHeight, ItemWidth, MenuBar, Tree},
    Element,
};

use crate::{
    app::icons,
    app::{Action, Message},
    fl,
};

use super::config::TasksConfig;

pub fn menu_bar<'a>(
    key_binds: &HashMap<KeyBind, Action>,
    config: &TasksConfig,
) -> Element<'a, Message> {
    MenuBar::new(vec![
        Tree::with_children(
            root(fl!("file")),
            items(
                key_binds,
                vec![
                    Item::Button(
                        fl!("new-window"),
                        Some(icons::get_handle("tabs-stack-symbolic", 14)),
                        Action::WindowNew,
                    ),
                    Item::Divider,
                    Item::Button(
                        fl!("new-list"),
                        Some(icons::get_handle("plus-square-filled-symbolic", 14)),
                        Action::NewList,
                    ),
                    Item::Divider,
                    Item::Button(
                        fl!("quit"),
                        Some(icons::get_handle("cross-small-square-filled-symbolic", 14)),
                        Action::WindowClose,
                    ),
                ],
            ),
        ),
        Tree::with_children(
            root(fl!("edit")),
            items(
                key_binds,
                vec![
                    Item::Button(
                        fl!("rename"),
                        Some(icons::get_handle("edit-symbolic", 14)),
                        Action::RenameList,
                    ),
                    Item::Divider,
                    Item::Button(
                        fl!("icon"),
                        Some(icons::get_handle("face-smile-big-symbolic", 14)),
                        Action::Icon,
                    ),
                    Item::Divider,
                    Item::Button(
                        fl!("delete"),
                        Some(icons::get_handle("user-trash-full-symbolic", 14)),
                        Action::DeleteList,
                    ),
                ],
            ),
        ),
        Tree::with_children(
            root(fl!("view")),
            items(
                key_binds,
                vec![
                    Item::Button(
                        fl!("menu-settings"),
                        Some(icons::get_handle("settings-symbolic", 14)),
                        Action::Settings,
                    ),
                    Item::Divider,
                    Item::CheckBox(
                        fl!("hide-completed"),
                        None,
                        config.hide_completed,
                        Action::ToggleHideCompleted(!config.hide_completed),
                    ),
                    Item::Divider,
                    Item::Button(
                        fl!("menu-about"),
                        Some(icons::get_handle("info-outline-symbolic", 14)),
                        Action::About,
                    ),
                ],
            ),
        ),
        Tree::with_children(
            root(fl!("sort")),
            items(
                key_binds,
                vec![
                    Item::Button(fl!("sort-name-asc"), None, Action::SortByNameAsc),
                    Item::Button(fl!("sort-name-desc"), None, Action::SortByNameDesc),
                    Item::Button(fl!("sort-date-asc"), None, Action::SortByDateAsc),
                    Item::Button(fl!("sort-date-desc"), None, Action::SortByDateDesc),
                ],
            ),
        ),
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(260))
    .spacing(4.0)
    .into()
}
