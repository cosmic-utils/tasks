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
    subtask_editing: SecondaryMap<DefaultKey, bool>,
    task_input_ids: SecondaryMap<DefaultKey, widget::Id>,
    subtask_input_ids: SecondaryMap<DefaultKey, widget::Id>,
    config: config::TasksConfig,
    input: String,
    storage: LocalStorage,
}

#[derive(Debug, Clone)]
pub enum Message {
    // New Task Actions
    TaskAdd,

    // Task Actions
    TaskExpand(DefaultKey),
    TaskComplete(DefaultKey, bool),
    TaskToggleTitleEditMode(DefaultKey, bool),
    TaskTitleInput(String),
    TaskOpenDetails(DefaultKey),
    TaskTitleSubmit(DefaultKey),
    TaskTitleUpdate(DefaultKey, String),
    TaskDelete(DefaultKey),

    // Sub-Task Actions
    SubTaskToggleTitleEditMode(DefaultKey, DefaultKey, bool),
    SubTaskTitleSubmit(DefaultKey, DefaultKey),
    SubTaskTitleUpdate(DefaultKey, DefaultKey, String),
    SubTaskOpenDetails(DefaultKey),
    SubTaskAdd(DefaultKey),
    SubTaskComplete(DefaultKey, DefaultKey, bool),
    SubTaskDelete(DefaultKey, DefaultKey),

    // Header Actions
    ToggleHideCompleted,

    // Input Actions
    SetList(Option<List>),
    SetTasks(Vec<models::Task>),
    SetConfig(config::TasksConfig),
}

pub enum Output {
    ToggleHideCompleted(models::List),
    Focus(widget::Id),
    OpenTaskDetails(models::Task),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TaskAction {
    AddSubTask(DefaultKey),
    /// Task key ID
    Edit(DefaultKey),
    /// Task key ID + optional sub-task key ID
    Delete(DefaultKey),
}

impl MenuAction for TaskAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            TaskAction::Edit(id) => Message::TaskOpenDetails(id.clone()),
            TaskAction::AddSubTask(id) => Message::SubTaskAdd(id.clone()),
            TaskAction::Delete(id) => Message::TaskDelete(id.clone()),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SubTaskAction {
    AddSubTask(DefaultKey),
    /// Task key ID
    Edit(DefaultKey),
    /// Task key ID + optional sub-task key ID
    Delete(DefaultKey, DefaultKey),
}

impl MenuAction for SubTaskAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            SubTaskAction::Edit(id) => Message::SubTaskOpenDetails(id.clone()),
            SubTaskAction::AddSubTask(id) => Message::SubTaskAdd(id.clone()),
            SubTaskAction::Delete(id, sub_id) => Message::SubTaskDelete(id.clone(), sub_id.clone()),
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
            subtask_editing: SecondaryMap::new(),
            task_input_ids: SecondaryMap::new(),
            subtask_input_ids: SecondaryMap::new(),
            input: String::new(),
            config: config::TasksConfig::config(),
            storage,
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
            .map(|(id, task)| {
                widget::column()
                    .push(self.task_view(id, task))
                    .push_maybe(self.sub_task_view(id, task.expanded))
                    .apply(widget::container)
                    .class(cosmic::style::Container::ContextDrawer)
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

    pub fn task_view<'a>(&'a self, id: DefaultKey, task: &'a models::Task) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let sub_tasks: Vec<&models::Task> = self
            .sub_tasks
            .iter()
            .filter(|(_, sub_task)| sub_task.parent == task.id)
            .map(|(_, sub_task)| sub_task)
            .collect();

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
                .class(theme::Button::Text)
                .on_press(Message::TaskExpand(id))
        });

