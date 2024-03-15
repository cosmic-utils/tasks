use std::ops::IndexMut;

use chrono::NaiveDate;
use cosmic::{cosmic_theme, Element, theme, widget};
use cosmic::iced::{Alignment, Length};
use cosmic::iced_widget::row;
use cosmic::widget::segmented_button;
use cosmic::widget::segmented_button::Entity;
use done_core::models::priority::Priority;
use done_core::models::status::Status;
use done_core::models::task::Task;

use crate::fl;

pub struct Details {
    pub task: Option<Task>,
    due_date: Option<NaiveDate>,
    pub priority_model: segmented_button::Model<segmented_button::SingleSelect>,
    pub subtask_input: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetTitle(String),
    SetNotes(String),
    CompleteSubTask(usize, bool),
    Favorite(bool),
    PriorityActivate(Entity),
    SubTaskInput(String),
    AddTask,
    OpenCalendarDialog,
    SetDueDate(NaiveDate),
}

pub enum Command {
    Update(Task),
    SetTitle(String, String),
    SetNotes(String, String),
    Favorite(String, bool),
    PriorityActivate(String, Priority),
    OpenCalendarDialog,
}

impl Details {
    pub fn new() -> Self {
        let priority_model = segmented_button::ModelBuilder::default()
            .insert(|entity| {
                entity
                    .icon(widget::icon(
                        widget::icon::from_name("security-medium-symbolic").handle(),
                    ))
                    .data(Priority::Low)
            })
            .insert(|entity| {
                entity
                    .icon(widget::icon(
                        widget::icon::from_name("security-high-symbolic").handle(),
                    ))
                    .data(Priority::Normal)
            })
            .insert(|entity| {
                entity
                    .icon(widget::icon(
                        widget::icon::from_name("security-low-symbolic").handle(),
                    ))
                    .data(Priority::High)
            })
            .build();

        Self {
            task: None,
            due_date: None,
            priority_model,
            subtask_input: String::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Vec<Command> {
        let mut commands = vec![];
        match message {
            Message::SetTitle(title) => {
                if let Some(ref mut task) = &mut self.task {
                    task.title = title.clone();
                    commands.push(Command::SetTitle(task.id.clone(), title));
                }
            }
            Message::SetNotes(notes) => {
                if let Some(ref mut task) = &mut self.task {
                    task.notes = notes.clone();
                    commands.push(Command::SetNotes(task.id.clone(), notes));
                }
            }
            Message::Favorite(favorite) => {
                if let Some(ref mut task) = &mut self.task {
                    task.favorite = favorite;
                    commands.push(Command::Favorite(task.id.clone(), favorite));
                }
            }
            Message::PriorityActivate(entity) => {
                self.priority_model.activate(entity);
                let priority = self.priority_model.data::<Priority>(entity);
                if let Some(task) = &self.task {
                    if let Some(priority) = priority {
                        commands.push(Command::PriorityActivate(task.id.clone(), *priority));
                    }
                }
            }
            Message::CompleteSubTask(i, completed) => {
                if let Some(ref mut task) = &mut self.task {
                    task.sub_tasks.index_mut(i).status = if completed {
                        Status::Completed
                    } else {
                        Status::NotStarted
                    };
                    commands.push(Command::Update(task.clone()));
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
                        commands.push(Command::Update(task.clone()));
                        self.subtask_input.clear();
                    }
                }
            }
            Message::OpenCalendarDialog => {
                commands.push(Command::OpenCalendarDialog);
            }
            Message::SetDueDate(date) => {
                self.due_date = Some(date);
            }
        }
        commands
    }

    pub fn view(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, space_xs, .. } = theme::active().cosmic().spacing;

        if let Some(task) = self.task.as_ref() {
            let mut sub_tasks: Vec<Element<Message>> = task
                .sub_tasks
                .iter()
                .enumerate()
                .map(|(i, sub_task)| {
                    widget::settings::item::builder(sub_task.title.clone())
                        .control(widget::checkbox(
                            "",
                            sub_task.status == Status::Completed,
                            move |value| Message::CompleteSubTask(i, value),
                        ))
                        .into()
                })
                .collect();

            sub_tasks.push(self.sub_task_input());

            return widget::settings::view_column(vec![
                widget::settings::view_section(fl!("details"))
                    .add(
                        widget::column::with_children(
                            vec![
                                widget::text::body(fl!("title")).into(),
                                widget::text_input(fl!("title"), &task.title).on_input(Message::SetTitle).into(),
                            ]
                        )
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
                                .width(Length::Shrink)
                                .on_activate(Message::PriorityActivate),
                        ),
                    )
                    .add(
                        widget::settings::item::builder(fl!("due-date"))
                            .control(
                                widget::button(widget::text(
                                    if self.due_date.is_some() {
                                        self.due_date.as_ref().unwrap().format("%m-%d-%Y").to_string()
                                    } else {
                                        fl!("select-date")
                                    },
                                ))
                                    .on_press(Message::OpenCalendarDialog)
                            ),
                    )
                    .add(
                        widget::column::with_children(
                            vec![
                                widget::text::body(fl!("notes")).into(),
                                widget::text_input(fl!("notes"), &task.notes).on_input(Message::SetNotes).into(),
                            ]
                        )
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
        widget::settings::view_column(vec![widget::settings::view_section(fl!("details")).into()]).into()
    }

    fn sub_task_input(&self) -> Element<Message> {
        let cosmic_theme::Spacing {
            space_xs, space_s, ..
        } = theme::active().cosmic().spacing;

        row(vec![
            widget::text_input(fl!("add-sub-task"), &self.subtask_input)
                .on_input(Message::SubTaskInput)
                .on_submit(Message::AddTask)
                .width(Length::Fill)
                .into(),
            widget::button::icon(
                widget::icon::from_name("mail-send-symbolic")
                    .size(16)
                    .handle(),
            )
                .on_press(Message::AddTask)
                .into(),
        ])
            .padding([0, space_s])
            .spacing(space_xs)
            .align_items(Alignment::Center)
            .into()
    }
}
