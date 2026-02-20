use std::collections::HashMap;

use cosmic::{
    Element,
    widget::{self, menu::KeyBind},
};

use crate::{
    app::{Message, config::Config, context::ContextPage},
    fl,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    File(FileAction),
    Edit(EditAction),
    View(ViewAction),
    Sort(SortAction),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileAction {
    WindowNew,
    NewList,
    WindowClose,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EditAction {
    RenameList,
    Icon,
    DeleteList,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewAction {
    Settings,
    ToggleHideCompleted(bool),
    About,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SortAction {
    SortByNameAsc,
    SortByNameDesc,
    SortByDateAsc,
    SortByDateDesc,
}

impl cosmic::widget::menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        Message::Menu(*self)
    }
}

pub fn menu_bar<'a>(
    key_binds: &HashMap<KeyBind, MenuAction>,
    config: &Config,
) -> impl Into<Element<'a, Message>> {
    widget::menu::MenuBar::new(vec![
        widget::menu::Tree::with_children(
            Element::from(widget::menu::root(fl!("file"))),
            widget::menu::items(
                key_binds,
                vec![
                    widget::menu::Item::Button(
                        fl!("new-window"),
                        Some(
                            widget::icon::from_name("tabs-stack-symbolic")
                                .size(14)
                                .into(),
                        ),
                        MenuAction::File(FileAction::WindowNew),
                    ),
                    widget::menu::Item::Divider,
                    widget::menu::Item::Button(
                        fl!("new-list"),
                        Some(
                            widget::icon::from_name("plus-square-filled-symbolic")
                                .size(14)
                                .into(),
                        ),
                        MenuAction::File(FileAction::NewList),
                    ),
                    widget::menu::Item::Divider,
                    widget::menu::Item::Button(
                        fl!("quit"),
                        Some(
                            widget::icon::from_name("cross-small-square-filled-symbolic")
                                .size(14)
                                .into(),
                        ),
                        MenuAction::File(FileAction::WindowClose),
                    ),
                ],
            ),
        ),
        widget::menu::Tree::with_children(
            Element::from(widget::menu::root(fl!("edit"))),
            widget::menu::items(
                key_binds,
                vec![
                    widget::menu::Item::Button(
                        fl!("rename"),
                        Some(widget::icon::from_name("edit-symbolic").size(14).into()),
                        MenuAction::Edit(EditAction::RenameList),
                    ),
                    widget::menu::Item::Divider,
                    widget::menu::Item::Button(
                        fl!("icon"),
                        Some(
                            widget::icon::from_name("face-smile-big-symbolic")
                                .size(14)
                                .into(),
                        ),
                        MenuAction::Edit(EditAction::Icon),
                    ),
                    widget::menu::Item::Divider,
                    widget::menu::Item::Button(
                        fl!("delete"),
                        Some(
                            widget::icon::from_name("user-trash-full-symbolic")
                                .size(14)
                                .into(),
                        ),
                        MenuAction::Edit(EditAction::DeleteList),
                    ),
                ],
            ),
        ),
        widget::menu::Tree::with_children(
            Element::from(widget::menu::root(fl!("view"))),
            widget::menu::items(
                key_binds,
                vec![
                    widget::menu::Item::Button(
                        fl!("menu-settings"),
                        Some(widget::icon::from_name("settings-symbolic").size(14).into()),
                        MenuAction::View(ViewAction::Settings),
                    ),
                    widget::menu::Item::Divider,
                    widget::menu::Item::CheckBox(
                        fl!("hide-completed"),
                        None,
                        config.hide_completed,
                        MenuAction::View(ViewAction::ToggleHideCompleted(!config.hide_completed)),
                    ),
                    widget::menu::Item::Divider,
                    widget::menu::Item::Button(
                        fl!("menu-about"),
                        Some(
                            widget::icon::from_name("info-outline-symbolic")
                                .size(14)
                                .into(),
                        ),
                        MenuAction::View(ViewAction::About),
                    ),
                ],
            ),
        ),
        widget::menu::Tree::with_children(
            Element::from(widget::menu::root(fl!("sort"))),
            widget::menu::items(
                key_binds,
                vec![
                    widget::menu::Item::Button(
                        fl!("sort-name-asc"),
                        None,
                        MenuAction::Sort(SortAction::SortByNameAsc),
                    ),
                    widget::menu::Item::Button(
                        fl!("sort-name-desc"),
                        None,
                        MenuAction::Sort(SortAction::SortByNameDesc),
                    ),
                    widget::menu::Item::Button(
                        fl!("sort-date-asc"),
                        None,
                        MenuAction::Sort(SortAction::SortByDateAsc),
                    ),
                    widget::menu::Item::Button(
                        fl!("sort-date-desc"),
                        None,
                        MenuAction::Sort(SortAction::SortByDateDesc),
                    ),
                ],
            ),
        ),
    ])
    .item_height(widget::menu::ItemHeight::Dynamic(40))
    .item_width(widget::menu::ItemWidth::Uniform(260))
    .spacing(4.0)
}
