// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use cosmic::{
    Element,
    iced::{Alignment, Length, widget::horizontal_rule},
    theme,
    //TODO: export iced::widget::horizontal_rule in cosmic::widget
    widget::{
        self,
        menu::{ItemHeight, ItemWidth, MenuBar, MenuTree},
    },
};

use crate::{
    app::{Action, Message},
    fl,
    key_bind::KeyBind,
};

macro_rules! menu_button {
    ($($x:expr),+ $(,)?) => (
        widget::button(
            widget::Row::with_children(
                vec![$(Element::from($x)),+]
            )
            .align_items(Alignment::Center)
        )
        .height(Length::Fixed(32.0))
        .padding([4, 16])
        .width(Length::Fill)
        .style(theme::Button::MenuItem)
    );
}

pub fn menu_bar<'a>(key_binds: &HashMap<KeyBind, Action>) -> Element<'a, Message> {
    let menu_root = |label| {
        widget::button(widget::text(label))
            .padding([4, 12])
            .style(theme::Button::MenuRoot)
    };

    let find_key = |action: &Action| -> String {
        for (key_bind, key_action) in key_binds.iter() {
            if action == key_action {
                return key_bind.to_string();
            }
        }
        String::new()
    };

    let menu_item = |label, action| {
        let key = find_key(&action);
        MenuTree::new(
            menu_button!(
                widget::text(label),
                widget::horizontal_space(Length::Fill),
                widget::text(key)
            )
                .on_press(action.message(None)),
        )
    };

    MenuBar::new(vec![
        MenuTree::with_children(
            menu_root(fl!("file")),
            vec![
                menu_item(fl!("new-window"), Action::WindowNew),
                MenuTree::new(horizontal_rule(1)),
                menu_item(fl!("quit"), Action::WindowClose),
            ],
        ),
        MenuTree::with_children(
            menu_root(fl!("edit")),
            vec![
                menu_item(fl!("new-list"), Action::NewList),
                MenuTree::new(horizontal_rule(1)),
                menu_item(fl!("delete-list"), Action::DeleteList),
            ],
        ),
        MenuTree::with_children(
            menu_root(fl!("view")),
            vec![
                menu_item(fl!("menu-settings"), Action::Settings),
                MenuTree::new(horizontal_rule(1)),
                menu_item(fl!("menu-about"), Action::About),
            ],
        ),
    ])
        .item_height(ItemHeight::Dynamic(40))
        .item_width(ItemWidth::Uniform(240))
        .spacing(4.0)
        .into()
}
