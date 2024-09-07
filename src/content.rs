use std::collections::HashMap;

use crate::app::icon_cache::IconCache;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::iced_widget::row;
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
    editing: SecondaryMap<DefaultKey, bool>,
    task_input_ids: SecondaryMap<DefaultKey, widget::Id>,
    input: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddTask,
    Complete(DefaultKey, bool),
    Delete(DefaultKey),
    EditMode(DefaultKey, bool),
    Input(String),
    List(Option<List>),
    Select(Task),
    SetItems(Vec<Task>),
    TitleSubmit(DefaultKey),
    TitleUpdate(DefaultKey, String),
    UpdateTask(Task),
    TaskAction(TaskAction),
    Clean,
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
    Delete(DefaultKey),
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

    pub fn list_view<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        if self.tasks.is_empty() {
            return self.empty(list);
        }

        let mut items = widget::list::list_column()
            .style(theme::Container::ContextDrawer)
            .spacing(spacing.space_xxxs)
            .padding([spacing.space_none, spacing.space_xxs]);

        for (id, item) in &self.tasks {
            let item_checkbox =
                widget::checkbox("", item.status == Status::Completed, move |value| {
                    Message::Complete(id, value)
                });

            let details_button = widget::button(IconCache::get("view-more-symbolic", 18))
                .padding(spacing.space_xxs)
                .style(theme::Button::Text)
                .on_press(Message::Select(item.clone()));

            let (completed, total) = item.sub_tasks.iter().fold((0, 0), |acc, subtask| {
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
                &item.title,
                self.editing.get(id).is_some(),
                move |editing| Message::EditMode(id, editing),
            )
            .id(self.task_input_ids[id].clone())
            .on_submit(Message::TitleSubmit(id))
            .on_input(move |text| Message::TitleUpdate(id, text));

            let row = widget::row::with_capacity(4)
                .align_items(Alignment::Center)
                .spacing(spacing.space_xxs)
                .padding([spacing.space_xxxs, spacing.space_xxs])
                .push(item_checkbox)
                .push(task_item_text)
                .push(subtask_count)
                .push(details_button);

            let row = widget::context_menu(
                row,
                Some(widget::menu::items(
                    &HashMap::new(),
                    vec![
                        widget::menu::Item::Button(fl!("edit"), TaskAction::Select(id)),
                        widget::menu::Item::Button(fl!("delete"), TaskAction::Delete(id)),
                    ],
                )),
            );

            items = items.add(row);
        }

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
        row(vec![
            widget::text_input(fl!("add-new-task"), &self.input)
                .on_input(Message::Input)
                .on_submit(Message::AddTask)
                .width(Length::Fill)
                .into(),
            widget::button(IconCache::get("mail-send-symbolic", 18))
                .padding(spacing.space_xxs)
                .style(theme::Button::Suggested)
                .on_press(Message::AddTask)
                .into(),
        ])
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
            Message::List(list) => {
                self.list.clone_from(&list);
                if let Some(list) = list {
                    commands.push(Command::GetTasks(list.id().clone()));
                }
            }
            Message::TitleUpdate(id, title) => {
                if let Some(task) = self.tasks.get_mut(id) {
                    task.title = title;
                }
            }
            Message::TitleSubmit(id) => {
                if let Some(task) = self.tasks.get(id) {
                    commands.push(Command::UpdateTask(task.clone()));
                    self.editing.insert(id, false);
                }
            }
            Message::Delete(id) => {
                if let Some(task) = self.tasks.remove(id) {
                    commands.push(Command::Delete(task.id().clone()));
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
                for task in tasks {
                    let id = self.tasks.insert(task);
                    self.task_input_ids.insert(id, widget::Id::unique());
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
            Message::TaskAction(action) => match action {
                TaskAction::Select(key) => {
                    if let Some(task) = self.tasks.get(key) {
                        for command in self.update(Message::Select(task.clone())) {
                            commands.push(command);
                        }
                    }
                }
                TaskAction::Delete(key) => {
                    for command in self.update(Message::Delete(key)) {
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
