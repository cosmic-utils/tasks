use std::ops::IndexMut;

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
    pub priority_model: segmented_button::Model<segmented_button::SingleSelect>,
    pub subtask_input: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Rename(String),
    CompleteSubTask(usize, bool),
    Favorite(bool),
    PriorityActivate(Entity),
    SubTaskInput(String),
    AddTask,
}

pub enum Command {
    Update(Task),
    Rename(String, String),
    Favorite(String, bool),
    PriorityActivate(String, Priority),
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
            priority_model,
            subtask_input: String::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Vec<Command> {
        let mut commands = vec![];
        match message {
            Message::Rename(title) => {
                if let Some(ref mut task) = &mut self.task {
                    task.title = title.clone();
                    commands.push(Command::Rename(task.id.clone(), title));
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
        }
        commands
    }

    pub fn view(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xs, .. } = theme::active().cosmic().spacing;

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
                        widget::container(
                            widget::text_input(fl!("title"), &task.title).on_input(Message::Rename),
                        )
                        .padding([0, 10, 0, 10]),
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
