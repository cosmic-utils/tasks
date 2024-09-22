use crate::app::icon_cache::IconCache;
use chrono::{NaiveDate, TimeZone, Utc};
use cosmic::iced::{Alignment, Length};
use cosmic::widget::segmented_button::Entity;
use cosmic::widget::{segmented_button, Row};
use cosmic::{widget, Apply, Element};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use tasks_core::models::priority::Priority;
use tasks_core::models::status::Status;
use tasks_core::models::task::Task;

use crate::fl;

pub struct Details {
    pub task: Option<Task>,
    pub priority_model: segmented_button::Model<segmented_button::SingleSelect>,
    pub subtask_input: String,
    pub subtasks: SlotMap<DefaultKey, Task>,
    pub editing: SecondaryMap<DefaultKey, bool>,
    pub sub_task_input_ids: SecondaryMap<DefaultKey, widget::Id>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetTitle(String),
    SetNotes(String),
    Favorite(bool),
    CompleteSubTask(DefaultKey, bool),
    DeleteSubTask(DefaultKey),
    SetSubTaskTitle(DefaultKey, String),
    SubTaskEditDone,
    EditMode(DefaultKey, bool),
    PriorityActivate(Entity),
    SubTaskInput(String),
    AddTask,
    OpenCalendarDialog,
    SetDueDate(NaiveDate),
}

pub enum Command {
    Focus(widget::Id),
    UpdateTask(Task),
    OpenCalendarDialog,
    Iced(cosmic::app::Command<super::app::Message>),
}

