use std::collections::HashMap;

use crate::app::icon_cache::IconCache;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::prelude::CollectionWidget;
use cosmic::widget::menu::Action;
use cosmic::{theme, widget, Apply, Element};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use tasks_core::models::list::List;
use tasks_core::models::status::Status;
use tasks_core::models::task::Task;

use crate::fl;

pub struct Content {
    list: Option<List>,
    tasks: SlotMap<DefaultKey, Task>,
    sub_tasks: SecondaryMap<DefaultKey, SlotMap<DefaultKey, Task>>,
    editing: SecondaryMap<DefaultKey, bool>,
    task_input_ids: SecondaryMap<DefaultKey, widget::Id>,
    input: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddTask,
    Complete(DefaultKey, bool),
    Delete(DefaultKey, Option<DefaultKey>),
    EditMode(DefaultKey, bool),
    Input(String),
    List(Option<List>),
    Select(Task),
    SetItems(Vec<Task>),
    TaskTitleSubmit(DefaultKey),
    TaskTitleUpdate(DefaultKey, String),
    SubTaskTitleSubmit(DefaultKey, DefaultKey),
    SubTaskTitleUpdate(DefaultKey, DefaultKey, String),
    UpdateTask(Task),
    UpdateSubTask(DefaultKey, DefaultKey, Task),
    TaskAction(TaskAction),
    Clean,
    Expand(DefaultKey),
}

pub enum Command {
    Iced(cosmic::app::Command<super::app::Message>),
    GetTasks(String),
    DisplayTask(Task),
    UpdateTask(Task),
    Delete(String),
    CreateTask(Task),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TaskAction {
    Select(DefaultKey),
    Delete(DefaultKey, Option<DefaultKey>),
}

impl Action for TaskAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        Message::TaskAction(self.clone())
    }
}

impl Content {
    pub fn new() -> Self {
        Self {
            list: None,
            tasks: SlotMap::new(),
            sub_tasks: SecondaryMap::new(),
            editing: SecondaryMap::new(),
            task_input_ids: SecondaryMap::new(),
            input: String::new(),
        }
    }

    fn list_header<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;
        let export_button = widget::button(IconCache::get("larger-brush-symbolic", 18))
            .style(theme::Button::Text)
            .padding(spacing.space_xxs)
            .on_press(Message::Clean);
        let default_icon = emojis::get_by_shortcode("pencil").unwrap().to_string();
        let icon = list.icon.clone().unwrap_or(default_icon);

        widget::row::with_capacity(3)
            .align_items(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(widget::text(icon).size(spacing.space_m))
            .push(widget::text::title3(&list.name).width(Length::Fill))
            .push(export_button)
            .into()
    }

