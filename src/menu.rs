// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::widget::menu::menu_tree::{menu_items, menu_root, MenuItem};
use cosmic::{
    widget::menu::{ItemHeight, ItemWidth, MenuBar, MenuTree},
    Element,
};

use crate::{
    app::{Action, Message},
    fl,
};

pub fn menu_bar<'a>(key_binds: &HashMap<KeyBind, Action>) -> Element<'a, Message> {
    MenuBar::new(vec![
        MenuTree::with_children(
            menu_root(fl!("file")),
            menu_items(
                key_binds,
                vec![
                    MenuItem::Button(fl!("new-window"), Action::WindowNew),
                    MenuItem::Divider,
                    MenuItem::Button(fl!("new-list"), Action::NewList),
                    MenuItem::Divider,
                    MenuItem::Button(fl!("quit"), Action::WindowClose),
                ],
            ),
        ),
        MenuTree::with_children(
            menu_root(fl!("edit")),
            menu_items(
                key_binds,
                vec![
                    MenuItem::Button(fl!("rename"), Action::RenameList),
                    MenuItem::Divider,
                    MenuItem::Button(fl!("delete"), Action::DeleteList),
                    MenuItem::Divider,
                    MenuItem::Button(fl!("icon"), Action::Icon),
                ],
            ),
        ),
        MenuTree::with_children(
            menu_root(fl!("view")),
            menu_items(
                key_binds,
                vec![
                    MenuItem::Button(fl!("menu-settings"), Action::Settings),
                    MenuItem::Divider,
                    MenuItem::Button(fl!("menu-about"), Action::About),
                ],
            ),
        ),
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(240))
    .spacing(4.0)
    .into()
}
