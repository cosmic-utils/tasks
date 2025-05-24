use chrono::{NaiveDate, TimeZone, Utc};
use cosmic::{
    iced::{Alignment, Length},
    iced_widget::row,
    theme,
    widget::{
        self,
        segmented_button::{self, Entity},
        text_editor,
    },
    Element,
};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};

use crate::{
    app::icons,
    core::{
        models::{self, Priority, Status},
        storage::LocalStorage,
    },
    fl,
};

pub struct Details {
    pub task: Option<models::Task>,
    pub priority_model: segmented_button::Model<segmented_button::SingleSelect>,
    pub subtask_input: String,
    pub subtasks: SlotMap<DefaultKey, models::Task>,
    pub editing: SecondaryMap<DefaultKey, bool>,
    pub sub_task_input_ids: SecondaryMap<DefaultKey, widget::Id>,
    pub text_editor_content: widget::text_editor::Content,
    pub storage: LocalStorage,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetTitle(String),
    Editor(text_editor::Action),
    Favorite(bool),
    CompleteSubTask(DefaultKey, bool),
    DeleteSubTask(DefaultKey),
    SetSubTaskTitle(DefaultKey, String),
    SubTaskEditDone,
    EditMode(DefaultKey, bool),
    PriorityActivate(Entity),
    SubTaskInput(String),
    AddSubTask,
    OpenCalendarDialog,
    SetDueDate(NaiveDate),
}

pub enum Task {
    Focus(widget::Id),
    OpenCalendarDialog,
}

impl Details {
    pub fn new(storage: LocalStorage) -> Self {
        let priority_model = segmented_button::ModelBuilder::default()
            .insert(|entity| {
                entity
                    .icon(icons::get_icon("flag-outline-thin-symbolic", 16))
                    .data(Priority::Low)
            })
            .insert(|entity| {
                entity
                    .icon(icons::get_icon("flag-outline-thick-symbolic", 16))
                    .data(Priority::Normal)
            })
            .insert(|entity| {
                entity
                    .icon(icons::get_icon("flag-filled-symbolic", 16))
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
            text_editor_content: widget::text_editor::Content::new(),
            storage,
        }
    }

    pub fn update(&mut self, message: Message) -> Vec<Task> {
        let mut tasks = vec![];
        match message {
            Message::Editor(action) => {
                if let Some(task) = &mut self.task {
                    self.text_editor_content.perform(action);
                    task.notes.clone_from(&self.text_editor_content.text());
                }
            }
            Message::SetTitle(title) => {
                if let Some(ref mut task) = &mut self.task {
                    task.title.clone_from(&title);
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
                    tasks.push(Task::Focus(self.sub_task_input_ids[id].clone()));
                } else if let Some(task) = self.subtasks.get(id) {
                    if let Err(e) = self.storage.update_task(task) {
                        tracing::error!("Failed to update sub-task: {}", e);
                    }
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
                let sub_task = self.subtasks.remove(id);
                if let Some(sub_task) = sub_task {
                    if let Err(e) = self.storage.delete_task(&sub_task) {
                        tracing::error!("Failed to delete sub-task: {}", e);
                    }
                }
            }
            Message::SubTaskEditDone => {
                tasks.push(Task::Focus(widget::Id::new("new_sub_task_input")));
            }
            Message::SubTaskInput(text) => {
                self.subtask_input = text;
            }
            Message::AddSubTask => {
                if let Some(ref mut task) = &mut self.task {
                    if !self.subtask_input.is_empty() {
                        let sub_task =
                            models::Task::new(self.subtask_input.clone(), task.sub_tasks_path());
                        if let Err(e) = self.storage.create_task(&sub_task) {
                            tracing::error!("Failed to add sub-task: {}", e);
                        }
                        let id = self.subtasks.insert(sub_task);
                        self.sub_task_input_ids.insert(id, widget::Id::unique());
                        self.subtask_input.clear();
                        tasks.push(Task::Focus(widget::Id::new("new_sub_task_input")));
                    }
                }
            }
            Message::OpenCalendarDialog => {
                tasks.push(Task::OpenCalendarDialog);
            }
            Message::SetDueDate(date) => {
                let tz = Utc::now().timezone();
                if let Some(task) = &mut self.task {
                    task.due_date = Some(tz.from_utc_datetime(&date.into()));
                }
            }
        }

        for sub_task in self.subtasks.values().cloned().collect::<Vec<_>>() {
            if let Err(e) = self.storage.update_task(&sub_task) {
                tracing::error!("Failed to update sub-task: {}", e);
            }
        }

        tasks
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        if let Some(task) = self.task.as_ref() {
            let mut sub_tasks: Vec<Element<Message>> = vec![];
            for (id, sub_task) in &self.subtasks {
                let item_checkbox = widget::checkbox("", sub_task.status == Status::Completed)
                    .on_toggle(move |value| Message::CompleteSubTask(id, value));

                let sub_task_item = widget::editable_input(
                    fl!("title"),
                    sub_task.title.clone(),
                    *self.editing.get(id).unwrap_or(&false),
                    move |editing| Message::EditMode(id, editing),
                )
                .id(self.sub_task_input_ids[id].clone())
                .on_input(move |title| Message::SetSubTaskTitle(id, title))
                .on_submit(|_| Message::SubTaskEditDone);

                let delete_button =
                    widget::button::icon(icons::get_handle("user-trash-full-symbolic", 18))
                        .padding(spacing.space_xxs)
                        .on_press(Message::DeleteSubTask(id));

                let row = widget::row::with_capacity(3)
                    .align_y(Alignment::Center)
                    .padding([spacing.space_none, spacing.space_s])
                    .spacing(spacing.space_xs)
                    .push(item_checkbox)
                    .push(sub_task_item)
                    .push(delete_button);

                sub_tasks.push(row.into());
            }

            sub_tasks.push(self.sub_task_input());

            return widget::settings::view_column(vec![
                widget::settings::section()
                    .title(fl!("details"))
                    .add(
                        widget::column::with_children(vec![
                            widget::text::body(fl!("title")).into(),
                            widget::text_input(fl!("title"), &task.title)
                                .on_input(Message::SetTitle)
                                .into(),
                        ])
                        .spacing(spacing.space_xxs)
                        .padding([0, 15, 0, 15]),
                    )
                    .add(
                        widget::settings::item::builder(fl!("favorite")).control(
                            widget::checkbox("", task.favorite).on_toggle(Message::Favorite),
                        ),
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
                            widget::text_editor(&self.text_editor_content)
                                .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
                                    text_editor_class,
                                )))
                                .padding(spacing.space_xxs)
                                .placeholder(fl!("notes"))
                                .height(100.0)
                                .on_action(Message::Editor)
                                .into(),
                        ])
                        .spacing(spacing.space_xxs)
                        .padding([0, 15, 0, 15]),
                    )
                    .into(),
                widget::settings::section()
                    .title(fl!("sub-tasks"))
                    .add(widget::column::with_children(sub_tasks).spacing(spacing.space_xs))
                    .into(),
            ])
            .into();
        }
        widget::settings::view_column(vec![widget::settings::section()
            .title(fl!("details"))
            .into()])
        .into()
    }

    fn sub_task_input(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        row(vec![
            widget::text_input(fl!("add-sub-task"), &self.subtask_input)
                .id(widget::Id::new("new_sub_task_input"))
                .on_input(Message::SubTaskInput)
                .on_submit(|_| Message::AddSubTask)
                .width(Length::Fill)
                .into(),
            widget::button::icon(icons::get_handle("mail-send-symbolic", 18))
                .padding(spacing.space_xxs)
                .on_press(Message::AddSubTask)
                .into(),
        ])
        .padding([spacing.space_none, spacing.space_s])
        .spacing(spacing.space_xs)
        .align_y(Alignment::Center)
        .into()
    }
}

