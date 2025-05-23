use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Alignment, Length, Subscription,
    },
    iced_widget::row,
    theme, widget, Apply, Element,
};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};

use crate::app::config;
use crate::{
    app::icons,
    core::models::{self, List, Status},
    fl,
};

pub struct Content {
    list: Option<List>,
    tasks: SlotMap<DefaultKey, models::Task>,
    editing: SecondaryMap<DefaultKey, bool>,
    task_input_ids: SecondaryMap<DefaultKey, widget::Id>,
    config: config::TasksConfig,
    input: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddTask,
    Complete(DefaultKey, bool),
    Delete(DefaultKey),
    EditMode(DefaultKey, bool),
    ToggleHideCompleted,
    Input(String),
    List(Option<List>),
    Select(models::Task),
    SetItems(Vec<models::Task>),
    TitleSubmit(DefaultKey),
    TitleUpdate(DefaultKey, String),
    UpdateTask(models::Task),
    SetConfig(config::TasksConfig),
}

pub enum Task {
    Focus(widget::Id),
    Get(String),
    Display(models::Task),
    Update(models::Task),
    Delete(String),
    Create(models::Task),
    ToggleCompleted(List),
}

impl Content {
    pub fn new() -> Self {
        Self {
            list: None,
            tasks: SlotMap::new(),
            editing: SecondaryMap::new(),
            task_input_ids: SecondaryMap::new(),
            input: String::new(),
            config: config::TasksConfig::config(),
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

        let hide_completed = if list.hide_completed {
            list.hide_completed
        } else {
            self.config.hide_completed
        };

        let mut items = widget::list::list_column()
            .style(theme::Container::ContextDrawer)
            .spacing(spacing.space_xxxs)
            .padding([spacing.space_none, spacing.space_xxs]);

        for (id, item) in &self.tasks {
            if item.status == Status::Completed && hide_completed {
                continue;
            }
            let item_checkbox = widget::checkbox("", item.status == Status::Completed)
                .on_toggle(move |value| Message::Complete(id, value));

            let delete_button =
                widget::button::icon(icons::get_handle("user-trash-full-symbolic", 18))
                    .padding(spacing.space_xxs)
                    .class(cosmic::style::Button::Destructive)
                    .on_press(Message::Delete(id));

            let details_button =
                widget::button::icon(icons::get_handle("info-outline-symbolic", 18))
                    .padding(spacing.space_xxs)
                    .class(cosmic::style::Button::Standard)
                    .on_press(Message::Select(item.clone()));

            let task_item_text = widget::editable_input(
                "",
                &item.title,
                *self.editing.get(id).unwrap_or(&false),
                move |editing| Message::EditMode(id, editing),
            )
            .id(self.task_input_ids[id].clone())
            .on_submit(move |_| Message::TitleSubmit(id))
            .on_input(move |text| Message::TitleUpdate(id, text))
            .width(Length::Fill);

            let row = widget::row::with_capacity(4)
                .align_y(Alignment::Center)
                .spacing(spacing.space_xxs)
                .padding([spacing.space_xxxs, spacing.space_xxs])
                .push(item_checkbox)
                .push(task_item_text)
                .push(details_button)
                .push(delete_button);

            items = items.add(row);
        }

        widget::column::with_capacity(2)
            .spacing(spacing.space_xxs)
            .push(self.list_header(list))
            .push(items.apply(widget::scrollable))
            .apply(widget::container)
            .height(Length::Shrink)
            .height(Length::Fill)
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
                .on_input(Message::Input)
                .on_submit(|_| Message::AddTask)
                .width(Length::Fill)
                .into(),
            widget::button::icon(icons::get_handle("mail-send-symbolic", 18))
                .padding(spacing.space_xxs)
                .class(cosmic::style::Button::Suggested)
                .on_press(Message::AddTask)
                .into(),
        ])
        .padding(spacing.space_xxs)
        .spacing(spacing.space_xxs)
        .align_y(Alignment::Center)
        .apply(widget::container)
        .class(cosmic::style::Container::ContextDrawer)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Vec<Task> {
        let mut tasks = Vec::new();
        match message {
            Message::SetConfig(config) => {
                self.config = config;
            }
            Message::List(list) => {
                match (&self.list, &list) {
                    (Some(current), Some(list)) => {
                        if current.id != list.id {
                            tasks.push(Task::Get(list.id.clone()));
                        }
                    }
                    (None, Some(list)) => {
                        tasks.push(Task::Get(list.id.clone()));
                    }
                    _ => {}
                }
                self.list.clone_from(&list);
            }
            Message::TitleUpdate(id, title) => {
                if let Some(task) = self.tasks.get_mut(id) {
                    task.title = title;
                }
            }
            Message::TitleSubmit(id) => {
                if let Some(task) = self.tasks.get(id) {
                    tasks.push(Task::Update(task.clone()));
                    self.editing.insert(id, false);
                }
            }
            Message::Delete(id) => {
                if let Some(task) = self.tasks.remove(id) {
                    tasks.push(Task::Delete(task.id.clone()));
                }
            }
            Message::EditMode(id, editing) => {
                self.editing.insert(id, editing);
                if editing {
                    tasks.push(Task::Focus(self.task_input_ids[id].clone()));
                } else if let Some(task) = self.tasks.get(id) {
                    tasks.push(Task::Update(task.clone()));
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
                tasks.push(Task::Display(task));
            }
            Message::Complete(id, complete) => {
                let task = self.tasks.get_mut(id);
                if let Some(task) = task {
                    task.status = if complete {
                        Status::Completed
                    } else {
                        Status::NotStarted
                    };
                    tasks.push(Task::Update(task.clone()));
                }
            }
            Message::Input(input) => self.input = input,
            Message::AddTask => {
                if let Some(list) = &self.list {
                    if !self.input.is_empty() {
                        let task = models::Task::new(self.input.clone(), list.id.clone());
                        tasks.push(Task::Create(task.clone()));
                        let id = self.tasks.insert(task);
                        self.task_input_ids.insert(id, widget::Id::unique());
                        self.input.clear();
                    }
                }
            }
            Message::UpdateTask(updated_task) => {
                let task = self.tasks.values_mut().find(|t| t.id == updated_task.id);
                if let Some(task) = task {
                    *task = updated_task.clone();
                    tasks.push(Task::Update(task.clone()));
                }
            }
            Message::ToggleHideCompleted => {
                if let Some(ref mut list) = self.list {
                    list.hide_completed = !list.hide_completed;
                    tasks.push(Task::ToggleCompleted(list.clone()));
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
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}