impl Details {
    pub fn new() -> Self {
        let priority_model = segmented_button::ModelBuilder::default()
            .insert(|entity| {
                entity
                    .icon(IconCache::get("flag-outline-thin-symbolic", 16))
                    .data(Priority::Low)
            })
            .insert(|entity| {
                entity
                    .icon(IconCache::get("flag-outline-thick-symbolic", 16))
                    .data(Priority::Normal)
            })
            .insert(|entity| {
                entity
                    .icon(IconCache::get("flag-filled-symbolic", 16))
                    .data(Priority::High)
            })
            .build();

        Self {
            task: None,
            priority_model,
            subtask_input: String::new(),
            subtasks: SlotMap::new(),
            editing: SecondaryMap::new(),
            sub_task_input_ids: SecondaryMap::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Vec<Command> {
        let mut commands = vec![];
        match message {
            Message::SetTitle(title) => {
                if let Some(ref mut task) = &mut self.task {
                    task.title.clone_from(&title);
                }
            }
            Message::SetNotes(notes) => {
                if let Some(ref mut task) = &mut self.task {
                    task.notes.clone_from(&notes);
                }
            }
            Message::Favorite(favorite) => {
                if let Some(ref mut task) = &mut self.task {
                    task.favorite = favorite;
                }
            }
            Message::EditMode(id, editing) => {
                self.editing.insert(id, editing);
                if editing {
                    commands.push(Command::Iced(widget::text_input::focus(
                        self.sub_task_input_ids[id].clone(),
                    )));
                } else if let Some(task) = self.subtasks.get(id) {
                    commands.push(Command::UpdateTask(task.clone()));
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
            Message::SetSubTaskTitle(id, title) => {
                let task = self.subtasks.get_mut(id);
                if let Some(task) = task {
                    task.title.clone_from(&title);
                }
            }
            Message::CompleteSubTask(id, completed) => {
                let task = self.subtasks.get_mut(id);
                if let Some(task) = task {
                    task.status = if completed {
                        Status::Completed
                    } else {
                        Status::NotStarted
                    };
                }
            }
            Message::DeleteSubTask(id) => {
                self.subtasks.remove(id);
            }
            Message::SubTaskEditDone => {
                commands.push(Command::Focus(widget::Id::new("new_sub_task_input")));
            }
            Message::SubTaskInput(text) => {
                self.subtask_input = text;
            }
            Message::AddTask => {
                if let Some(ref mut task) = &mut self.task {
                    if !self.subtask_input.is_empty() {
                        let sub_task = Task::new(self.subtask_input.clone(), task.id().clone());
                        task.sub_tasks.push(sub_task.clone());
                        let id = self.subtasks.insert(sub_task);
                        self.sub_task_input_ids.insert(id, widget::Id::unique());
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

        if let Some(task) = &mut self.task {
            task.sub_tasks = self.subtasks.values().cloned().collect();
            commands.push(Command::UpdateTask(task.clone()));
        }

        commands
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;

        let Some(task) = self.task.as_ref() else {
            return widget::settings::section().title(fl!("details")).into();
        };

        let sub_tasks: Vec<Row<Message>> = self
            .subtasks
            .iter()
            .map(|(id, sub_task)| {
                let item_checkbox =
                    widget::checkbox("", sub_task.status == Status::Completed, move |value| {
                        Message::CompleteSubTask(id, value)
                    })
                    .into();

                let sub_task_item = widget::editable_input(
                    fl!("title"),
                    sub_task.title.clone(),
                    *self.editing.get(id).unwrap_or(&false),
                    move |editing| Message::EditMode(id, editing),
                )
                .trailing_icon(widget::column().into())
                .id(self.sub_task_input_ids[id].clone())
                .on_input(move |title| Message::SetSubTaskTitle(id, title))
                .on_submit(Message::SubTaskEditDone)
                .into();

                let delete_button =
                    widget::button::custom(IconCache::get("cross-small-symbolic", 18))
                        .style(widget::button::Style::Text)
                        .on_press(Message::DeleteSubTask(id))
                        .into();

                widget::settings::item_row(vec![item_checkbox, sub_task_item, delete_button])
            })
            .collect();

        let mut sub_tasks_section = widget::settings::section().title(fl!("sub-tasks"));
        for sub_task in sub_tasks {
            sub_tasks_section = sub_tasks_section.add(sub_task);
        }

        sub_tasks_section = sub_tasks_section.add(
            widget::settings::item_row(vec![self.sub_task_input()]).padding([
                spacing.space_xxs,
                spacing.space_xs,
                spacing.space_xxs,
                spacing.space_xs,
            ]),
        );

        let details = widget::settings::section()
            .title(fl!("details"))
            .add(
                widget::settings::item_row(vec![widget::text_input(fl!("title"), &task.title)
                    .label("Title")
                    .on_input(Message::SetTitle)
                    .into()])
                .padding([
                    spacing.space_xxs,
                    spacing.space_xs,
                    spacing.space_xxs,
                    spacing.space_xs,
                ]),
            )
            .add(
                widget::settings::item(
                    fl!("favorite"),
                    widget::checkbox("", task.favorite, Message::Favorite),
                )
                .height(Length::Fixed(35.0)),
            )
            .add(widget::settings::item(
                fl!("priority"),
                widget::segmented_control::horizontal(&self.priority_model)
                    .button_alignment(Alignment::Center)
                    .width(Length::Shrink)
                    .on_activate(Message::PriorityActivate),
            ))
            .add(widget::settings::item(
                fl!("due-date"),
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
            ))
            .add(widget::settings::item_row(vec![widget::text_input(
                fl!("notes"),
                &task.notes,
            )
            .label("Notes")
            .on_input(Message::SetNotes)
            .apply(widget::container)
            .padding([
                spacing.space_none,
                spacing.space_none,
                spacing.space_xs,
                spacing.space_none,
            ])
            .into()]));
        widget::settings::view_column(vec![details.into(), sub_tasks_section.into()]).into()
    }

    fn sub_task_input(&self) -> Element<Message> {
        widget::text_input(fl!("add-sub-task"), &self.subtask_input)
            .id(widget::Id::new("new_sub_task_input"))
            .on_input(Message::SubTaskInput)
            .on_submit(Message::AddTask)
            .width(Length::Fill)
            .into()
    }
}
