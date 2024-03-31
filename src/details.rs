use std::ops::IndexMut;

use crate::app::config;
use crate::app::config::get_icon;
use chrono::{NaiveDate, TimeZone, Utc};
use cosmic::iced::{Alignment, Length};
use cosmic::iced_widget::row;
use cosmic::widget::segmented_button;
use cosmic::widget::segmented_button::Entity;
use cosmic::{cosmic_theme, theme, widget, Element};
use done_core::models::priority::Priority;
use done_core::models::status::Status;
use done_core::models::task::Task;

use crate::fl;

pub struct Details {
    pub task: Option<Task>,
    pub is_editable: bool,
    pub priority_model: segmented_button::Model<segmented_button::SingleSelect>,
    pub subtask_input: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetTitle(String),
    SetNotes(String),
    CompleteSubTask(usize, bool),
    DeleteSubTask(usize),
    Favorite(bool),
    PriorityActivate(Entity),
    SubTaskInput(String),
    SetSubTaskTitle(usize, String),
    AddTask,
    OpenCalendarDialog,
    SetDueDate(NaiveDate),
    SubTaskEditDone,
}

pub enum Command {
    Focus(widget::Id),
    UpdateTask(Task),
    OpenCalendarDialog,
}

impl Details {
    pub fn new() -> Self {
        let priority_model = segmented_button::ModelBuilder::default()
            .insert(|entity| {
                entity
                    .icon(config::get_icon("flag-outline-thin-symbolic", 16))
                    .data(Priority::Low)
            })
            .insert(|entity| {
                entity
                    .icon(config::get_icon("flag-outline-thick-symbolic", 16))
                    .data(Priority::Normal)
            })
            .insert(|entity| {
                entity
                    .icon(config::get_icon("flag-filled-symbolic", 16))
                    .data(Priority::High)
            })
            .build();

        Self {
            task: None,
            priority_model,
            is_editable: false,
            subtask_input: String::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Vec<Command> {
        let mut commands = vec![];
        match message {
            Message::SetTitle(title) => {
                if let Some(ref mut task) = &mut self.task {
                    task.title = title.clone();
                }
            }
            Message::SetNotes(notes) => {
                if let Some(ref mut task) = &mut self.task {
                    task.notes = notes.clone();
                }
            }
            Message::Favorite(favorite) => {
                if let Some(ref mut task) = &mut self.task {
                    task.favorite = favorite;
                }
            }
            Message::PriorityActivate(entity) => {
                self.priority_model.activate(entity);
                let priority = self.priority_model.data::<Priority>(entity);
                if let Some(task) = &mut self.task {
                    if let Some(priority) = priority {
                        task.priority = *priority;
                    }
                }
            }
            Message::SetSubTaskTitle(i, title) => {
                if let Some(ref mut task) = &mut self.task {
                    task.sub_tasks.index_mut(i).title = title.clone();
                }
            }
            Message::CompleteSubTask(i, completed) => {
                if let Some(ref mut task) = &mut self.task {
                    task.sub_tasks.index_mut(i).status = if completed {
                        Status::Completed
                    } else {
                        Status::NotStarted
                    };
                }
            }
            Message::SubTaskEditDone => {
                commands.push(Command::Focus(widget::Id::new("new_sub_task_input")));
            }
            Message::DeleteSubTask(i) => {
                if let Some(ref mut task) = &mut self.task {
                    task.sub_tasks.index_mut(i).deletion_date = Some(Utc::now());
                    task.sub_tasks.remove(i);
                }
            }
            Message::SubTaskInput(text) => {
                self.subtask_input = text;
            }
            Message::AddTask => {
                if let Some(ref mut task) = &mut self.task {
                    if !self.subtask_input.is_empty() {
                        task.sub_tasks
                            .push(Task::new(self.subtask_input.clone(), task.id.clone()));
                        self.subtask_input.clear();
                        commands.push(Command::Focus(widget::Id::new("new_sub_task_input")));
                    }
                }
            }
            Message::OpenCalendarDialog => {
                commands.push(Command::OpenCalendarDialog);
            }
            Message::SetDueDate(date) => {
                let tz = Utc::now().timezone();
                if let Some(task) = &mut self.task {
                    task.due_date = Some(tz.from_utc_datetime(&date.into()));
                }
            }
        }

        if let Some(task) = &self.task {
            commands.push(Command::UpdateTask(task.clone()));
        }

        commands
    }

    pub fn view(&self) -> Element<Message> {
        let cosmic_theme::Spacing {
            space_xxs,
            space_xs,
            ..
        } = theme::active().cosmic().spacing;

        if let Some(task) = self.task.as_ref() {
            let mut sub_tasks: Vec<Element<Message>> = task
                .sub_tasks
                .iter()
                .enumerate()
                .map(|(i, sub_task)| {
                    widget::row::with_children(vec![
                        widget::checkbox("", sub_task.status == Status::Completed, move |value| {
                            Message::CompleteSubTask(i, value)
                        })
                        .into(),
                        widget::text_input(fl!("title"), sub_task.title.clone())
                            .id(widget::Id::new("sub_task_input"))
                            .on_input(move |title| Message::SetSubTaskTitle(i, title))
                            .on_submit(Message::SubTaskEditDone)
                            .into(),
                        widget::button(config::get_icon("user-trash-full-symbolic", 18))
                            .padding(space_xxs)
                            .style(widget::button::Style::Destructive)
                            .on_press(Message::DeleteSubTask(i))
                            .into(),
                    ])
                    .align_items(Alignment::Center)
                    .padding([0, 18])
                    .spacing(12)
                    .into()
                })
                .collect();

            sub_tasks.push(self.sub_task_input());

            return widget::settings::view_column(vec![
                widget::settings::view_section(fl!("details"))
                    .add(
                        widget::column::with_children(vec![
                            widget::text::body(fl!("title")).into(),
                            widget::text_input(fl!("title"), &task.title)
                                .on_input(Message::SetTitle)
                                .into(),
                        ])
                        .spacing(space_xxs)
                        .padding([0, 15, 0, 15]),
                    )
                    .add(
                        widget::settings::item::builder(fl!("favorite")).control(widget::checkbox(
                            "",
                            task.favorite,
                            Message::Favorite,
                        )),
                    )
                    .add(
                        widget::settings::item::builder(fl!("priority")).control(
                            widget::segmented_control::horizontal(&self.priority_model)
                                .button_alignment(Alignment::Center)
                                .width(Length::Shrink)
                                .on_activate(Message::PriorityActivate),
                        ),
                    )
                    .add(
                        widget::settings::item::builder(fl!("due-date")).control(
                            widget::button::text(if let Some(task) = &self.task {
                                if task.due_date.is_some() {
                                    task.due_date
                                        .as_ref()
                                        .unwrap()
                                        .format("%m-%d-%Y")
                                        .to_string()
                                } else {
                                    fl!("select-date")
                                }
                            } else {
                                fl!("select-date")
                            })
                            .on_press(Message::OpenCalendarDialog),
                        ),
                    )
                    .add(
                        widget::column::with_children(vec![
                            widget::text::body(fl!("notes")).into(),
                            widget::text_input(fl!("notes"), &task.notes)
                                .on_input(Message::SetNotes)
                                .into(),
                        ])
                        .spacing(space_xxs)
                        .padding([0, 15, 0, 15]),
                    )
                    .into(),
                widget::settings::view_section(fl!("sub-tasks"))
                    .add(widget::column::with_children(sub_tasks).spacing(space_xs))
                    .into(),
            ])
            .into();
        }
        widget::settings::view_column(vec![widget::settings::view_section(fl!("details")).into()])
            .into()
    }

    fn sub_task_input(&self) -> Element<Message> {
        let cosmic_theme::Spacing {
            space_xxs,
            space_xs,
            space_s,
            ..
        } = theme::active().cosmic().spacing;

        row(vec![
            widget::text_input(fl!("add-sub-task"), &self.subtask_input)
                .id(widget::Id::new("new_sub_task_input"))
                .on_input(Message::SubTaskInput)
                .on_submit(Message::AddTask)
                .width(Length::Fill)
                .into(),
            widget::button(config::get_icon("mail-send-symbolic", 18))
                .padding(space_xxs)
                .on_press(Message::AddTask)
                .into(),
        ])
        .padding([0, space_s])
        .spacing(space_xs)
        .align_items(Alignment::Center)
        .into()
    }
}
