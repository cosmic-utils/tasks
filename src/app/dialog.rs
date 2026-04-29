use cosmic::{
    iced::{
        Length,
        alignment::{Horizontal, Vertical},
    },
    widget::{self, calendar::CalendarModel, segmented_button},
};

use crate::{app::Message, app::actions::ApplicationAction, fl};

#[derive(Debug, Clone)]
pub enum DialogAction {
    Open(DialogPage),
    Update(DialogPage),
    Close,
    Complete,
    /// "Save to file" branch of the Export dialog; runs alongside the
    /// primary "Copy to clipboard" action.
    ExportSave,
    None,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogPage {
    New(String),
    Icon(Option<segmented_button::Entity>, String, String),
    Rename(Option<segmented_button::Entity>, String),
    Delete(Option<segmented_button::Entity>),
    Calendar(CalendarModel),
    /// (markdown_contents, save_path). `save_path` is the file path the
    /// user typed for "Save to file"; empty until they fill it in.
    Export(String, String),
    /// Path to a markdown file on disk plus a status line for feedback.
    Import {
        path: String,
        status: String,
    },
}

impl DialogPage {
    pub fn view(&self, text_input_id: &widget::Id) -> widget::Dialog<'_, Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;

        match self {
            DialogPage::New(name) => widget::dialog()
                .title(fl!("create-list"))
                .primary_action(widget::button::suggested(fl!("save")).on_press_maybe(Some(
                    Message::Application(ApplicationAction::Dialog(DialogAction::Complete)),
                )))
                .secondary_action(widget::button::standard(fl!("cancel")).on_press(
                    Message::Application(ApplicationAction::Dialog(DialogAction::Close)),
                ))
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("list-name")).into(),
                        widget::text_input("", name.as_str())
                            .id(text_input_id.clone())
                            .on_input(move |name| {
                                Message::Application(ApplicationAction::Dialog(
                                    DialogAction::Update(DialogPage::New(name)),
                                ))
                            })
                            .on_submit(|_| {
                                Message::Application(ApplicationAction::Dialog(
                                    DialogAction::Complete,
                                ))
                            })
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::Rename(entity, name) => widget::dialog()
                .title(fl!("rename-list"))
                .primary_action(widget::button::suggested(fl!("save")).on_press_maybe(Some(
                    Message::Application(ApplicationAction::Dialog(DialogAction::Complete)),
                )))
                .secondary_action(widget::button::standard(fl!("cancel")).on_press(
                    Message::Application(ApplicationAction::Dialog(DialogAction::Close)),
                ))
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("list-name")).into(),
                        widget::text_input("", name.as_str())
                            .id(text_input_id.clone())
                            .on_input(move |name| {
                                Message::Application(ApplicationAction::Dialog(
                                    DialogAction::Update(DialogPage::Rename(*entity, name)),
                                ))
                            })
                            .on_submit(|_| {
                                Message::Application(ApplicationAction::Dialog(
                                    DialogAction::Complete,
                                ))
                            })
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::Delete(_) => widget::dialog()
                .title(fl!("delete-list"))
                .body(fl!("delete-list-confirm"))
                .primary_action(widget::button::suggested(fl!("ok")).on_press_maybe(Some(
                    Message::Application(ApplicationAction::Dialog(DialogAction::Complete)),
                )))
                .secondary_action(widget::button::standard(fl!("cancel")).on_press(
                    Message::Application(ApplicationAction::Dialog(DialogAction::Close)),
                )),
            DialogPage::Icon(entity, icon, search) => {
                let search_lower = search.to_lowercase();
                let icon_buttons = crate::core::icons::get_all_icon_handles(20)
                    .iter()
                    .filter(|(name, _)| name.to_lowercase().contains(&search_lower))
                    .map(|(name, icon)| {
                        widget::button::icon(icon.clone())
                            .medium()
                            .on_press(Message::Application(ApplicationAction::Dialog(
                                DialogAction::Update(DialogPage::Icon(
                                    *entity,
                                    name.clone(),
                                    search.clone(),
                                )),
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
                            Message::Application(ApplicationAction::Dialog(DialogAction::Update(
                                DialogPage::Icon(entity, icon.clone(), s),
                            )))
                        }
                    });

                let dialog = widget::dialog()
                    .title(fl!("icon-select"))
                    .icon(crate::core::icons::get_icon(icon, 32))
                    .primary_action(widget::button::suggested(fl!("ok")).on_press_maybe(Some(
                        Message::Application(ApplicationAction::Dialog(DialogAction::Complete)),
                    )))
                    .secondary_action(widget::button::standard(fl!("cancel")).on_press(
                        Message::Application(ApplicationAction::Dialog(DialogAction::Close)),
                    ))
                    .control(
                        widget::column::with_children(vec![
                            search_input.into(),
                            widget::container(widget::scrollable(
                                widget::row()
                                    .push(widget::flex_row(icon_buttons))
                                    .push(widget::horizontal_space()),
                            ))
                            .height(Length::Fixed(300.0))
                            .into(),
                        ])
                        .spacing(spacing.space_xxs),
                    );

                dialog
            }
            DialogPage::Calendar(date) => {
                let dialog = widget::dialog()
                    .title(fl!("select-date"))
                    .primary_action(widget::button::suggested(fl!("ok")).on_press_maybe(Some(
                        Message::Application(ApplicationAction::Dialog(DialogAction::Complete)),
                    )))
                    .secondary_action(widget::button::standard(fl!("cancel")).on_press(
                        Message::Application(ApplicationAction::Dialog(DialogAction::Close)),
                    ))
                    .control(
                        widget::container(widget::calendar(
                            date,
                            |selected_date| {
                                Message::Application(ApplicationAction::Dialog(
                                    DialogAction::Update(DialogPage::Calendar(CalendarModel::new(
                                        selected_date,
                                        selected_date,
                                    ))),
                                ))
                            },
                            || Message::Application(ApplicationAction::Dialog(DialogAction::None)),
                            || Message::Application(ApplicationAction::Dialog(DialogAction::None)),
                            chrono::Weekday::Mon,
                        ))
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                    );
                dialog
            }
            DialogPage::Export(contents, save_path) => {
                let preview = widget::container(
                    widget::scrollable(widget::text(contents)).width(Length::Fill),
                )
                .height(Length::Fixed(200.0))
                .width(Length::Fill);

                let path_input = widget::text_input(fl!("export-save-path-hint"), save_path)
                    .on_input({
                        let contents = contents.clone();
                        move |s| {
                            Message::Application(ApplicationAction::Dialog(DialogAction::Update(
                                DialogPage::Export(contents.clone(), s),
                            )))
                        }
                    });

                let save_path_field = widget::column::with_children(vec![
                    widget::text::caption(fl!("export-save-path-label")).into(),
                    path_input.into(),
                ])
                .spacing(spacing.space_xxxs);

                widget::dialog()
                    .title(fl!("export"))
                    .control(
                        widget::column::with_children(vec![preview.into(), save_path_field.into()])
                            .spacing(spacing.space_s),
                    )
                    .primary_action(widget::button::suggested(fl!("copy")).on_press(
                        Message::Application(ApplicationAction::Dialog(DialogAction::Complete)),
                    ))
                    .secondary_action(widget::button::standard(fl!("cancel")).on_press(
                        Message::Application(ApplicationAction::Dialog(DialogAction::Close)),
                    ))
                    .tertiary_action(
                        widget::button::standard(fl!("export-save-to-file")).on_press_maybe(
                            (!save_path.trim().is_empty()).then_some(Message::Application(
                                ApplicationAction::Dialog(DialogAction::ExportSave),
                            )),
                        ),
                    )
            }
            DialogPage::Import { path, status } => {
                let path_input = widget::text_input(fl!("import-path-hint"), path)
                    .id(text_input_id.clone())
                    .on_input(move |p| {
                        Message::Application(ApplicationAction::Dialog(DialogAction::Update(
                            DialogPage::Import {
                                path: p,
                                status: String::new(),
                            },
                        )))
                    })
                    .on_submit(|_| {
                        Message::Application(ApplicationAction::Dialog(DialogAction::Complete))
                    });

                let mut children: Vec<cosmic::Element<'_, Message>> = vec![
                    widget::text::caption(fl!("import-description")).into(),
                    widget::text::body(fl!("import-path-label")).into(),
                    path_input.into(),
                ];
                if !status.is_empty() {
                    children.push(widget::text::caption(status.clone()).into());
                }

                widget::dialog()
                    .title(fl!("import"))
                    .primary_action(
                        widget::button::suggested(fl!("import-action")).on_press_maybe(
                            (!path.trim().is_empty()).then_some(Message::Application(
                                ApplicationAction::Dialog(DialogAction::Complete),
                            )),
                        ),
                    )
                    .secondary_action(widget::button::standard(fl!("cancel")).on_press(
                        Message::Application(ApplicationAction::Dialog(DialogAction::Close)),
                    ))
                    .control(
                        widget::column::with_children(children).spacing(spacing.space_xxs),
                    )
            }
        }
    }
}
