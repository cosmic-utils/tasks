use std::collections::HashMap;

use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Alignment, Length, Subscription,
    },
    iced_widget::row,
    theme,
    widget::{self, menu::Action as MenuAction},
    Apply, Element,
};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};

use crate::{app::config, core::storage::LocalStorage};
use crate::{
    app::icons,
    core::models::{self, List, Status},
    fl,
};

pub struct Content {
    list: Option<List>,
    tasks: SlotMap<DefaultKey, models::Task>,
    sub_tasks: SlotMap<DefaultKey, models::Task>,
    task_editing: SecondaryMap<DefaultKey, bool>,
    sub_task_editing: SecondaryMap<DefaultKey, bool>,
    task_input_ids: SecondaryMap<DefaultKey, widget::Id>,
    sub_task_input_ids: SecondaryMap<DefaultKey, widget::Id>,
    config: config::TasksConfig,
    input: String,
    storage: LocalStorage,
    context_menu_open: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    TaskAdd,

    TaskExpand(DefaultKey),
    TaskAddSubTask(DefaultKey),
    TaskComplete(DefaultKey, bool),
    TaskDelete(DefaultKey),
    TaskToggleTitleEditMode(DefaultKey, bool),
    TaskTitleInput(String),
    TaskOpenDetails(DefaultKey),
    TaskTitleSubmit(DefaultKey),
    TaskTitleUpdate(DefaultKey, String),

    SubTaskExpand(DefaultKey),
    SubTaskAddSubTask(DefaultKey),
    SubTaskComplete(DefaultKey, bool),
    SubTaskDelete(DefaultKey),
    SubTaskToggleTitleEditMode(DefaultKey, bool),
    SubTaskTitleSubmit(DefaultKey),
    SubTaskTitleUpdate(DefaultKey, String),
    SubTaskOpenDetails(DefaultKey),

    ToggleHideCompleted,

    SetList(Option<List>),
    SetTasks(Vec<models::Task>),
    SetConfig(config::TasksConfig),
    RefreshTask(models::Task),
    Empty,
    ContextMenuOpen(bool),
}

pub enum Output {
    ToggleHideCompleted(models::List),
    Focus(widget::Id),
    OpenTaskDetails(models::Task),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TaskAction {
    AddSubTask(DefaultKey),
    Edit(DefaultKey),
    Delete(DefaultKey),
}

impl MenuAction for TaskAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            TaskAction::Edit(id) => Message::TaskOpenDetails(*id),
            TaskAction::AddSubTask(id) => Message::TaskAddSubTask(*id),
            TaskAction::Delete(id) => Message::TaskDelete(*id),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SubTaskAction {
    AddSubTask(DefaultKey),
    Edit(DefaultKey),
    Delete(DefaultKey),
}

impl MenuAction for SubTaskAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            SubTaskAction::Edit(id) => Message::SubTaskOpenDetails(*id),
            SubTaskAction::AddSubTask(id) => Message::SubTaskAddSubTask(*id),
            SubTaskAction::Delete(id) => Message::SubTaskDelete(*id),
        }
    }
}

impl Content {
    pub fn new(storage: LocalStorage) -> Self {
        Self {
            list: None,
            tasks: SlotMap::new(),
            sub_tasks: SlotMap::new(),
            task_editing: SecondaryMap::new(),
            sub_task_editing: SecondaryMap::new(),
            task_input_ids: SecondaryMap::new(),
            sub_task_input_ids: SecondaryMap::new(),
            input: String::new(),
            config: config::TasksConfig::config(),
            storage,
            context_menu_open: false,
        }
    }

    fn list_header<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let mut hide_completed_button =
            widget::button::icon(icons::get_handle("check-round-outline-symbolic", 18))
                .selected(list.hide_completed || self.config.hide_completed)
                .padding(spacing.space_xxs);

        if !self.config.hide_completed {
            hide_completed_button = hide_completed_button.on_press(Message::ToggleHideCompleted);
        }

