// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::{
    widget::menu::{items, root, Item, ItemHeight, ItemWidth, MenuBar, Tree},
    Element,
};

use crate::{
    app::{Action, Message},
    fl,
};

pub fn menu_bar<'a>(key_binds: &HashMap<KeyBind, Action>) -> Element<'a, Message> {
    MenuBar::new(vec![
        Tree::with_children(
            root(fl!("file")),
            items(
                key_binds,
                vec![
                    Item::Button(fl!("new-window"), Action::WindowNew),
                    Item::Divider,
                    Item::Button(fl!("new-list"), Action::NewList),
                    Item::Divider,
                    Item::Button(fl!("quit"), Action::WindowClose),
                ],
            ),
        ),
        Tree::with_children(
            root(fl!("edit")),
            items(
                key_binds,
                vec![
                    Item::Button(fl!("rename"), Action::RenameList),
                    Item::Divider,
                    Item::Button(fl!("icon"), Action::Icon),
                    Item::Divider,
                    Item::Button(fl!("delete"), Action::DeleteList),
                ],
            ),
        ),
        Tree::with_children(
            root(fl!("view")),
            items(
                key_binds,
                vec![
                    Item::Button(fl!("menu-settings"), Action::Settings),
                    Item::Divider,
                    Item::Button(fl!("menu-about"), Action::About),
                ],
            ),
        ),
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(240))
    .spacing(4.0)
    .into()
}
