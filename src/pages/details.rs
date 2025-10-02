use chrono::{NaiveDate, TimeZone, Utc};
use cosmic::{
    iced::{Alignment, Length},
    theme,
    widget::{
        self,
        segmented_button::{self, Entity},
        text_editor,
    },
    Element,
};

use crate::{
    core::icons,
    fl,
    storage::{
        models::{self, Priority},
        LocalStorage,
    },
};

pub struct Details {
    pub task: models::Task,
    pub priority_model: segmented_button::Model<segmented_button::SingleSelect>,
    pub text_editor_content: widget::text_editor::Content,
    pub storage: LocalStorage,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetTitle(String),
    Editor(text_editor::Action),
    Favorite(bool),
    PriorityActivate(Entity),
    OpenCalendarDialog,
    SetDueDate(NaiveDate),
}

pub enum Output {
    OpenCalendarDialog,
    RefreshTask(models::Task),
}

impl Details {
    pub fn new(storage: LocalStorage) -> Self {
        let priority_model = segmented_button::ModelBuilder::default()
            .insert(|entity| {
                entity
                    .icon(icons::get_icon("flag-outline-thin-symbolic", 14))
                    .data(Priority::Low)
            })
            .insert(|entity| {
                entity
                    .icon(icons::get_icon("flag-outline-thick-symbolic", 14))
                    .data(Priority::Normal)
            })
            .insert(|entity| {
                entity
                    .icon(icons::get_icon("flag-filled-symbolic", 14))
                    .data(Priority::High)
            })
            .build();

        Self {
            task: models::Task::default(),
            priority_model,
            text_editor_content: widget::text_editor::Content::new(),
            storage,
        }
    }

    pub fn update(&mut self, message: Message) -> Vec<Output> {
        let mut tasks = vec![];
        match message {
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
            }
            Message::OpenCalendarDialog => {
                tasks.push(Output::OpenCalendarDialog);
            }
            Message::SetDueDate(date) => {
                let tz = Utc::now().timezone();
                self.task.due_date = Some(tz.from_utc_datetime(&date.into()));
            }
        }

        if let Err(e) = self.storage.update_task(&self.task) {
            tracing::error!("Failed to update task: {}", e);
        }
        tasks.push(Output::RefreshTask(self.task.clone()));
        tasks
    }

    pub fn view(&self) -> Element<'_, Message> {
        let spacing = theme::active().cosmic().spacing;

        widget::settings::view_column(vec![widget::settings::section()
            .title(fl!("details"))
            .add(
                widget::column::with_children(vec![
                    widget::text::body(fl!("title")).into(),
                    widget::text_input(fl!("title"), &self.task.title)
                        .style(crate::core::style::text_input())
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
                    .control(widget::checkbox("", self.task.favorite).on_toggle(Message::Favorite)),
            )
            .add(
                widget::settings::item::builder(fl!("priority")).control(
                    widget::segmented_control::horizontal(&self.priority_model)
                        .button_alignment(Alignment::Center)
                        .width(Length::Shrink)
                        .style(crate::core::style::segmented_control())
                        .on_activate(Message::PriorityActivate),
                ),
            )
            .add(
                widget::settings::item::builder(fl!("due-date")).control(
                    widget::button::text(if self.task.due_date.is_some() {
                        self.task
                            .due_date
                            .as_ref()
                            .unwrap()
                            .format("%m-%d-%Y")
                            .to_string()
                    } else {
                        fl!("select-date")
                    })
                    .on_press(Message::OpenCalendarDialog),
                ),
            )
            .add(
                widget::column::with_children(vec![
                    widget::text::body(fl!("notes")).into(),
                    widget::text_editor(&self.text_editor_content)
                        .class(crate::core::style::text_editor())
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
            .into()])
        .padding([
            spacing.space_none,
            spacing.space_s,
            spacing.space_none,
            spacing.space_s,
        ])
        .into()
    }
}
