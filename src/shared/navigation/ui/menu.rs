// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    widget::{
        menu::{items, root, Item, ItemHeight, ItemWidth, MenuBar, Tree},
    },
    Element,
};

use crate::{
    app::{AppModel, Message},
    config::SortBy,
    features::lists::List,
    fl,
    shared::navigation::ui::MenuAction,
};

pub fn menu_bar<'a>(state: &AppModel) -> Element<'a, Message> {
    let list_selected = state.nav.active_data::<List>().is_some();

    MenuBar::new(vec![
        Tree::with_children(
            Element::from(root(fl!("file"))),
            items(
                &state.key_binds,
                list_selected
                    .then_some(vec![
                        Item::Button(
                            fl!("new-window"),
                            None,
                            MenuAction::WindowNew,
                        ),
                        Item::Button(
                            fl!("new-list"),
                            None,
                            MenuAction::NewList,
                        ),
                        Item::Divider,
                        Item::Button(
                            fl!("rename"),
                            None,
                            MenuAction::RenameList,
                        ),
                        Item::Button(
                            fl!("icon"),
                            None,
                            MenuAction::Icon,
                        ),
                        Item::Button(
                            fl!("move-to-trash"),
                            None,
                            MenuAction::DeleteList,
                        ),
                        Item::Divider,
                        Item::Button(
                            fl!("quit"),
                            None,
                            MenuAction::WindowClose,
                        ),
                    ])
                    .unwrap_or(vec![
                        Item::Button(
                            fl!("new-window"),
                            None,
                            MenuAction::WindowNew,
                        ),
                        Item::Button(
                            fl!("new-list"),
                            None,
                            MenuAction::NewList,
                        ),
                        Item::Divider,
                        Item::ButtonDisabled(
                            fl!("rename"),
                            None,
                            MenuAction::RenameList,
                        ),
                        Item::ButtonDisabled(
                            fl!("icon"),
                            None,
                            MenuAction::Icon,
                        ),
                        Item::ButtonDisabled(
                            fl!("move-to-trash"),
                            None,
                            MenuAction::DeleteList,
                        ),
                        Item::Divider,
                        Item::Button(
                            fl!("quit"),
                            None,
                            MenuAction::WindowClose,
                        ),
                    ]),
            ),
        ),
        Tree::with_children(
            Element::from(root(fl!("view"))),
            items(
                &state.key_binds,
                vec![
                    Item::CheckBox(
                        fl!("hide-completed"),
                        None,
                        state.config.hide_completed,
                        MenuAction::ToggleHideCompleted(!state.config.hide_completed),
                    ),
                    Item::Divider,
                    Item::Button(
                        fl!("menu-settings"),
                        None,
                        MenuAction::Settings,
                    ),
                    Item::Divider,
                    Item::Button(
                        fl!("menu-about"),
                        None,
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
                        Item::CheckBox(
                            fl!("sort-name-asc"),
                            None,
                            state.config.sort_by == SortBy::NameAsc,
                            MenuAction::SortByNameAsc,
                        ),
                        Item::CheckBox(
                            fl!("sort-name-desc"),
                            None,
                            state.config.sort_by == SortBy::NameDesc,
                            MenuAction::SortByNameDesc,
                        ),
                        Item::CheckBox(
                            fl!("sort-date-asc"),
                            None,
                            state.config.sort_by == SortBy::DateAsc,
                            MenuAction::SortByDateAsc,
                        ),
                        Item::CheckBox(
                            fl!("sort-date-desc"),
                            None,
                            state.config.sort_by == SortBy::DateDesc,
                            MenuAction::SortByDateDesc,
                        ),
                        Item::CheckBox(
                            fl!("sort-manual"),
                            None,
                            state.config.sort_by == SortBy::Manual,
                            MenuAction::SortByManual,
                        ),
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
                        Item::ButtonDisabled(fl!("sort-manual"), None, MenuAction::SortByManual),
                    ]),
            ),
        ),
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(360))
    .spacing(4.0)
    .into()
}