        let more_button = widget::menu::MenuBar::new(vec![widget::menu::Tree::with_children(
            cosmic::widget::button::icon(icons::get_handle("view-more-symbolic", 18)),
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

        let subtask_count =
            widget::text(format!("{}/{}", completed, total)).class(cosmic::style::Text::Accent);

        let task_item_text = widget::editable_input(
            "",
            &task.title,
            *self.task_editing.get(id).unwrap_or(&false),
            move |editing| Message::TaskToggleTitleEditMode(id, editing),
        )
        .trailing_icon(widget::column().into())
        .id(self.task_input_ids[id].clone())
        .on_submit(move |_| Message::TaskTitleSubmit(id))
        .on_input(move |text| Message::TaskTitleUpdate(id, text));

        widget::row::with_capacity(4)
            .align_y(Alignment::Center)
            .spacing(spacing.space_xxxs)
            .padding([spacing.space_xxs, spacing.space_s])
            .push(item_checkbox)
            .push(task_item_text)
            .push(subtask_count)
            .push_maybe(expand_button)
            .push(more_button)
            .into()
    }

    pub fn sub_task_view<'a>(
        &'a self,
        parent_id: DefaultKey,
        expanded: bool,
    ) -> Option<Element<'a, Message>> {
        let spacing = theme::active().cosmic().spacing;

        if expanded {
            let sub_tasks: Vec<Element<'a, Message>> = self
                .sub_tasks
                .iter()
                .filter(|(_, sub_task)| {
                    sub_task.parent
                        == self
                            .tasks
                            .get(parent_id)
                            .map(|t| t.id.clone())
                            .unwrap_or_default()
                })
                .map(|(sub_id, sub_task)| {
                    let more_button =
                        widget::menu::MenuBar::new(vec![widget::menu::Tree::with_children(
                            cosmic::widget::button::icon(icons::get_handle(
                                "view-more-symbolic",
                                18,
                            )),
                            widget::menu::items(
                                &HashMap::new(),
                                vec![
                                    widget::menu::Item::Button(
                                        fl!("edit"),
                                        None,
                                        SubTaskAction::Edit(sub_id),
                                    ),
                                    widget::menu::Item::Button(
                                        fl!("add-sub-task"),
                                        None,
                                        SubTaskAction::AddSubTask(sub_id),
                                    ),
                                    widget::menu::Item::Button(
                                        fl!("delete"),
                                        None,
                                        SubTaskAction::Delete(parent_id, sub_id),
                                    ),
                                ],
                            ),
                        )])
                        .item_height(widget::menu::ItemHeight::Dynamic(40))
                        .item_width(widget::menu::ItemWidth::Uniform(260))
                        .spacing(4.0);

                    let subtask_checkbox =
                        widget::checkbox("", sub_task.status == Status::Completed).on_toggle(
                            move |value| Message::SubTaskComplete(parent_id, sub_id, value),
                        );

                    let subtask_text = widget::editable_input(
                        "",
                        &sub_task.title,
                        *self.subtask_editing.get(sub_id).unwrap_or(&false),
                        move |editing| {
                            Message::SubTaskToggleTitleEditMode(parent_id, sub_id, editing)
                        },
                    )
                    .trailing_icon(widget::column().into())
                    .id(self.subtask_input_ids[sub_id].clone())
                    .on_submit(move |_| Message::SubTaskTitleSubmit(parent_id, sub_id))
                    .on_input(move |text| Message::SubTaskTitleUpdate(parent_id, sub_id, text));

                    widget::row()
                        .align_y(Alignment::Center)
                        .spacing(spacing.space_xxxs)
                        .padding([
                            spacing.space_none,
                            spacing.space_xxxs,
                            spacing.space_none,
                            spacing.space_l,
                        ])
                        .push(subtask_checkbox)
                        .push(subtask_text)
                        .push(more_button)
                        .into()
                })
                .collect();
            if !sub_tasks.is_empty() {
                Some(
                    widget::column::with_children(sub_tasks)
                        .spacing(spacing.space_xxs)
                        .padding(spacing.space_xs)
                        .into(),
                )
            } else {
                None
            }
        } else {
            None
        }
    }

    // pub fn list_view<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
    //     let spacing = theme::active().cosmic().spacing;

    //     if self.tasks.is_empty() {
    //         return self.empty(list);
    //     }

    //     let hide_completed = if list.hide_completed {
    //         list.hide_completed
    //     } else {
    //         self.config.hide_completed
    //     };

    //     let mut items = widget::list::list_column()
    //         .style(theme::Container::ContextDrawer)
    //         .spacing(spacing.space_xxxs)
    //         .padding([spacing.space_none, spacing.space_xxs]);

    //     for (id, item) in &self.tasks {
    //         if item.status == Status::Completed && hide_completed {
    //             continue;
    //         }
    //         let item_checkbox = widget::checkbox("", item.status == Status::Completed)
    //             .on_toggle(move |value| Message::Complete(id, value));

    //         let delete_button =
    //             widget::button::icon(icons::get_handle("user-trash-full-symbolic", 18))
    //                 .padding(spacing.space_xxs)
    //                 .class(cosmic::style::Button::Destructive)
    //                 .on_press(Message::Delete(id));

