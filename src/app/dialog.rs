use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Length,
    },
    widget::{self, calendar::CalendarModel, segmented_button},
};

use crate::{app::actions::ApplicationAction, app::Message, fl};

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
    New(String),
    Icon(Option<segmented_button::Entity>, String, String),
    Rename(Option<segmented_button::Entity>, String),
    Delete(Option<segmented_button::Entity>),
    Calendar(CalendarModel),
    Export(String),
}

impl DialogPage {
    pub fn view(&self, text_input_id: &widget::Id) -> widget::Dialog<Message> {
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
                let date_clone = date.clone();
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
                            move || {
                                // Previous month - update visible month, keep selected date
                                let mut new_model = date_clone.clone();
                                new_model.show_prev_month();
                                Message::Application(ApplicationAction::Dialog(
                                    DialogAction::Update(DialogPage::Calendar(new_model))
                                ))
                            },
                            move || {
                                // Next month - update visible month, keep selected date
                                let mut new_model = date_clone.clone();
                                new_model.show_next_month();
                                Message::Application(ApplicationAction::Dialog(
                                    DialogAction::Update(DialogPage::Calendar(new_model))
                                ))
                            },
                            chrono::Weekday::Mon,
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
                    .primary_action(widget::button::suggested(fl!("copy")).on_press_maybe(Some(
                        Message::Application(ApplicationAction::Dialog(DialogAction::Complete)),
                    )))
                    .secondary_action(widget::button::standard(fl!("cancel")).on_press(
                        Message::Application(ApplicationAction::Dialog(DialogAction::Close)),
                    ));

                dialog
            }
        }
    }
}
