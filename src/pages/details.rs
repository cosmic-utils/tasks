use chrono::{NaiveDate, TimeZone, Utc};
use cosmic::{
    iced::{Alignment, Length},
    theme,
    widget::{
        self,
        segmented_button::{self, Entity},
        text_editor,
    },
    Element,
};

use crate::{
    core::icons,
    fl,
    storage::{
        models::{self, Priority, ChecklistItem},
        LocalStorage,
    },
};

pub struct Details {
    pub task: models::Task,
    pub priority_model: segmented_button::Model<segmented_button::SingleSelect>,
    pub text_editor_content: widget::text_editor::Content,
    pub storage: LocalStorage,
    // Checklist editing state
    pub editing_checklist_item: Option<String>,
    pub new_checklist_item_text: String,
    pub editing_checklist_item_title: String, // Temporary title during editing
}

#[derive(Debug, Clone)]
pub enum Message {
    SetTitle(String),
    Editor(text_editor::Action),
    Favorite(bool),
    PriorityActivate(Entity),
    OpenCalendarDialog,
    SetDueDate(NaiveDate),
    // Checklist messages
    AddChecklistItem(String),
    ToggleChecklistItem(String),
    StartEditChecklistItem(String),
    FinishEditChecklistItem(String, String),
    CancelEditChecklistItem,
    DeleteChecklistItem(String),
    UpdateChecklistItemTitle(String, String),
    UpdateNewChecklistItemText(String),
    EmptyInputMessage(String),
}

pub enum Output {
    OpenCalendarDialog,
    RefreshTask(models::Task),
    UpdateTaskAsync(models::Task),
    // Checklist outputs
    AddChecklistItemAsync(String),
    UpdateChecklistItemAsync(String, String),
    ToggleChecklistItemAsync(String),
    DeleteChecklistItemAsync(String),
    FetchChecklistItems,
}

impl Details {
    pub fn new(storage: LocalStorage) -> Self {
        let priority_model = segmented_button::ModelBuilder::default()
            .insert(|entity| {
                entity
                    .icon(icons::get_icon("flag-outline-thin-symbolic", 14))
                    .data(Priority::Low)
            })
            .insert(|entity| {
                entity
                    .icon(icons::get_icon("flag-outline-thick-symbolic", 14))
                    .data(Priority::Normal)
            })
            .insert(|entity| {
                entity
                    .icon(icons::get_icon("flag-filled-symbolic", 14))
                    .data(Priority::High)
            })
            .build();

        Self {
            task: models::Task::default(),
            priority_model,
            text_editor_content: widget::text_editor::Content::new(),
            storage,
            editing_checklist_item: None,
            new_checklist_item_text: String::new(),
            editing_checklist_item_title: String::new(),
        }
    }

    /// Set a new task and trigger checklist fetch
    pub fn set_task(&mut self, task: models::Task) -> Vec<Output> {
        self.task = task;
        self.text_editor_content = widget::text_editor::Content::new();
        // Note: We can't directly set text in text_editor, it will be populated when the task is loaded
        
        // Reset checklist editing state
        self.editing_checklist_item = None;
        self.new_checklist_item_text.clear();
        self.editing_checklist_item_title.clear();
        
        // Trigger checklist fetch if task has an ID
        if !self.task.id.is_empty() {
            vec![Output::FetchChecklistItems]
        } else {
            vec![]
        }
    }

