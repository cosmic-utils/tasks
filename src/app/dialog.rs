use cosmic::{
    iced::Length,
    widget::{self, segmented_button},
};
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
    New(String),
    Rename(Uuid, String),
    Delete(Uuid),
    Export(String),
}

impl DialogPage {
    pub fn view(&self, text_input_id: &widget::Id) -> widget::Dialog<'_, Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;

        match self {
            DialogPage::New(name) => widget::dialog()
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
                                Message::Dialog(DialogAction::Update(DialogPage::New(name)))
                            })
                            .on_submit(|_| Message::Dialog(DialogAction::Complete))
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::Rename(entity, name) => widget::dialog()
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
                                Message::Dialog(DialogAction::Update(DialogPage::Rename(
                                    *entity, name,
                                )))
                            })
                            .on_submit(|_| Message::Dialog(DialogAction::Complete))
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::Delete(_) => widget::dialog()
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