    pub fn task_view<'a>(&'a self, id: DefaultKey, task: &'a Task) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let item_checkbox = widget::checkbox("", task.status == Status::Completed, move |value| {
            Message::Complete(id, value)
        });

        let not_empty = !task.sub_tasks.is_empty();
        let icon = if task.expanded {
            "go-up-symbolic"
        } else {
            "go-down-symbolic"
        };
        let expand_button = not_empty.then(|| {
            widget::button(IconCache::get(icon, 18))
                .padding(spacing.space_xxs)
                .style(theme::Button::Text)
                .on_press(Message::Expand(id))
        });

        let (completed, total) = task.sub_tasks.iter().fold((0, 0), |acc, subtask| {
            if subtask.status == Status::Completed {
                (acc.0 + 1, acc.1 + 1)
            } else {
                (acc.0, acc.1 + 1)
            }
        });

        let subtask_count =
            widget::text(format!("{}/{}", completed, total)).style(cosmic::style::Text::Accent);

        let task_item_text = widget::editable_input(
            "",
            &task.title,
            *self.editing.get(id).unwrap_or(&false),
            move |editing| Message::EditMode(id, editing),
        )
        .trailing_icon(widget::column().into())
        .id(self.task_input_ids[id].clone())
        .on_submit(Message::TaskTitleSubmit(id))
        .on_input(move |text| Message::TaskTitleUpdate(id, text));

        widget::context_menu(
            widget::row::with_capacity(4)
                .align_items(Alignment::Center)
                .spacing(spacing.space_xxxs)
                .padding([spacing.space_xxs, spacing.space_s])
                .push(item_checkbox)
                .push(task_item_text)
                .push(subtask_count)
                .push_maybe(expand_button),
            Some(widget::menu::items(
                &HashMap::new(),
                vec![
                    widget::menu::Item::Button(fl!("edit"), TaskAction::Select(id)),
                    widget::menu::Item::Button(fl!("delete"), TaskAction::Delete(id, None)),
                ],
            )),
        )
        .into()
    }

    pub fn sub_task_view<'a>(
        &'a self,
        id: DefaultKey,
        expanded: bool,
    ) -> Option<Element<'a, Message>> {
        let spacing = theme::active().cosmic().spacing;

        let Some(sub_tasks) = self.sub_tasks.get(id) else {
            return None;
        };

        let sub_tasks: Vec<Element<'a, Message>> = sub_tasks
            .iter()
            .map(|(sub_id, sub_task)| {
                let subtask_checkbox =
                    widget::checkbox("", sub_task.status == Status::Completed, move |value| {
                        let mut subtask = sub_task.clone();
                        subtask.status = if value {
                            Status::Completed
                        } else {
                            Status::NotStarted
                        };
                        Message::UpdateSubTask(id, sub_id, subtask)
                    });

                let subtask_text = widget::editable_input(
                    "",
                    &sub_task.title,
                    *self.editing.get(sub_id).unwrap_or(&false),
                    move |editing| Message::EditMode(sub_id, editing),
                )
                .trailing_icon(widget::column().into())
                .id(self.task_input_ids[id].clone())
                .on_submit(Message::SubTaskTitleSubmit(id, sub_id))
                .on_input(move |text| Message::SubTaskTitleUpdate(id, sub_id, text));

                widget::context_menu(
                    widget::row()
                        .align_items(Alignment::Center)
                        .spacing(spacing.space_xxxs)
                        .padding([spacing.space_xxs, spacing.space_s])
                        .push(subtask_checkbox)
                        .push(subtask_text)
                        .apply(widget::container)
                        .style(theme::Container::Card),
                    Some(widget::menu::items(
                        &HashMap::new(),
                        vec![widget::menu::Item::Button(
                            fl!("delete"),
                            TaskAction::Delete(id, Some(sub_id)),
                        )],
                    )),
                )
                .into()
            })
            .collect();

        if !sub_tasks.is_empty() && expanded {
            Some(
                widget::column::with_children(sub_tasks)
                    .spacing(spacing.space_xxs)
                    .padding(spacing.space_xs)
                    .into(),
            )
        } else {
            None
        }
    }

    pub fn list_view<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        if self.tasks.is_empty() {
            return self.empty(list);
        }

        let items = self
            .tasks
            .iter()
            .map(|(id, task)| {
                widget::column()
                    .push(self.task_view(id, task))
                    .push_maybe(self.sub_task_view(id, task.expanded))
                    .apply(widget::container)
                    .style(theme::Container::ContextDrawer)
                    .into()
            })
            .collect();

        let items = widget::column::with_children(items)
            .spacing(spacing.space_xs)
            .padding([spacing.space_none, spacing.space_xxs]);

        widget::column::with_capacity(2)
            .spacing(spacing.space_xxs)
            .push(self.list_header(list))
            .push(items)
            .apply(widget::container)
            .height(Length::Shrink)
            .apply(widget::scrollable)
            .height(Length::Fill)
            .into()
    }

    pub fn empty<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let container = widget::container(
            widget::column::with_children(vec![
                IconCache::get("task-past-due-symbolic", 56).into(),
                widget::text::title1(fl!("no-tasks")).into(),
                widget::text(fl!("no-tasks-suggestion")).into(),
            ])
            .spacing(10)
            .align_items(Alignment::Center),
        )
        .align_y(Vertical::Center)
        .align_x(Horizontal::Center)
        .height(Length::Fill)
        .width(Length::Fill);

        widget::column::with_capacity(2)
            .spacing(spacing.space_xxs)
            .push(self.list_header(list))
            .push(container)
            .into()
    }

    pub fn new_task_view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;
        widget::row::with_children(vec![widget::text_input(fl!("add-new-task"), &self.input)
            .on_input(Message::Input)
            .on_submit(Message::AddTask)
            .width(Length::Fill)
            .into()])
        .padding(spacing.space_xxs)
        .spacing(spacing.space_xxs)
        .align_items(Alignment::Center)
        .apply(widget::container)
        .style(cosmic::style::Container::ContextDrawer)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Vec<Command> {
        let mut commands = Vec::new();
        match message {
            Message::Expand(id) => {
                if let Some(task) = self.tasks.get_mut(id) {
                    task.expanded = !task.expanded;
                    commands.push(Command::UpdateTask(task.clone()));
                }
            }
            Message::List(list) => {
                self.list.clone_from(&list);
                if let Some(list) = list {
                    commands.push(Command::GetTasks(list.id().clone()));
                }
            }
            Message::TaskTitleUpdate(id, title) => {
                if let Some(task) = self.tasks.get_mut(id) {
                    task.title = title;
                }
            }
            Message::TaskTitleSubmit(id) => {
                if let Some(task) = self.tasks.get(id) {
                    commands.push(Command::UpdateTask(task.clone()));
                    self.editing.remove(id);
                }
            }
            Message::SubTaskTitleUpdate(id, sub_id, title) => {
                if let Some(sm) = self.sub_tasks.get_mut(id) {
                    if let Some(sub_task) = sm.get_mut(sub_id) {
                        sub_task.title = title;
                    }
                }
            }
            Message::SubTaskTitleSubmit(id, sub_id) => {
                if let Some(slotmap) = self.sub_tasks.get(id) {
                    if let Some(sub_task) = slotmap.get(sub_id) {
                        let task = self.tasks.get_mut(id);
                        if let Some(task) = task {
                            let sub_task =
                                task.sub_tasks.iter_mut().find(|t| t.id() == sub_task.id());
                            if let Some(sub_task) = sub_task {
                                *sub_task = sub_task.clone();
                                commands.push(Command::UpdateTask(task.clone()));
                                self.editing.remove(sub_id);
                            }
                        }
                    }
                }
            }
            Message::Delete(id, sub_id) => {
                let Some(sub_id) = sub_id else {
                    if let Some(task) = self.tasks.remove(id) {
                        commands.push(Command::Delete(task.id().clone()));
                    }
                    return commands;
                };
                if let (Some(task), Some(slotmap)) =
                    (self.tasks.get_mut(id), self.sub_tasks.get_mut(id))
                {
                    if let Some(subtask) = slotmap.remove(sub_id) {
                        task.sub_tasks.retain(|t| t.id() != subtask.id());
                        commands.push(Command::UpdateTask(task.clone()));
                    }
                }
            }
            Message::EditMode(id, editing) => {
                self.editing.insert(id, editing);
                if editing {
                    commands.push(Command::Iced(widget::text_input::focus(
                        self.task_input_ids[id].clone(),
                    )));
                } else if let Some(task) = self.tasks.get(id) {
                    commands.push(Command::UpdateTask(task.clone()));
                }
            }
            Message::SetItems(tasks) => {
                self.tasks.clear();
                self.task_input_ids.clear();
                self.editing.clear();
                for task in tasks {
                    let id = self.tasks.insert(task.clone());
                    self.task_input_ids.insert(id, widget::Id::unique());

                    let mut sub_tasks = SlotMap::new();
                    for subtask in task.sub_tasks {
                        let sub_task_id = sub_tasks.insert(subtask);
                        self.task_input_ids
                            .insert(sub_task_id, widget::Id::unique());
                    }
                    self.sub_tasks.insert(id, sub_tasks);
                }
            }
            Message::Select(task) => {
                commands.push(Command::DisplayTask(task));
            }
            Message::Complete(id, complete) => {
                let task = self.tasks.get_mut(id);
                if let Some(task) = task {
                    task.status = if complete {
                        Status::Completed
                    } else {
                        Status::NotStarted
                    };
                    commands.push(Command::UpdateTask(task.clone()));
                }
            }
            Message::Input(input) => self.input = input,
            Message::AddTask => {
                if let Some(list) = &self.list {
                    if !self.input.is_empty() {
                        let task = Task::new(self.input.clone(), list.id().clone());
                        commands.push(Command::CreateTask(task.clone()));
                        let id = self.tasks.insert(task);
                        self.task_input_ids.insert(id, widget::Id::unique());
                        self.input.clear();
                    }
                }
            }
            Message::UpdateTask(updated_task) => {
                let task = self
                    .tasks
                    .values_mut()
                    .find(|t| t.id() == updated_task.id());
                if let Some(task) = task {
                    *task = updated_task.clone();
                    commands.push(Command::UpdateTask(task.clone()));
                }
            }
            Message::UpdateSubTask(id, sub_id, updated_subtask) => {
                let task = self.tasks.get_mut(id);
                if let Some(task) = task {
                    let sub_task = task
                        .sub_tasks
                        .iter_mut()
                        .find(|t| t.id() == updated_subtask.id());
                    if let Some(sub_task) = sub_task {
                        *sub_task = updated_subtask.clone();
                        commands.push(Command::UpdateTask(task.clone()));
                        if let Some(sm) = self.sub_tasks.get_mut(id) {
                            let sub_task = sm.get_mut(sub_id);
                            if let Some(sub_task) = sub_task {
                                *sub_task = updated_subtask.clone();
                            }
                        };
                    }
                }
            }
            Message::TaskAction(action) => match action {
                TaskAction::Select(key) => {
                    if let Some(task) = self.tasks.get(key) {
                        for command in self.update(Message::Select(task.clone())) {
                            commands.push(command);
                        }
                    }
                }
                TaskAction::Delete(id, sub_id) => {
                    for command in self.update(Message::Delete(id, sub_id)) {
                        commands.push(command);
                    }
                }
            },
            Message::Clean => todo!("Clean up task list"),
        }
        commands
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let Some(ref list) = self.list else {
            return widget::container(
                widget::column::with_children(vec![
                    IconCache::get("applications-office-symbolic", 56).into(),
                    widget::text::title1(fl!("no-list-selected")).into(),
                    widget::text(fl!("no-list-suggestion")).into(),
                ])
                .spacing(10)
                .align_items(Alignment::Center),
            )
            .align_y(Vertical::Center)
            .align_x(Horizontal::Center)
            .height(Length::Fill)
            .width(Length::Fill)
            .into();
        };

        widget::column::with_capacity(2)
            .push(self.list_view(list))
            .push(self.new_task_view())
            .spacing(spacing.space_xxs)
            .apply(widget::container)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}