        let default_icon = emojis::get_by_shortcode("pencil").unwrap().to_string();
        let icon = list.icon.clone().unwrap_or(default_icon);

        widget::row::with_capacity(3)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(widget::text(icon).size(spacing.space_m))
            .push(widget::text::title3(&list.name).width(Length::Fill))
            .push(hide_completed_button)
            .into()
    }

    pub fn list_view<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        if self.tasks.is_empty() {
            return self.empty(list);
        }

        let items = self
            .tasks
            .iter()
            .map(|(id, task)| self.task_view(id, task))
            .collect::<Vec<_>>();

        let items = widget::column::with_children(items)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs]);

        widget::column::with_capacity(2)
            .push(self.list_header(list))
            .push(items)
            .padding([spacing.space_none, spacing.space_m])
            .spacing(spacing.space_xs)
            .apply(widget::container)
            .height(Length::Shrink)
            .apply(widget::scrollable)
            .height(Length::Fill)
            .into()
    }

    pub fn task_view<'a>(&'a self, id: DefaultKey, task: &'a models::Task) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let sub_tasks = self
            .sub_tasks
            .values()
            .filter(|sub_task| task.sub_tasks.iter().any(|st| st.id == sub_task.id))
            .collect::<Vec<_>>();

        let item_checkbox = widget::checkbox("", task.status == Status::Completed)
            .on_toggle(move |value| Message::TaskComplete(id, value));

        let not_empty = !sub_tasks.is_empty();
        let icon = if task.expanded {
            "go-up-symbolic"
        } else {
            "go-down-symbolic"
        };
        let expand_button = not_empty.then(|| {
            widget::button::icon(icons::get_handle(icon, 18))
                .padding(spacing.space_xxs)
                .on_press(Message::TaskExpand(id))
        });

        let more_button = widget::menu::MenuBar::new(vec![widget::menu::Tree::with_children(
            cosmic::widget::button::icon(icons::get_handle("view-more-symbolic", 18))
                .on_press(Message::Empty),
            widget::menu::items(
                &HashMap::new(),
                vec![
                    widget::menu::Item::Button(fl!("edit"), None, TaskAction::Edit(id)),
                    widget::menu::Item::Button(
                        fl!("add-sub-task"),
                        None,
                        TaskAction::AddSubTask(id),
                    ),
                    widget::menu::Item::Button(fl!("delete"), None, TaskAction::Delete(id)),
                ],
            ),
        )])
        .item_height(widget::menu::ItemHeight::Dynamic(40))
        .item_width(widget::menu::ItemWidth::Uniform(260))
        .spacing(4.0);

        let (completed, total) = sub_tasks.iter().fold((0, 0), |acc, subtask| {
            if subtask.status == Status::Completed {
                (acc.0 + 1, acc.1 + 1)
            } else {
                (acc.0, acc.1 + 1)
            }
        });

        let subtask_count = if total > 0 {
            Some(widget::text(format!("{}/{}", completed, total)))
        } else {
            None
        };

        let task_item_text = widget::editable_input(
            "",
            &task.title,
            *self.task_editing.get(id).unwrap_or(&false),
            move |editing| Message::TaskToggleTitleEditMode(id, editing),
        )
        .size(13)
        .trailing_icon(widget::column().into())
        .id(self.task_input_ids[id].clone())
        .on_submit(move |_| Message::TaskTitleSubmit(id))
        .on_input(move |text| Message::TaskTitleUpdate(id, text));

        let row = widget::row::with_capacity(5)
            .align_y(Alignment::Center)
            .spacing(spacing.space_xxxs)
            .padding([spacing.space_xxs, spacing.space_s])
            .push(item_checkbox)
            .push(task_item_text)
            .push_maybe(expand_button)
            .push_maybe(subtask_count)
            .push(more_button);

        let mut column = widget::column::with_capacity(2).push(row);

        if task.expanded && !sub_tasks.is_empty() {
            let subtask_elements = self
                .sub_tasks
                .iter()
                .filter(|(_, sub_task)| task.sub_tasks.iter().any(|st| st.id == sub_task.id))
                .map(|(sub_id, sub_task)| {
                    widget::container(self.sub_task_view(sub_id, sub_task))
                        .padding([0, 0, 0, spacing.space_l])
                        .into()
                })
                .collect::<Vec<_>>();
            column = column.push(widget::column::with_children(subtask_elements));
        }

        column
            .padding(spacing.space_xxs)
            .apply(widget::container)
            .class(cosmic::style::Container::ContextDrawer)
            .into()
    }

    pub fn sub_task_view<'a>(
        &'a self,
        id: DefaultKey,
        task: &'a models::Task,
    ) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let sub_tasks = self
            .sub_tasks
            .values()
            .filter(|sub_task| task.sub_tasks.iter().any(|st| st.id == sub_task.id))
            .collect::<Vec<_>>();

        let item_checkbox = widget::checkbox("", task.status == Status::Completed)
            .on_toggle(move |value| Message::SubTaskComplete(id, value));

        let not_empty = !sub_tasks.is_empty();
        let icon = if task.expanded {
            "go-up-symbolic"
        } else {
            "go-down-symbolic"
        };
        let expand_button = not_empty.then(|| {
            widget::button::icon(icons::get_handle(icon, 18))
                .padding(spacing.space_xxs)
                .on_press(Message::SubTaskExpand(id))
        });

        let more_button = widget::menu::MenuBar::new(vec![widget::menu::Tree::with_children(
            cosmic::widget::button::icon(icons::get_handle("view-more-symbolic", 18))
                .on_press(Message::Empty),
            widget::menu::items(
                &HashMap::new(),
                vec![
                    widget::menu::Item::Button(fl!("edit"), None, SubTaskAction::Edit(id)),
                    widget::menu::Item::Button(
                        fl!("add-sub-task"),
                        None,
                        SubTaskAction::AddSubTask(id),
                    ),
                    widget::menu::Item::Button(fl!("delete"), None, SubTaskAction::Delete(id)),
                ],
            ),
        )])
        .item_height(widget::menu::ItemHeight::Dynamic(40))
        .item_width(widget::menu::ItemWidth::Uniform(260))
        .spacing(4.0);

        let (completed, total) = sub_tasks.iter().fold((0, 0), |acc, subtask| {
            if subtask.status == Status::Completed {
                (acc.0 + 1, acc.1 + 1)
            } else {
                (acc.0, acc.1 + 1)
            }
        });

        let subtask_count = if total > 0 {
            Some(widget::text(format!("{}/{}", completed, total)))
        } else {
            None
        };

        let task_item_text = widget::editable_input(
            "",
            &task.title,
            *self.sub_task_editing.get(id).unwrap_or(&false),
            move |editing| Message::SubTaskToggleTitleEditMode(id, editing),
        )
        .size(13)
        .trailing_icon(widget::column().into())
        .id(self.sub_task_input_ids[id].clone())
        .on_submit(move |_| Message::SubTaskTitleSubmit(id))
        .on_input(move |text| Message::SubTaskTitleUpdate(id, text));

        let row = widget::row::with_capacity(4)
            .align_y(Alignment::Center)
            .spacing(spacing.space_xxxs)
            .padding([spacing.space_xxs, spacing.space_s])
            .push(item_checkbox)
            .push(task_item_text)
            .push_maybe(expand_button)
            .push_maybe(subtask_count)
            .push(more_button);

        let mut column = widget::column::with_capacity(2).push(row);

        if task.expanded && !sub_tasks.is_empty() {
            let subtask_elements = self
                .sub_tasks
                .iter()
                .filter(|(_, sub_task)| task.sub_tasks.iter().any(|st| st.id == sub_task.id))
                .map(|(sub_id, sub_task)| {
                    widget::container(self.sub_task_view(sub_id, sub_task))
                        .padding([0, 0, 0, spacing.space_l])
                        .into()
                })
                .collect::<Vec<_>>();
            column = column.push(widget::column::with_children(subtask_elements));
        }

        column
            .apply(widget::container)
            .class(cosmic::style::Container::ContextDrawer)
            .into()
    }

    pub fn empty<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let container = widget::container(
            widget::column::with_children(vec![
                icons::get_icon("task-past-due-symbolic", 56).into(),
                widget::text::title1(fl!("no-tasks")).into(),
                widget::text(fl!("no-tasks-suggestion")).into(),
            ])
            .spacing(10)
            .align_x(Alignment::Center),
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
                .on_input(Message::TaskTitleInput)
                .on_submit(|_| Message::TaskAdd)
                .width(Length::Fill)
                .into(),
            widget::button::icon(icons::get_handle("mail-send-symbolic", 18))
                .padding(spacing.space_xxs)
                .class(cosmic::style::Button::Suggested)
                .on_press(Message::TaskAdd)
                .into(),
        ])
        .padding(spacing.space_xxs)
        .spacing(spacing.space_xxs)
        .align_y(Alignment::Center)
        .into()
    }

    fn populate_task_slotmap(&mut self, tasks: Vec<models::Task>) {
        for task in tasks {
            let task_id = self.tasks.insert(task.clone());
            self.task_input_ids.insert(task_id, widget::Id::unique());
            self.task_editing.insert(task_id, false);
            if !task.sub_tasks.is_empty() {
                self.populate_sub_task_slotmap(task.sub_tasks);
            }
        }
    }

    fn populate_sub_task_slotmap(&mut self, tasks: Vec<models::Task>) {
        for task in tasks {
            let task_id = self.sub_tasks.insert(task.clone());
            self.sub_task_input_ids
                .insert(task_id, widget::Id::unique());
            self.sub_task_editing.insert(task_id, false);
            if !task.sub_tasks.is_empty() {
                self.populate_sub_task_slotmap(task.sub_tasks);
            }
        }
    }

    pub fn update(&mut self, message: Message) -> Vec<Output> {
        let mut tasks = Vec::new();
        match message {
            Message::Empty => return tasks,
            Message::ContextMenuOpen(open) => {
                self.context_menu_open = open;
            }
            Message::SetTasks(tasks) => {
                self.tasks.clear();
                self.task_input_ids.clear();
                self.task_editing.clear();
                self.sub_tasks.clear();
                self.sub_task_input_ids.clear();
                self.sub_task_editing.clear();
                self.input.clear();
                self.populate_task_slotmap(tasks);
            }
            Message::SetList(list) => {
                match (&self.list, &list) {
                    (Some(current), Some(list)) => {
                        if current.id != list.id {
                            match self.storage.tasks(list) {
                                Ok(tasks) => {
                                    self.update(Message::SetTasks(tasks));
                                }
                                Err(error) => {
                                    tracing::error!("Failed to fetch tasks for list: {:?}", error)
                                }
                            }
                        }
                    }
                    (None, Some(list)) => match self.storage.tasks(list) {
                        Ok(tasks) => {
                            self.update(Message::SetTasks(tasks));
                        }
                        Err(error) => {
                            tracing::error!("Failed to fetch tasks for list: {:?}", error)
                        }
                    },
                    _ => {}
                }
                self.list.clone_from(&list);
            }
            Message::SetConfig(config) => {
                self.config = config;
            }
            Message::RefreshTask(refreshed_task) => {
                if let Some((id, _)) = self.tasks.iter().find(|(_, t)| t.id == refreshed_task.id) {
                    if let Some(task) = self.tasks.get_mut(id) {
                        *task = refreshed_task.clone();
                    }
                } else if let Some((id, _)) = self
                    .sub_tasks
                    .iter()
                    .find(|(_, t)| t.id == refreshed_task.id)
                {
                    if let Some(task) = self.sub_tasks.get_mut(id) {
                        *task = refreshed_task.clone();
                    }
                } else {
                    tracing::warn!("Task with ID {:?} not found", refreshed_task.id);
                }
            }
            Message::TaskOpenDetails(id) => match self.tasks.get(id) {
                Some(task) => tasks.push(Output::OpenTaskDetails(task.clone())),
                None => tracing::warn!("Task with ID {:?} not found", id),
            },
            Message::TaskExpand(default_key) => {
                if let Some(task) = self.tasks.get_mut(default_key) {
                    task.expanded = !task.expanded;
                    if let Err(error) = self.storage.update_task(task) {
                        tracing::error!("Failed to update task: {:?}", error);
                    }
                }
            }
            Message::TaskAdd => {
                if let Some(list) = &self.list {
                    if !self.input.is_empty() {
                        let task = models::Task::new(self.input.clone(), list.tasks_path());
                        match self.storage.create_task(&task) {
                            Ok(task) => {
                                let id = self.tasks.insert(task);
                                self.task_input_ids.insert(id, widget::Id::unique());
                                self.input.clear();
                            }
                            Err(error) => {
                                tracing::error!("Failed to create task: {:?}", error);
                            }
                        }
                    }
                }
            }
            Message::TaskToggleTitleEditMode(id, editing) => {
                self.task_editing.insert(id, editing);
                if editing {
                    tasks.push(Output::Focus(self.task_input_ids[id].clone()));
                } else if let Some(task) = self.tasks.get(id) {
                    if let Err(error) = self.storage.update_task(task) {
                        tracing::error!("Failed to update task: {:?}", error);
                    }
                }
            }
            Message::TaskTitleInput(input) => self.input = input,
            Message::TaskTitleSubmit(id) => {
                if let Some(task) = self.tasks.get(id) {
                    match self.storage.update_task(task) {
                        Ok(_) => {
                            self.task_editing.insert(id, false);
                        }
                        Err(error) => tracing::error!("Failed to update task: {:?}", error),
                    }
                }
            }
            Message::TaskTitleUpdate(id, title) => {
                if let Some(task) = self.tasks.get_mut(id) {
                    task.title = title;
                }
            }
            Message::TaskDelete(id) => {
                if let Some(task) = self.tasks.remove(id) {
                    if let Err(error) = self.storage.delete_task(&task) {
                        tracing::error!("Failed to delete task: {:?}", error);
                    }
                }
            }
            Message::TaskComplete(id, complete) => {
                let task = self.tasks.get_mut(id);
                if let Some(task) = task {
                    task.status = if complete {
                        Status::Completed
                    } else {
                        Status::NotStarted
                    };
                    if let Err(error) = self.storage.update_task(task) {
                        tracing::error!("Failed to update task: {:?}", error);
                    }
                }
            }
            Message::TaskAddSubTask(id) => {
                if let Some(task) = self.tasks.get_mut(id) {
                    task.expanded = true;
                    let sub_task = models::Task::new("".to_string(), task.sub_tasks_path());
                    match self.storage.create_task(&sub_task) {
                        Ok(sub_task) => {
                            task.sub_tasks.push(sub_task.clone());
                            if let Err(error) = self.storage.update_task(task) {
                                tracing::error!("Failed to update task with sub-task: {:?}", error);
                            }

                            let sub_task_id = self.sub_tasks.insert(sub_task);
                            self.sub_task_input_ids
                                .insert(sub_task_id, widget::Id::unique());
                            self.sub_task_editing.insert(sub_task_id, false);
                            tasks.push(Output::Focus(self.sub_task_input_ids[sub_task_id].clone()));
                        }
                        Err(error) => {
                            tracing::error!("Failed to add sub-task: {:?}", error);
                        }
                    }
                }
            }
            Message::ToggleHideCompleted => {
                if let Some(ref mut list) = self.list {
                    list.hide_completed = !list.hide_completed;
                    tasks.push(Output::ToggleHideCompleted(list.clone()));
                }
            }
            Message::SubTaskToggleTitleEditMode(id, editing) => {
                self.sub_task_editing.insert(id, editing);
                if editing {
                    tasks.push(Output::Focus(self.sub_task_input_ids[id].clone()));
                } else if let Some(task) = self.sub_tasks.get(id) {
                    if let Err(error) = self.storage.update_task(task) {
                        tracing::error!("Failed to update sub-task: {:?}", error);
                    }
                }
            }
            Message::SubTaskTitleSubmit(id) => {
                if let Some(task) = self.sub_tasks.get(id) {
                    match self.storage.update_task(task) {
                        Ok(_) => {
                            self.sub_task_editing.insert(id, false);
                        }
                        Err(error) => tracing::error!("Failed to update sub-task: {:?}", error),
                    }
                }
            }
            Message::SubTaskTitleUpdate(id, title) => {
                if let Some(task) = self.sub_tasks.get_mut(id) {
                    task.title = title;
                    if let Err(error) = self.storage.update_task(task) {
                        tracing::error!("Failed to update sub-task: {:?}", error);
                    }
                }
            }
            Message::SubTaskOpenDetails(id) => {
                if let Some(task) = self.sub_tasks.get(id) {
                    tasks.push(Output::OpenTaskDetails(task.clone()));
                } else {
                    tracing::warn!("Sub-task with ID {:?} not found", id);
                }
            }
            Message::SubTaskAddSubTask(id) => {
                if let Some(task) = self.sub_tasks.get_mut(id) {
                    task.expanded = true;
                    let sub_task = models::Task::new("".to_string(), task.sub_tasks_path());
                    match self.storage.create_task(&sub_task) {
                        Ok(sub_task) => {
                            task.sub_tasks.push(sub_task.clone());
                            if let Err(error) = self.storage.update_task(task) {
                                tracing::error!("Failed to update task with sub-task: {:?}", error);
                            }

                            let sub_task_id = self.sub_tasks.insert(sub_task);
                            self.sub_task_input_ids
                                .insert(sub_task_id, widget::Id::unique());
                            self.sub_task_editing.insert(sub_task_id, false);
                            tasks.push(Output::Focus(self.sub_task_input_ids[sub_task_id].clone()));
                        }
                        Err(error) => {
                            tracing::error!("Failed to add sub-task: {:?}", error);
                        }
                    }
                }
            }
            Message::SubTaskComplete(id, complete) => {
                let task = self.sub_tasks.get_mut(id);
                if let Some(task) = task {
                    task.status = if complete {
                        Status::Completed
                    } else {
                        Status::NotStarted
                    };
                    if let Err(error) = self.storage.update_task(task) {
                        tracing::error!("Failed to update sub-task: {:?}", error);
                    }
                }
            }
            Message::SubTaskDelete(id) => {
                if let Some(task) = self.sub_tasks.remove(id) {
                    if let Err(error) = self.storage.delete_task(&task) {
                        tracing::error!("Failed to delete sub-task: {:?}", error);
                    }
                }
            }
            Message::SubTaskExpand(id) => {
                if let Some(task) = self.sub_tasks.get_mut(id) {
                    task.expanded = !task.expanded;
                    if let Err(error) = self.storage.update_task(task) {
                        tracing::error!("Failed to update sub-task: {:?}", error);
                    }
                }
            }
        }
        tasks
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let Some(ref list) = self.list else {
            return widget::container(
                widget::column::with_children(vec![
                    icons::get_icon("applications-office-symbolic", 56).into(),
                    widget::text::title1(fl!("no-list-selected")).into(),
                    widget::text(fl!("no-list-suggestion")).into(),
                ])
                .spacing(10)
                .align_x(Alignment::Center),
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
            .max_width(800.)
            .apply(widget::container)
            .height(Length::Fill)
            .width(Length::Fill)
            .center(if self.context_menu_open {
                Length::Shrink
            } else {
                Length::Fill
            })
            .padding([spacing.space_xxs, spacing.space_none])
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}