fn text_editor_class(
    theme: &cosmic::Theme,
    status: cosmic::widget::text_editor::Status,
) -> cosmic::iced_widget::text_editor::Style {
    let cosmic = theme.cosmic();
    let container = theme.current_container();

    let mut background: cosmic::iced::Color = container.component.base.into();
    background.a = 0.25;
    let selection = cosmic.accent.base.into();
    let value = cosmic.palette.neutral_9.into();
    let mut placeholder = cosmic.palette.neutral_9;
    placeholder.alpha = 0.7;
    let placeholder = placeholder.into();
    let icon = cosmic.background.on.into();

    match status {
        cosmic::iced_widget::text_editor::Status::Active
        | cosmic::iced_widget::text_editor::Status::Disabled => {
            cosmic::iced_widget::text_editor::Style {
                background: background.into(),
                border: cosmic::iced::Border {
                    radius: cosmic.corner_radii.radius_m.into(),
                    width: 2.0,
                    color: container.component.divider.into(),
                },
                icon,
                placeholder,
                value,
                selection,
            }
        }
        cosmic::iced_widget::text_editor::Status::Hovered
        | cosmic::iced_widget::text_editor::Status::Focused => {
            cosmic::iced_widget::text_editor::Style {
                background: background.into(),
                border: cosmic::iced::Border {
                    radius: cosmic.corner_radii.radius_m.into(),
                    width: 2.0,
                    color: cosmic::iced::Color::from(cosmic.accent.base),
                },
                icon,
                placeholder,
                value,
                selection,
            }
        }
    }
}
