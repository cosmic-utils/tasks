use chrono;

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

use crate::{
    core::{config, icons},
    fl,
    storage::models::{self, List, Status},
    storage::LocalStorage,
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
    search_bar_visible: bool,
    search_query: String,
    sort_type: SortType,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SortType {
    NameAsc,
    NameDesc,
    DateAsc,
    DateDesc,
}

#[derive(Debug, Clone)]
pub enum Message {
    TaskAdd,

    TaskExpand(DefaultKey),
    
    TaskComplete(DefaultKey, bool),
    TaskDelete(DefaultKey),
    TaskToggleTitleEditMode(DefaultKey, bool),
    TaskTitleInput(String),
    TaskOpenDetails(DefaultKey),
    TaskTitleSubmit(DefaultKey),
    TaskTitleUpdate(DefaultKey, String),

    

    ToggleHideCompleted,

    SetList(Option<List>),
    SetTasks(Vec<models::Task>),
    SetConfig(config::TasksConfig),
    RefreshTask(models::Task),
    Empty,
    ContextMenuOpen(bool),

    ToggleSearchBar,
    SearchQueryChanged(String),
    SetSort(SortType),
    
    // New async message variants
    TaskCreated(models::Task),           // Task was created successfully
    TaskUpdated(models::Task),          // Task was updated successfully
    TaskDeleted(models::Task),          // Task was deleted successfully
}

pub enum Output {
    ToggleHideCompleted(models::List),
    Focus(widget::Id),
    OpenTaskDetails(models::Task),
    FinishedTasksChanged,
    
    // New async output variants
    CreateTaskAsync(models::Task),
    UpdateTaskAsync(models::Task),
    DeleteTaskAsync(models::Task),
    
    // NEW: Add this to fetch tasks when list is selected
    FetchTasksAsync(models::List),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TaskAction {
    
    Edit(DefaultKey),
    Delete(DefaultKey),
}

impl MenuAction for TaskAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            TaskAction::Edit(id) => Message::TaskOpenDetails(*id),
            
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
        
            
             Message::Empty
        
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
            search_bar_visible: false,
            search_query: String::new(),
            sort_type: SortType::DateAsc,
        }
    }

    fn list_header<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let hide_completed_active = list.hide_completed || self.config.hide_completed;
        let mut hide_completed_button =
            widget::button::icon(icons::get_handle("check-round-outline-symbolic", 18))
                .selected(hide_completed_active)
                .padding(spacing.space_xxs);

        if hide_completed_active {
            hide_completed_button = hide_completed_button.class(cosmic::style::Button::Suggested);
        }

        hide_completed_button = hide_completed_button.on_press(Message::ToggleHideCompleted);

        let search_button = widget::button::icon(icons::get_handle("edit-find-symbolic", 18))
            .selected(self.search_bar_visible)
            .padding(spacing.space_xxs)
            .on_press(Message::ToggleSearchBar);

        let icon = crate::core::icons::get_icon(
            list.icon.as_deref().unwrap_or("view-list-symbolic"),
            spacing.space_m,
        );
        widget::row::with_capacity(4)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(icon)
            .push(widget::text::body(&list.name).size(24).width(Length::Fill))
            .push(hide_completed_button)
            .push(search_button)
            .into()
    }

    pub fn list_view<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let mut column = widget::column::with_capacity(3);
        column = column.push(self.list_header(list));

        if self.search_bar_visible {
            column = column.push(
                widget::text_input(fl!("search-tasks"), &self.search_query)
                    .on_input(Message::SearchQueryChanged)
                    .width(Length::Fill)
                    .padding([spacing.space_xxs, spacing.space_xxs]),
            );
        }

        let mut tasks_vec: Vec<_> = self.tasks.iter().collect();
        match self.sort_type {
            SortType::NameAsc => {
                tasks_vec.sort_by(|a, b| a.1.title.to_lowercase().cmp(&b.1.title.to_lowercase()))
            }
            SortType::NameDesc => {
                tasks_vec.sort_by(|a, b| b.1.title.to_lowercase().cmp(&a.1.title.to_lowercase()))
            }
            SortType::DateAsc => {
                tasks_vec.sort_by(|a, b| a.1.created_date_time.cmp(&b.1.created_date_time))
            }
            SortType::DateDesc => {
                tasks_vec.sort_by(|a, b| b.1.created_date_time.cmp(&a.1.created_date_time))
            }
        }

        let filtered_tasks: Vec<_> = tasks_vec
            .into_iter()
            .filter(|(_, task)| {
                // Search filter
                (!self.search_bar_visible || self.search_query.is_empty() || task.title.to_lowercase().contains(&self.search_query.to_lowercase()))
                // Hide completed filter
                && (!(list.hide_completed || self.config.hide_completed) || task.status != Status::Completed)
            })
            .map(|(id, task)| self.task_view(id, task))
            .collect();

        if filtered_tasks.is_empty() && self.search_query.is_empty() {
            return self.empty(list);
        }

        let items = widget::column::with_children(filtered_tasks).spacing(spacing.space_s);

        column
            .push(items)
            .padding([spacing.space_none, spacing.space_l])
            .spacing(spacing.space_s)
            .apply(widget::container)
            .height(Length::Shrink)
            .apply(widget::scrollable)
            .height(Length::Fill)
            .into()
    }

    pub fn task_view<'a>(&'a self, id: DefaultKey, task: &'a models::Task) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        // Checkbox for task completion
        let item_checkbox = widget::checkbox("", task.status == Status::Completed)
            .on_toggle(move |value| Message::TaskComplete(id, value));

        // Priority flag icon
        // let priority_icon = match task.priority {
        //     models::Priority::Low => icons::get_icon("flag-outline-thin-symbolic", 16),
        //     models::Priority::Normal => icons::get_icon("flag-outline-thick-symbolic", 16),
        //     models::Priority::High => icons::get_icon("flag-filled-symbolic", 16),
        // };
        

        // Due date with color coding
        let due_date_widget: Element<Message> = if let Some(due_date) = task.due_date {
            let now = chrono::Utc::now();
            let due_date_naive = due_date.naive_utc().date();
            let today = now.naive_utc().date();
            
            let theme = theme::active();
            let (color, text) = if due_date_naive < today {
                (theme.cosmic().destructive_color().into(), due_date_naive.format("%b %d").to_string())
            } else if due_date_naive == today {
                (theme.cosmic().warning_color().into(), due_date_naive.format("%b %d").to_string())
            } else {
                (theme.cosmic().palette.neutral_9.into(), due_date_naive.format("%b %d").to_string())
            };

            let calendar_icon = icons::get_icon("office-calendar-symbolic", 14);
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(calendar_icon)
                .push(widget::text(text).class(cosmic::style::Text::Color(color)))
                .into()
        } else {
            widget::text::text("").into()
        };

        // Reminder date with color coding
        let reminder_widget: Element<Message> = if let Some(reminder_date) = task.reminder_date {
            let now = chrono::Utc::now();
            let reminder_naive = reminder_date.naive_utc().date();
            let today = now.naive_utc().date();
            
            let theme = theme::active();
            let (color, text) = if reminder_naive < today {
                (theme.cosmic().destructive_color().into(), reminder_naive.format("%b %d").to_string())
            } else if reminder_naive == today {
                (theme.cosmic().warning_color().into(), reminder_naive.format("%b %d").to_string())
            } else {
                (theme.cosmic().palette.neutral_9.into(), reminder_naive.format("%b %d").to_string())
            };

            let alarm_icon = icons::get_icon("alarm-symbolic", 14);
            widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(alarm_icon)
                .push(widget::text(text).class(cosmic::style::Text::Color(color)))
                .into()
        } else {
            widget::text::text("").into()
        };

        // Notes (truncated to 255 chars)
        let notes_widget: Element<Message> = if !task.notes.is_empty() {
            let truncated_notes = if task.notes.len() > 255 {
                format!("{}...", &task.notes[..255])
            } else {
                task.notes.clone()
            };
            widget::text(truncated_notes)
                .size(12)
                .class(cosmic::style::Text::Color(theme::active().cosmic().palette.neutral_6.into()))
                .into()
        } else {
            widget::text::text("").into()
        };

        // Edit and Delete buttons
        let edit_button = widget::button::icon(icons::get_handle("edit-symbolic", 16))
            .padding(spacing.space_xxs)
            .on_press(Message::TaskOpenDetails(id));

        let delete_button = widget::button::icon(icons::get_handle("user-trash-symbolic", 16))
            .padding(spacing.space_xxs)
            .on_press(Message::TaskDelete(id));

        // Task title input
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

        // Main row with checkbox, priority, title, and action buttons
        let main_row = widget::row::with_capacity(5)
            .align_y(Alignment::Center)
            .spacing(spacing.space_xxs)
            .padding([spacing.space_xxs, spacing.space_s])
            .push(item_checkbox)
            
            .push(task_item_text)
            .push(edit_button)
            .push(delete_button);

        // Info row with dates and notes
        let info_row = widget::row::with_capacity(3)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_s])
            
            .push(due_date_widget)
            .push(reminder_widget)
            .push(widget::Space::new(Length::Fill, Length::Fixed(4 as f32))); // Spacer

        // Notes row
        let notes_row: Element<Message> = if !task.notes.is_empty() {
            widget::row::with_capacity(1)
                .align_y(Alignment::Center)
                .padding([spacing.space_none, spacing.space_s])
                .push(notes_widget)
                .into()
        } else {
            widget::text::text("").into()
        };

        // Main column containing all rows
        let mut column = widget::column::with_capacity(3)
            .push(main_row)
            .push(info_row);

        if !task.notes.is_empty() {
            column = column.push(notes_row);
        }

        column
            .padding(spacing.space_xxs)
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
            .push(self.list_header(list))
            .push(container)
            .padding([spacing.space_none, spacing.space_l])
            .spacing(spacing.space_s)
            .into()
    }

    pub fn new_task_view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;
        row(vec![
            widget::text_input(fl!("add-new-task"), &self.input)
                .id(widget::Id::new("new-task-input"))
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
            
        }
    }

    fn populate_sub_task_slotmap(&mut self, tasks: Vec<models::Task>) {
        for task in tasks {
            let task_id = self.sub_tasks.insert(task.clone());
            self.sub_task_input_ids
                .insert(task_id, widget::Id::unique());
            self.sub_task_editing.insert(task_id, false);
            
        }
    }

    pub fn update(&mut self, message: Message) -> Vec<Output> {
        let mut tasks = Vec::new();
        match message {
            Message::ToggleSearchBar => {
                self.search_bar_visible = !self.search_bar_visible;
                if !self.search_bar_visible {
                    self.search_query.clear();
                }
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
            }
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
                            // Clear current tasks and fetch new ones
                            self.tasks.clear();
                            tasks.push(Output::FetchTasksAsync(list.clone()));
                        }
                    }
                    (None, Some(list)) => {
                        // First time selecting a list, fetch its tasks
                        self.tasks.clear();
                        tasks.push(Output::FetchTasksAsync(list.clone()));
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
                    // Emit async output instead of calling storage directly
                    tasks.push(Output::UpdateTaskAsync(task.clone()));
                }
            }
            Message::TaskAdd => {
                if let Some(list) = &self.list {
                    if !self.input.is_empty() {
                        let task = models::Task::new(self.input.clone(), Some(list.id.clone()));
                        // Emit async output instead of calling storage directly
                        tasks.push(Output::CreateTaskAsync(task));
                        self.input.clear();
                    }
                }
            }
            Message::TaskToggleTitleEditMode(id, editing) => {
                self.task_editing.insert(id, editing);
                if editing {
                    tasks.push(Output::Focus(self.task_input_ids[id].clone()));
                } else if let Some(task) = self.tasks.get(id) {
                    // Emit async output instead of calling storage directly
                    tasks.push(Output::UpdateTaskAsync(task.clone()));
                }
            }
            Message::TaskTitleInput(input) => self.input = input,
            Message::TaskTitleSubmit(id) => {
                if let Some(task) = self.tasks.get(id) {
                    // Emit async output instead of calling storage directly
                    tasks.push(Output::UpdateTaskAsync(task.clone()));
                    self.task_editing.insert(id, false);
                    tasks.push(Output::Focus(widget::Id::new("new-task-input")));
                }
            }
            Message::TaskTitleUpdate(id, title) => {
                if let Some(task) = self.tasks.get_mut(id) {
                    task.title = title;
                }
            }
            Message::TaskDelete(id) => {
                if let Some(task) = self.tasks.remove(id) {
                    // Emit async output instead of calling storage directly
                    tasks.push(Output::DeleteTaskAsync(task));
                    tasks.push(Output::FinishedTasksChanged);
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
                    tasks.push(Output::FinishedTasksChanged);
                    // Emit async output instead of calling storage directly
                    tasks.push(Output::UpdateTaskAsync(task.clone()));
                }
            }
            
            Message::ToggleHideCompleted => {
                if let Some(ref mut list) = self.list {
                    list.hide_completed = !list.hide_completed;
                    tasks.push(Output::ToggleHideCompleted(list.clone()));
                }
            }
            
            Message::SetSort(sort_type) => {
                self.sort_type = sort_type;
            }
            
            // New async message handlers
            Message::TaskCreated(task) => {
                let id = self.tasks.insert(task);
                self.task_input_ids.insert(id, widget::Id::unique());
                tasks.push(Output::FinishedTasksChanged);
            }
            Message::TaskUpdated(task) => {
                // Task was updated successfully, no action needed
                tracing::info!("Task updated successfully: {}", task.title);
            }
            Message::TaskDeleted(task) => {
                // Task was deleted successfully, no action needed
                tracing::info!("Task deleted successfully: {}", task.title);
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