    //         let details_button =
    //             widget::button::icon(icons::get_handle("info-outline-symbolic", 18))
    //                 .padding(spacing.space_xxs)
    //                 .class(cosmic::style::Button::Standard)
    //                 .on_press(Message::Select(item.clone()));

    //         let task_item_text = widget::editable_input(
    //             "",
    //             &item.title,
    //             *self.editing.get(id).unwrap_or(&false),
    //             move |editing| Message::EditMode(id, editing),
    //         )
    //         .id(self.task_input_ids[id].clone())
    //         .on_submit(move |_| Message::TitleSubmit(id))
    //         .on_input(move |text| Message::TitleUpdate(id, text))
    //         .width(Length::Fill);

    //         let row = widget::row::with_capacity(4)
    //             .align_y(Alignment::Center)
    //             .spacing(spacing.space_xxs)
    //             .padding([spacing.space_xxxs, spacing.space_xxs])
    //             .push(item_checkbox)
    //             .push(task_item_text)
    //             .push(details_button)
    //             .push(delete_button);

    //         items = items.add(row);
    //     }

    //     widget::column::with_capacity(2)
    //         .spacing(spacing.space_xxs)
    //         .push(self.list_header(list))
    //         .push(items.apply(widget::scrollable))
    //         .apply(widget::container)
    //         .height(Length::Shrink)
    //         .height(Length::Fill)
    //         .into()
    // }

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

