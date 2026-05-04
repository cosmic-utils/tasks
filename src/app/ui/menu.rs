// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    widget::{
        self,
        menu::{items, root, Item, ItemHeight, ItemWidth, MenuBar, Tree},
    },
    Element,
};

use crate::{
    app::{core::Message, ui::MenuAction, AppModel},
    fl,
    model::List,
};

pub fn menu_bar<'a>(state: &AppModel) -> Element<'a, Message> {
    let list_selected = state.nav.active_data::<List>().is_some();

    MenuBar::new(vec![
        Tree::with_children(
            Element::from(root(fl!("file"))),
            items(
                &state.key_binds,
                vec![
                    Item::Button(
                        fl!("new-window"),
                        Some(
                            widget::icon::from_name("new-window-symbolic")
                                .size(14)
                                .handle(),
                        ),
                        MenuAction::WindowNew,
                    ),
                    Item::Divider,
                    Item::Button(
                        fl!("new-list"),
                        Some(
                            widget::icon::from_name("list-add-symbolic")
                                .size(14)
                                .handle(),
                        ),
                        MenuAction::NewList,
                    ),
                    Item::Divider,
                    Item::Button(
                        fl!("quit"),
                        Some(
                            widget::icon::from_name("application-exit-symbolic")
                                .size(14)
                                .handle(),
                        ),
                        MenuAction::WindowClose,
                    ),
                ],
            ),
        ),
        Tree::with_children(
            Element::from(root(fl!("edit"))),
            items(
                &state.key_binds,
                list_selected
                    .then_some(vec![
                        Item::Button(
                            fl!("rename"),
                            Some(widget::icon::from_name("edit-symbolic").size(14).handle()),
                            MenuAction::RenameList,
                        ),
                        Item::Divider,
                        Item::Button(
                            fl!("icon"),
                            Some(
                                widget::icon::from_name("face-smile-big-symbolic")
                                    .size(14)
                                    .handle(),
                            ),
                            MenuAction::Icon,
                        ),
                        Item::Divider,
                        Item::Button(
                            fl!("delete"),
                            Some(
                                widget::icon::from_name("user-trash-full-symbolic")
                                    .size(14)
                                    .handle(),
                            ),
                            MenuAction::DeleteList,
                        ),
                    ])
                    .unwrap_or(vec![
                        Item::ButtonDisabled(
                            fl!("rename"),
                            Some(widget::icon::from_name("edit-symbolic").size(14).handle()),
                            MenuAction::RenameList,
                        ),
                        Item::Divider,
                        Item::ButtonDisabled(
                            fl!("icon"),
                            Some(
                                widget::icon::from_name("face-smile-big-symbolic")
                                    .size(14)
                                    .handle(),
                            ),
                            MenuAction::Icon,
                        ),
                        Item::Divider,
                        Item::ButtonDisabled(
                            fl!("delete"),
                            Some(
                                widget::icon::from_name("user-trash-full-symbolic")
                                    .size(14)
                                    .handle(),
                            ),
                            MenuAction::DeleteList,
                        ),
                    ]),
            ),
        ),
        Tree::with_children(
            Element::from(root(fl!("view"))),
            items(
                &state.key_binds,
                vec![
                    Item::Button(
                        fl!("menu-settings"),
                        Some(
                            widget::icon::from_name("preferences-system-symbolic")
                                .size(14)
                                .handle(),
                        ),
                        MenuAction::Settings,
                    ),
                    Item::Divider,
                    Item::Button(
                        fl!("menu-about"),
                        Some(
                            widget::icon::from_name("dialog-information-symbolic")
                                .size(14)
                                .handle(),
                        ),
                        MenuAction::About,
                    ),
                ],
            ),
        ),
        Tree::with_children(
            Element::from(root(fl!("sort"))),
            items(
                &state.key_binds,
                list_selected
                    .then_some(vec![
                        Item::Button(fl!("sort-name-asc"), None, MenuAction::SortByNameAsc),
                        Item::Button(fl!("sort-name-desc"), None, MenuAction::SortByNameDesc),
                        Item::Button(fl!("sort-date-asc"), None, MenuAction::SortByDateAsc),
                        Item::Button(fl!("sort-date-desc"), None, MenuAction::SortByDateDesc),
                    ])
                    .unwrap_or(vec![
                        Item::ButtonDisabled(fl!("sort-name-asc"), None, MenuAction::SortByNameAsc),
                        Item::ButtonDisabled(
                            fl!("sort-name-desc"),
                            None,
                            MenuAction::SortByNameDesc,
                        ),
                        Item::ButtonDisabled(fl!("sort-date-asc"), None, MenuAction::SortByDateAsc),
                        Item::ButtonDisabled(
                            fl!("sort-date-desc"),
                            None,
                            MenuAction::SortByDateDesc,
                        ),
                    ]),
            ),
        ),
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(260))
    .spacing(4.0)
    .into()
}
