use cosmic::{
    Element, Task,
    iced::{Alignment, Length},
    theme,
    widget::{
        self,
        segmented_button::{self, Entity},
        text_editor,
    },
};
use uuid::Uuid;

use crate::{
    app::context::ContextPage,
    fl,
    model::{self, Priority},
    services::store::Store,
};

pub struct Details {
    pub task: model::Task,
    pub selected_list: Option<Uuid>,
    pub priority_model: segmented_button::Model<segmented_button::SingleSelect>,
    pub text_editor_content: widget::text_editor::Content,
    pub store: Store,
}

#[derive(Debug, Clone)]
pub enum Message {
    Open(Option<Uuid>, Uuid),
    SetTitle(String),
    Editor(text_editor::Action),
    Favorite(bool),
    PriorityActivate(Entity),
    // OpenCalendarDialog,
    // SetDueDate(NaiveDate),
}

pub enum Output {
    // OpenCalendarDialog,
}

impl Details {
    pub fn new(store: &Store) -> Self {
        let priority_model = segmented_button::ModelBuilder::default()
            .insert(|entity| {
                entity
                    .icon(widget::icon::from_name("flag-outline-thin-symbolic").size(14))
                    .data(Priority::Low)
            })
            .insert(|entity| {
                entity
                    .icon(widget::icon::from_name("flag-outline-thick-symbolic").size(14))
                    .data(Priority::Normal)
            })
            .insert(|entity| {
                entity
                    .icon(widget::icon::from_name("flag-filled-symbolic").size(14))
                    .data(Priority::High)
            })
            .build();

        Self {
            task: model::Task::default(),
            selected_list: None,
            priority_model,
            text_editor_content: widget::text_editor::Content::new(),
            store: store.clone(),
        }
    }

    pub fn update(&mut self, message: Message) -> Option<crate::app::Message> {
        match message {
            Message::Open(list_id, task) => {
                self.selected_list = list_id;
                if let Some(list_id) = self.selected_list {
                    if let Ok(task) = self.store.tasks(list_id).get(task) {
                        self.task.clone_from(&task);
                        self.text_editor_content =
                            widget::text_editor::Content::with_text(&task.notes);
                        let entity = self.priority_model.entity_at(task.priority as u16);
                        if let Some(entity) = entity {
                            self.priority_model.activate(entity);
                        } else {
                            self.priority_model.deactivate();
                        }
                    } else {
                        tracing::error!("Task with id {} not found in list {}", task, list_id);
                    }
                }

                return Some(crate::app::Message::ToggleContextPage(
                    ContextPage::TaskDetails,
                ));
            }
            Message::Editor(action) => {
                self.text_editor_content.perform(action);
                self.task.notes.clone_from(&self.text_editor_content.text());
            }
            Message::SetTitle(title) => {
                self.task.title.clone_from(&title);
            }
            Message::Favorite(favorite) => {
                self.task.favorite = favorite;
            }
            Message::PriorityActivate(entity) => {
                self.priority_model.activate(entity);
                let priority = self.priority_model.data::<Priority>(entity);
                if let Some(priority) = priority {
                    self.task.priority = *priority;
                }
            } // Message::OpenCalendarDialog => {
              //     return Some(Output::OpenCalendarDialog);
              // }
              // Message::SetDueDate(date) => {
              //     let tz = Utc::now().timezone();
              //     self.task.due_date = Some(tz.from_utc_datetime(&date.into()));
              // }
        }

        if let Err(e) = self
            .store
            .tasks(self.selected_list?)
            .update(self.task.id, |task| {
                task.clone_from(&self.task);
            })
        {
            tracing::error!("Failed to update task: {}", e);
        }

        return None;
    }

    pub fn view(&self) -> Element<'_, Message> {
        let spacing = theme::active().cosmic().spacing;

        widget::settings::view_column(vec![
            widget::settings::section()
                .title(fl!("details"))
                .add(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("title")).into(),
                        widget::text_input(fl!("title"), &self.task.title)
                            // .style(crate::core::style::text_input())
                            .on_input(Message::SetTitle)
                            .size(13)
                            .into(),
                    ])
                    .padding([
                        spacing.space_s,
                        spacing.space_none,
                        spacing.space_s,
                        spacing.space_none,
                    ])
                    .spacing(spacing.space_xxs),
                )
                .add(
                    widget::settings::item::builder(fl!("favorite"))
                        .control(widget::checkbox(self.task.favorite).on_toggle(Message::Favorite)),
                )
                .add(
                    widget::settings::item::builder(fl!("priority")).control(
                        widget::segmented_control::horizontal(&self.priority_model)
                            .button_alignment(Alignment::Center)
                            .width(Length::Shrink)
                            // .style(crate::core::style::segmented_control())
                            .on_activate(Message::PriorityActivate),
                    ),
                )
                // .add(
                //     widget::settings::item::builder(fl!("due-date")).control(
                //         widget::button::text(if self.task.due_date.is_some() {
                //             self.task
                //                 .due_date
                //                 .as_ref()
                //                 .unwrap()
                //                 .format("%m-%d-%Y")
                //                 .to_string()
                //         } else {
                //             fl!("select-date")
                //         })
                //         .on_press(Message::OpenCalendarDialog),
                //     ),
                // )
                .add(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("notes")).into(),
                        widget::text_editor(&self.text_editor_content)
                            // .class(crate::core::style::text_editor())
                            .padding(spacing.space_xxs)
                            .placeholder(fl!("add-notes"))
                            .height(100.0)
                            .size(13)
                            .on_action(Message::Editor)
                            .into(),
                    ])
                    .spacing(spacing.space_xxs)
                    .padding([
                        spacing.space_s,
                        spacing.space_none,
                        spacing.space_s,
                        spacing.space_none,
                    ]),
                )
                .into(),
        ])
        .padding([
            spacing.space_none,
            spacing.space_s,
            spacing.space_none,
            spacing.space_s,
        ])
        .into()
    }
}