    pub fn update(&mut self, message: Message) -> Vec<Output> {
        let mut tasks = Vec::new();
        match message {
            Message::SetTasks(tasks_vec) => {
                self.tasks.clear();
                self.task_input_ids.clear();
                self.task_editing.clear();
                self.sub_tasks.clear();
                self.subtask_input_ids.clear();
                self.subtask_editing.clear();
                self.input.clear();
                for task in tasks_vec {
                    let id = self.tasks.insert(task.clone());
                    self.task_input_ids.insert(id, widget::Id::unique());
                    self.task_editing.insert(id, false);
                    if let Ok(sub_tasks) = self.storage.get_sub_tasks(&task.parent, &task.id) {
                        for subtask in sub_tasks {
                            let sub_task_id = self.sub_tasks.insert(subtask);
                            self.subtask_input_ids
                                .insert(sub_task_id, widget::Id::unique());
                            self.subtask_editing.insert(sub_task_id, false);
                        }
                    }
                }
            }
            Message::SetList(list) => {
                match (&self.list, &list) {
                    (Some(current), Some(list)) => {
                        if current.id != list.id {
                            match self.storage.tasks(&list.id) {
                                Ok(tasks) => {
                                    self.update(Message::SetTasks(tasks));
                                }
                                Err(error) => {
                                    tracing::error!("Failed to fetch tasks for list: {:?}", error)
                                }
                            }
                        }
                    }
                    (None, Some(list)) => match self.storage.tasks(&list.id) {
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
            Message::TaskOpenDetails(task) => match self.tasks.get(task) {
                Some(task) => tasks.push(Output::OpenTaskDetails(task.clone())),
                None => tracing::warn!("Task with ID {:?} not found", task),
            },
            Message::TaskExpand(default_key) => {
                if let Some(task) = self.tasks.get_mut(default_key) {
                    task.expanded = !task.expanded;
                    if let Err(error) = self.storage.update_task(task.clone()) {
                        tracing::error!("Failed to update task: {:?}", error);
                    }
                }
            }
            Message::TaskAdd => {
                if let Some(list) = &self.list {
                    if !self.input.is_empty() {
                        let task = models::Task::new(self.input.clone(), list.id.clone());
                        match self.storage.create_task(task.clone()) {
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
                    if let Err(error) = self.storage.update_task(task.clone()) {
                        tracing::error!("Failed to update task: {:?}", error);
                    }
                }
            }
            Message::TaskTitleInput(input) => self.input = input,
            Message::TaskTitleSubmit(id) => {
                if let Some(task) = self.tasks.get(id) {
                    match self.storage.update_task(task.clone()) {
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
                    if let Err(error) = self.storage.delete_task(&task.parent, &task.id) {
                        tracing::error!("Failed to delete task: {:?}", error);
                    }
                    // Remove all sub_tasks with this parent
                    let parent_id = &task.id;
                    let sub_keys: Vec<_> = self
                        .sub_tasks
                        .iter()
                        .filter(|(_, sub)| &sub.parent == parent_id)
                        .map(|(k, _)| k)
                        .collect();
                    for sub_id in sub_keys {
                        self.sub_tasks.remove(sub_id);
                        self.subtask_input_ids.remove(sub_id);
                        self.subtask_editing.remove(sub_id);
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
                    if let Err(error) = self.storage.update_task(task.clone()) {
                        tracing::error!("Failed to update task: {:?}", error);
                    }
                }
            }
            Message::SubTaskToggleTitleEditMode(parent_key, subtask_key, editing) => {
                self.subtask_editing.insert(subtask_key, editing);
                if editing {
                    tasks.push(Output::Focus(self.subtask_input_ids[subtask_key].clone()));
                } else if let Some(task) = self.tasks.get(parent_key) {
                    if let Some(sub_task) = self.sub_tasks.get(subtask_key) {
                        if let Err(error) =
                            self.storage.update_sub_task(&task.parent, sub_task.clone())
                        {
                            tracing::error!("Failed to update sub-task: {:?}", error);
                        }
                    }
                }
            }
            Message::SubTaskTitleSubmit(parent_key, subtask_key) => {
                if let Some(task) = self.tasks.get(parent_key) {
                    if let Some(sub_task) = self.sub_tasks.get(subtask_key) {
                        if let Err(error) =
                            self.storage.update_sub_task(&task.parent, sub_task.clone())
                        {
                            tracing::error!("Failed to update sub-task: {:?}", error);
                        }
                        self.subtask_editing.insert(subtask_key, false);
                    }
                }
            }
            Message::SubTaskTitleUpdate(parent_key, subtask_key, title) => {
                if let Some(task) = self.tasks.get(parent_key) {
                    if let Some(sub_task) = self.sub_tasks.get_mut(subtask_key) {
                        sub_task.title = title;
                        if let Err(error) =
                            self.storage.update_sub_task(&task.parent, sub_task.clone())
                        {
                            tracing::error!("Failed to update sub-task: {:?}", error);
                        }
                    }
                }
            }
            Message::SubTaskAdd(parent_key) => {
                if let Some(task) = self.tasks.get_mut(parent_key) {
                    task.expanded = true;
                    if let Err(error) = self.storage.update_task(task.clone()) {
                        tracing::error!("Failed to update task: {:?}", error);
                    }
                    let sub_task = models::Task::new("".to_string(), task.id.clone());
                    match self.storage.add_sub_task(&task.parent, sub_task.clone()) {
                        Ok(sub_task) => {
                            let sub_task_id = self.sub_tasks.insert(sub_task);
                            self.subtask_input_ids
                                .insert(sub_task_id, widget::Id::unique());
                            self.subtask_editing.insert(sub_task_id, false);
                            tasks.push(Output::Focus(self.subtask_input_ids[sub_task_id].clone()));
                        }
                        Err(error) => {
                            tracing::error!("Failed to add sub-task: {:?}", error);
                        }
                    }
                }
            }
            Message::SubTaskComplete(parent_key, subtask_key, complete) => {
                if let Some(task) = self.tasks.get(parent_key) {
                    if let Some(sub_task) = self.sub_tasks.get_mut(subtask_key) {
                        sub_task.status = if complete {
                            Status::Completed
                        } else {
                            Status::NotStarted
                        };
                        if let Err(error) =
                            self.storage.update_sub_task(&task.parent, sub_task.clone())
                        {
                            tracing::error!("Failed to update sub-task: {:?}", error);
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
            Message::SubTaskOpenDetails(sub_id) => {
                if let Some(sub_task) = self.sub_tasks.get(sub_id) {
                    tasks.push(Output::OpenTaskDetails(sub_task.clone()));
                }
            }
            Message::SubTaskDelete(parent_id, sub_id) => {
                if let Some(sub_task) = self.sub_tasks.remove(sub_id) {
                    if let Some(task) = self.tasks.get(parent_id) {
                        if let Err(error) =
                            self.storage
                                .delete_sub_task(&task.parent, &task.id, &sub_task.id)
                        {
                            tracing::error!("Failed to delete sub-task: {:?}", error);
                        }
                    }
                    self.subtask_input_ids.remove(sub_id);
                    self.subtask_editing.remove(sub_id);
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
            .center(Length::Fill)
            .padding([spacing.space_xxs, spacing.space_none])
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}