    pub fn update(&mut self, message: Message) -> Vec<Output> {
        let mut tasks = vec![];
        match message {
            Message::Editor(ref action) => {
                self.text_editor_content.perform(action.clone());
                self.task.notes.clone_from(&self.text_editor_content.text());
            }
            Message::SetTitle(ref title) => {
                self.task.title.clone_from(title);
            }
            Message::Favorite(favorite) => {
                self.task.favorite = favorite;
            }
            Message::PriorityActivate(entity) => {
                self.priority_model.activate(entity);
                let priority = self.priority_model.data::<Priority>(entity);
                if let Some(priority) = priority {
                    self.task.priority = *priority;
                }
            }
            Message::OpenCalendarDialog => {
                tasks.push(Output::OpenCalendarDialog);
            }
            Message::SetDueDate(date) => {
                let tz = Utc::now().timezone();
                self.task.due_date = Some(tz.from_utc_datetime(&date.into()));
            }
            // Checklist message handling
            Message::AddChecklistItem(ref title) => {
                if !title.trim().is_empty() {
                    self.new_checklist_item_text = title.clone();
                    // Output async operation instead of local update
                    tasks.push(Output::AddChecklistItemAsync(title.clone()));
                }
            }
            Message::ToggleChecklistItem(ref item_id) => {
                // Output async operation instead of local update
                tasks.push(Output::ToggleChecklistItemAsync(item_id.clone()));
            }
            Message::StartEditChecklistItem(ref item_id) => {
                // Find the current item and set its title as the editing title
                if let Some(item) = self.task.checklist_items.iter().find(|item| item.id == *item_id) {
                    self.editing_checklist_item_title = item.display_name.clone();
                }
                self.editing_checklist_item = Some(item_id.clone());
            }
            Message::FinishEditChecklistItem(ref item_id, ref _new_title) => {
                if !self.editing_checklist_item_title.trim().is_empty() {
                    // Output async operation instead of local update
                    tasks.push(Output::UpdateChecklistItemAsync(item_id.clone(), self.editing_checklist_item_title.clone()));
                }
                self.editing_checklist_item = None;
                self.editing_checklist_item_title.clear();
            }
            Message::CancelEditChecklistItem => {
                self.editing_checklist_item = None;
                self.editing_checklist_item_title.clear();
            }
            Message::DeleteChecklistItem(ref item_id) => {
                // Output async operation instead of local update
                tasks.push(Output::DeleteChecklistItemAsync(item_id.clone()));
            }
            Message::UpdateChecklistItemTitle(ref _item_id, ref new_title) => {
                // Store the title locally during editing
                self.editing_checklist_item_title = new_title.clone();
            }
            Message::UpdateNewChecklistItemText(ref text) => {
                self.new_checklist_item_text = text.clone();
            }
            Message::EmptyInputMessage(ref _text) => {
                // do nothing 
            }
        }

        // Note: Checklist operations are handled separately via async outputs
        // Only trigger task update for non-checklist operations
        let is_checklist_operation = matches!(message, 
            Message::AddChecklistItem(_) | 
            Message::ToggleChecklistItem(_) | 
            Message::FinishEditChecklistItem(_, _) | 
            Message::DeleteChecklistItem(_) | 
            Message::UpdateChecklistItemTitle(_, _) | 
            Message::EmptyInputMessage(_)
        );
        
        if !is_checklist_operation {
            tasks.push(Output::UpdateTaskAsync(self.task.clone()));
        }
        tasks
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        return widget::settings::view_column(vec![widget::settings::section()
            .title(fl!("details"))
            .add(
                widget::column::with_children(vec![
                    widget::text::body(fl!("title")).into(),
                    widget::text_input(fl!("title"), &self.task.title)
                        .style(crate::core::style::text_input())
                        .on_input(Message::SetTitle)
                        .size(13)
                        .into(),
                ])
                .padding([
                    spacing.space_s,
                    spacing.space_none,
                    spacing.space_s,
                    spacing.space_none,
                ])
                .spacing(spacing.space_xxs),
            )
            .add(
                widget::settings::item::builder(fl!("favorite"))
                    .control(widget::checkbox("", self.task.favorite).on_toggle(Message::Favorite)),
            )
            .add(
                widget::settings::item::builder(fl!("priority")).control(
                    widget::segmented_control::horizontal(&self.priority_model)
                        .button_alignment(Alignment::Center)
                        .width(Length::Shrink)
                        .style(crate::core::style::segmented_control())
                        .on_activate(Message::PriorityActivate),
                ),
            )
            .add(
                widget::settings::item::builder(fl!("due-date")).control(
                    widget::button::text(if self.task.due_date.is_some() {
                        self.task
                            .due_date
                            .as_ref()
                            .unwrap()
                            .format("%m-%d-%Y")
                            .to_string()
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
                        .class(crate::core::style::text_editor())
                        .padding(spacing.space_xxs)
                        .placeholder(fl!("add-notes"))
                        .height(100.0)
                        .size(13)
                        .on_action(Message::Editor)
                        .into(),
                ])
                .spacing(spacing.space_xxs)
                .padding([
                    spacing.space_s,
                    spacing.space_none,
                    spacing.space_s,
                    spacing.space_none,
                ]),
            )
            // Add interactive checklist display
            .add(
                widget::column::with_children(vec![
                    widget::text::body("Checklist").into(),
                    // Show completion percentage if there are items
                    if !self.task.checklist_items.is_empty() {
                        widget::text::caption(format!(
                            "{}% complete ({} of {})",
                            self.task.checklist_completion_percentage() as i32,
                            self.task.checklist_items.iter().filter(|item| item.is_checked).count(),
                            self.task.checklist_items.len()
                        )).into()
                    } else {
                        widget::text::caption("No checklist items yet").into()
                    },
                    // Add new checklist item input
                    widget::text_input("Add new checklist item...", &self.new_checklist_item_text)
                        .on_input(Message::UpdateNewChecklistItemText)
                        .on_submit(|text| Message::AddChecklistItem(text))
                        .size(13)
                        .into(),
                    // Display checklist items
                    {
                        let mut items_column = widget::column::with_capacity(self.task.checklist_items.len());
                        for item in &self.task.checklist_items {
                            items_column = items_column.push(self.view_checklist_item_interactive(item));
                        }
                        items_column.into()
                    },
                ])
                .spacing(spacing.space_xxs)
                .padding([
                    spacing.space_s,
                    spacing.space_none,
                    spacing.space_s,
                    spacing.space_none,
                ]),
            )
            .into()])
        .padding([
            spacing.space_none,
            spacing.space_s,
            spacing.space_none,
            spacing.space_s,
        ])
        .into();
    }

    fn view_checklist_item_interactive<'a>(&'a self, item: &'a ChecklistItem) -> Element<'a, Message> {
        let is_editing = self.editing_checklist_item.as_ref() == Some(&item.id);
        let spacing = theme::active().cosmic().spacing;

        if is_editing {
            // Edit mode - use existing widgets
            widget::row::with_children(vec![
                widget::text_input("Item title", &self.editing_checklist_item_title)
                    .on_input(move |text| Message::UpdateChecklistItemTitle(item.id.clone(), text))
                    .on_submit(move |text| Message::FinishEditChecklistItem(item.id.clone(), text))
                    .size(13)
                    .into(),
                widget::button::suggested("✓")
                    .on_press(Message::FinishEditChecklistItem(item.id.clone(), self.editing_checklist_item_title.clone()))
                    .into(),
                widget::button::destructive("✗")
                    .on_press(Message::CancelEditChecklistItem)
                    .into(),
            ])
            .spacing(spacing.space_xxs)
            .align_y(Alignment::Center)
            .into()
        } else {
            // View mode - use existing widgets
            widget::row::with_children(vec![
                widget::checkbox("", item.is_checked)
                    .on_toggle(move |_| Message::ToggleChecklistItem(item.id.clone()))
                    .into(),
                widget::text::body(&item.display_name)
                    .size(13)
                    .into(),
                widget::button::icon(icons::get_handle("edit-symbolic", 16))
                    .on_press(Message::StartEditChecklistItem(item.id.clone()))
                    .into(),
                widget::button::icon(icons::get_handle("edit-delete-symbolic", 16))
                    .on_press(Message::DeleteChecklistItem(item.id.clone()))
                    .into(),
            ])
            .spacing(spacing.space_xxs)
            .align_y(Alignment::Center)
            .into()
        }
    }
}
