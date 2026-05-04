use std::path::PathBuf;

use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Length,
    },
    widget::{self, calendar::CalendarModel, segmented_button},
};
use slotmap::DefaultKey;
use uuid::Uuid;

use crate::{app::Message, fl};

#[derive(Debug, Clone)]
pub enum DialogAction {
    Open(DialogPage),
    Update(DialogPage),
    Close,
    Complete,
    None,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogPage {
    NewList(String),
    SetListIcon(Option<segmented_button::Entity>, String, String),
    RenameList(Option<segmented_button::Entity>, String),
    DeleteList(Option<segmented_button::Entity>),
    DeleteTask(DefaultKey, Uuid, Uuid),
    Calendar(CalendarModel),
    Export(String),
}

pub fn get_all_icon_handles(size: u16) -> Vec<(String, widget::icon::Handle)> {
    let mut icons = Vec::new();

    // Scan system icon directories
    let icon_dirs = vec![
        PathBuf::from("/usr/share/icons/hicolor"),
        PathBuf::from("/usr/share/icons/Adwaita"),
    ];

    for dir in icon_dirs {
        if let Ok(entries) = scan_icon_directory(&dir) {
            for name in entries {
                let handle = widget::icon::from_name(&*name).size(size).handle();
                icons.push((name, handle));
            }
        }
    }

    icons
}

fn scan_icon_directory(path: &PathBuf) -> std::io::Result<Vec<String>> {
    let mut icons = Vec::new();

    for entry in std::fs::read_dir(path.join("scalable/actions"))? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            if name.ends_with(".svg") {
                icons.push(name.trim_end_matches(".svg").to_string());
            }
        }
    }

    Ok(icons)
}

impl DialogPage {
    pub fn view(&self, text_input_id: &widget::Id) -> widget::Dialog<'_, Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;

        match self {
            DialogPage::NewList(name) => widget::dialog()
                .title(fl!("create-list"))
                .primary_action(
                    widget::button::suggested(fl!("save"))
                        .on_press_maybe(Some(Message::Dialog(DialogAction::Complete))),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(Message::Dialog(DialogAction::Close)),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("list-name")).into(),
                        widget::text_input("", name.as_str())
                            .id(text_input_id.clone())
                            .on_input(move |name| {
                                Message::Dialog(DialogAction::Update(DialogPage::NewList(name)))
                            })
                            .on_submit(|_| Message::Dialog(DialogAction::Complete))
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::RenameList(entity, name) => widget::dialog()
                .title(fl!("rename-list"))
                .primary_action(
                    widget::button::suggested(fl!("save"))
                        .on_press_maybe(Some(Message::Dialog(DialogAction::Complete))),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(Message::Dialog(DialogAction::Close)),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("list-name")).into(),
                        widget::text_input("", name.as_str())
                            .id(text_input_id.clone())
                            .on_input(move |name| {
                                Message::Dialog(DialogAction::Update(DialogPage::RenameList(
                                    *entity, name,
                                )))
                            })
                            .on_submit(|_| Message::Dialog(DialogAction::Complete))
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::DeleteList(_) => widget::dialog()
                .title(fl!("delete-list"))
                .body(fl!("delete-list-confirm"))
                .primary_action(
                    widget::button::suggested(fl!("ok"))
                        .on_press_maybe(Some(Message::Dialog(DialogAction::Complete))),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(Message::Dialog(DialogAction::Close)),
                ),
            DialogPage::SetListIcon(entity, icon, search) => {
                let search_lower = search.to_lowercase();
                let icon_buttons = get_all_icon_handles(20)
                    .iter()
                    .filter(|(name, _)| name.to_lowercase().contains(&search_lower))
                    .map(|(name, icon)| {
                        widget::button::icon(icon.clone())
                            .medium()
                            .on_press(Message::Dialog(DialogAction::Update(
                                DialogPage::SetListIcon(*entity, name.clone(), search.clone()),
                            )))
                            .into()
                    })
                    .collect();

                let search_input = widget::text_input(fl!("search-icons"), search.as_str())
                    .id(text_input_id.clone())
                    .on_input({
                        let entity = *entity;
                        let icon = icon.clone();
                        move |s| {
                            Message::Dialog(DialogAction::Update(DialogPage::SetListIcon(
                                entity,
                                icon.clone(),
                                s.to_string(),
                            )))
                        }
                    });

                let dialog = widget::dialog()
                    .title(fl!("icon-select"))
                    .icon(widget::icon::from_name(icon.clone()).size(32))
                    .primary_action(
                        widget::button::suggested(fl!("ok"))
                            .on_press_maybe(Some(Message::Dialog(DialogAction::Complete))),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel"))
                            .on_press(Message::Dialog(DialogAction::Close)),
                    )
                    .control(
                        widget::column::with_children(vec![
                            search_input.into(),
                            widget::container(widget::scrollable(
                                widget::row(vec![])
                                    .push(widget::flex_row(icon_buttons))
                                    .push(widget::space::horizontal()),
                            ))
                            .height(Length::Fixed(300.0))
                            .into(),
                        ])
                        .spacing(spacing.space_xxs),
                    );

                dialog
            }
            DialogPage::DeleteTask(_, _, _) => widget::dialog()
                .title(fl!("delete-task"))
                .body(fl!("delete-task-confirm"))
                .primary_action(
                    widget::button::destructive(fl!("delete"))
                        .on_press_maybe(Some(Message::Dialog(DialogAction::Complete))),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(Message::Dialog(DialogAction::Close)),
                ),
            DialogPage::Calendar(date) => {
                let dialog = widget::dialog()
                    .title(fl!("select-date"))
                    .primary_action(
                        widget::button::suggested(fl!("ok"))
                            .on_press_maybe(Some(Message::Dialog(DialogAction::Complete))),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel"))
                            .on_press(Message::Dialog(DialogAction::Close)),
                    )
                    .control(
                        widget::container(widget::calendar(
                            date,
                            |selected_date| {
                                Message::Dialog(DialogAction::Update(DialogPage::Calendar(
                                    CalendarModel::new(selected_date, selected_date),
                                )))
                            },
                            || Message::Dialog(DialogAction::None),
                            || Message::Dialog(DialogAction::None),
                            jiff::civil::Weekday::Monday,
                        ))
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                    );
                dialog
            }
            DialogPage::Export(contents) => {
                let dialog = widget::dialog()
                    .title(fl!("export"))
                    .control(
                        widget::container(
                            widget::scrollable(widget::text(contents)).width(Length::Fill),
                        )
                        .height(Length::Fixed(200.0))
                        .width(Length::Fill),
                    )
                    .primary_action(
                        widget::button::suggested(fl!("copy"))
                            .on_press_maybe(Some(Message::Dialog(DialogAction::Complete))),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel"))
                            .on_press(Message::Dialog(DialogAction::Close)),
                    );

                dialog
            }
        }
    }
}
