use std::collections::HashMap;

use cosmic::{
    cosmic_theme::Spacing,
    iced::{
        alignment::{Horizontal, Vertical},
        Alignment, Length,
    },
    theme,
    widget::{self, menu::Action as MenuAction},
    Apply, Element,
};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use uuid::Uuid;

use crate::{
    config, fl,
    model::{self, List, Status},
    services::store::Store,
};

/// Represents the edit state of an input field.
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
enum EditState {
    /// Input is not in edit mode.
    #[default]
    Idle,
    /// Programmatic focus was requested (e.g. a newly created sub-task).
    /// The widget has not yet self-focused via a user click, so we wait for
    /// the first `on_toggle_edit` callback before moving to `Editing`.
    Entering,
    /// The input is actively being edited.
    Editing,
}

pub struct Content {
    selected_list: Option<List>,
    tasks: SlotMap<DefaultKey, model::Task>,
    editing: SecondaryMap<DefaultKey, EditState>,
    inputs: SecondaryMap<DefaultKey, widget::Id>,
    config: config::AppConfig,
    store: Store,

    search_bar_visible: bool,
    add_task_input: String,
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
    TaskAddSubTask(DefaultKey),
    TaskComplete(DefaultKey, bool),
    TaskToggleTitleEditMode(DefaultKey, bool),
    TaskTitleInput(String),
    TaskOpenDetails(DefaultKey),
    TaskTitleSubmit(DefaultKey),
    TaskTitleUpdate(DefaultKey, String),

    ToggleHideCompleted,

    SetList(Option<List>),
    SetTasks(Vec<model::Task>),
    SetConfig(config::AppConfig),
    RefreshTask(model::Task),
    Empty,
    /// Request the deletion of a task; immediately moves the task to trash.
    OpenTaskDeletionDialog(DefaultKey),

    ToggleSearchBar,
    SearchQueryChanged(String),
    SetSort(SortType),
}

pub enum Output {
    ToggleHideCompleted(model::List),
    Focus(widget::Id),
    OpenTaskDetails(DefaultKey, Uuid),
    /// The pending deletion was committed; the app should close the details
    /// drawer if it is currently showing task details.
    TaskDeleted,
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
            TaskAction::Delete(id) => Message::OpenTaskDeletionDialog(*id),
        }
    }
}

impl Content {
    pub fn view(&self) -> Element<'_, Message> {
        let spacing = theme::active().cosmic().spacing;

        let Some(ref list) = self.selected_list else {
            return self.create_no_list_selected_view();
        };

        let mut column = widget::column(vec![self.list_view(list)]);

        column = column.push(self.new_task_view());

        column
            .max_width(800.)
            .padding([spacing.space_xxs, spacing.space_xxxs])
            .spacing(spacing.space_xxs)
            .apply(widget::container)
            .height(Length::Fill)
            .width(Length::Fill)
            .center(Length::Fill)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Option<Output> {
        let mut output = None;
        match message {
            Message::ToggleSearchBar => {
                self.search_bar_visible = !self.search_bar_visible;
                if self.search_bar_visible {
                    output = Some(Output::Focus(widget::Id::new("search-tasks-input")));
                } else {
                    self.search_query.clear();
                }
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
            }
            Message::Empty => (),
            Message::SetTasks(tasks) => {
                self.tasks.clear();
                self.inputs.clear();
                self.editing.clear();
                self.add_task_input.clear();
                self.populate_task_slotmap(tasks);
            }
            Message::SetList(list) => {
                match (&self.selected_list, &list) {
                    (Some(current), Some(list)) => {
                        if current.id != list.id {
                            match self.store.tasks(list.id).load_all() {
                                Ok(tasks) => {
                                    self.update(Message::SetTasks(tasks));
                                }
                                Err(error) => {
                                    tracing::error!("Failed to fetch tasks for list: {:?}", error)
                                }
                            }
                        }
                    }
                    (None, Some(list)) => match self.store.tasks(list.id).load_all() {
                        Ok(tasks) => {
                            self.update(Message::SetTasks(tasks));
                        }
                        Err(error) => {
                            tracing::error!("Failed to fetch tasks for list: {:?}", error)
                        }
                    },
                    _ => {}
                }
                self.selected_list.clone_from(&list);
                if list.is_some() {
                    return Some(Output::Focus(widget::Id::new("new-task-input")));
                }
            }
            Message::SetConfig(config) => {
                self.config = config;
            }
            Message::RefreshTask(refreshed_task) => {
                if let Some((id, _)) = self.tasks.iter().find(|(_, t)| t.id == refreshed_task.id) {
                    if let Some(task) = self.tasks.get_mut(id) {
                        *task = refreshed_task.clone();
                    }
                } else {
                    tracing::warn!("Task with ID {:?} not found", refreshed_task.id);
                }
            }
            Message::TaskOpenDetails(key) => match self.tasks.get(key) {
                Some(task) => output = Some(Output::OpenTaskDetails(key, task.id)),
                None => tracing::warn!("Task with ID {:?} not found", key),
            },
            Message::TaskExpand(default_key) => {
                let Some(list) = &self.selected_list else {
                    tracing::warn!("No list selected");
                    return None;
                };
                if let Some(task) = self.tasks.get_mut(default_key) {
                    task.expanded = !task.expanded;
                    if let Err(error) = self
                        .store
                        .tasks(list.id)
                        .update(task.id, |task| task.expanded = !task.expanded)
                    {
                        tracing::error!("Failed to update task: {:?}", error);
                    }
                }
            }
            Message::TaskAdd => {
                if let Some(list) = &self.selected_list {
                    if !self.add_task_input.is_empty() {
                        let task = model::Task::new(self.add_task_input.clone());
                        match self.store.tasks(list.id).save(&task) {
                            Ok(_) => {
                                let id = self.tasks.insert(task);
                                self.inputs.insert(id, widget::Id::unique());
                                self.add_task_input.clear();
                            }
                            Err(error) => {
                                tracing::error!("Failed to create task: {:?}", error);
                            }
                        }
                    }
                }
            }
            Message::TaskToggleTitleEditMode(id, editing) => {
                let Some(list) = &self.selected_list else {
                    tracing::warn!("No list selected");
                    return None;
                };

                let current_state = self.editing.get(id).copied().unwrap_or_default();

                let new_state = match (current_state, editing) {
                    // User clicked the widget – it already self-focused internally.
                    // Going straight to Editing avoids sending a redundant programmatic
                    // focus command that would reset LAST_FOCUS_UPDATE and immediately
                    // unfocus the widget, requiring a second click.
                    (EditState::Idle, true) => Some(EditState::Editing),
                    // A programmatic focus (e.g. new sub-task) was requested and the
                    // widget has now confirmed it entered edit mode.
                    (EditState::Entering, true) => Some(EditState::Editing),
                    // Sub-task lost focus before the user submitted – save and return.
                    (EditState::Entering, false) => {
                        if let Some(task) = self.tasks.get(id) {
                            if let Err(error) = self
                                .store
                                .tasks(list.id)
                                .update(task.id, |t| *t = task.clone())
                            {
                                tracing::error!("Failed to update task: {:?}", error);
                            }
                        }
                        Some(EditState::Idle)
                    }
                    // User clicked away – save and return to idle.
                    // A second on_toggle_edit(false) may fire from the widget's own
                    // focus-loss handler, but (Idle, false) is ignored below, so it
                    // is harmless.
                    (EditState::Editing, false) => {
                        if let Some(task) = self.tasks.get(id) {
                            if let Err(error) = self
                                .store
                                .tasks(list.id)
                                .update(task.id, |t| *t = task.clone())
                            {
                                tracing::error!("Failed to update task: {:?}", error);
                            }
                        }
                        Some(EditState::Idle)
                    }
                    // Already in the requested state – nothing to do.
                    (EditState::Idle, false) | (EditState::Editing, true) => None,
                };

                if let Some(state) = new_state {
                    self.editing.insert(id, state);
                }
            }
            Message::TaskTitleInput(input) => self.add_task_input = input,
            Message::TaskTitleSubmit(id) => {
                let Some(list) = &self.selected_list else {
                    tracing::warn!("No list selected");
                    return None;
                };

                if let Some(task) = self.tasks.get(id) {
                    match self
                        .store
                        .tasks(list.id)
                        .update(task.id, |t| *t = task.clone())
                    {
                        Ok(_) => {
                            self.editing.insert(id, EditState::Idle);
                            output = Some(Output::Focus(widget::Id::new("new-task-input")));
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
            Message::OpenTaskDeletionDialog(id) => {
                let Some(list) = &self.selected_list else {
                    tracing::warn!("No list selected");
                    return None;
                };
                let list_id = list.id;

                if let Some(task) = self.tasks.remove(id) {
                    self.editing.remove(id);
                    self.inputs.remove(id);

                    let trashed = crate::model::TrashedTask::new(task.clone(), list_id);
                    if let Err(err) = self.store.trash().save(&trashed) {
                        tracing::error!("Error moving task to trash: {err}");
                    }
                    if let Err(err) = self.store.tasks(list_id).delete(task.id) {
                        tracing::error!("Error removing task from list after trashing: {err}");
                    }
                    output = Some(Output::TaskDeleted);
                }
            }
            Message::TaskComplete(id, complete) => {
                let Some(list) = &self.selected_list else {
                    tracing::warn!("No list selected");
                    return None;
                };

                let task = self.tasks.get_mut(id);
                if let Some(task) = task {
                    task.status = if complete {
                        task.completion_date = Some(jiff::Timestamp::now());
                        Status::Completed
                    } else {
                        Status::NotStarted
                    };

                    if let Err(error) = self
                        .store
                        .tasks(list.id)
                        .update(task.id, |t| *t = task.clone())
                    {
                        tracing::error!("Failed to update task: {:?}", error);
                    }
                }
            }
            Message::TaskAddSubTask(id) => {
                let Some(list) = &self.selected_list else {
                    tracing::warn!("No list selected");
                    return None;
                };

                if let Some(task) = self.tasks.get_mut(id) {
                    task.expanded = true;
                    let mut sub_task = model::Task::new("".to_string());
                    sub_task.parent_id = Some(task.id);

                    match self.store.tasks(list.id).save(&sub_task) {
                        Ok(_) => {
                            // Add sub_task ID to parent's sub_task_ids
                            task.sub_task_ids.push(sub_task.id);
                            if let Err(error) = self
                                .store
                                .tasks(list.id)
                                .update(task.id, |t| *t = task.clone())
                            {
                                tracing::error!("Failed to update task with sub-task: {:?}", error);
                            }

                            // Insert subtask into the same tasks slotmap
                            let sub_task_id = self.tasks.insert(sub_task);
                            self.inputs.insert(sub_task_id, widget::Id::unique());
                            self.editing.insert(sub_task_id, EditState::Entering);
                            output = Some(Output::Focus(self.inputs[sub_task_id].clone()));
                        }
                        Err(error) => {
                            tracing::error!("Failed to add sub-task: {:?}", error);
                        }
                    }
                }
            }
            Message::ToggleHideCompleted => {
                if let Some(ref mut selected_list) = self.selected_list {
                    match self.store.lists().update(selected_list.id, |list| {
                        list.hide_completed = !selected_list.hide_completed;
                    }) {
                        Ok(updated) => {
                            selected_list.hide_completed = updated.hide_completed;
                            output = Some(Output::ToggleHideCompleted(updated.clone()));
                        }
                        Err(err) => {
                            tracing::error!("Error updating list: {err}");
                        }
                    }
                }
            }
            Message::SetSort(sort_type) => {
                self.sort_type = sort_type;
            }
        }
        output
    }
}

impl Content {
    pub fn new(storage: Store, config: config::AppConfig) -> Self {
        Self {
            selected_list: None,
            tasks: SlotMap::new(),
            editing: SecondaryMap::new(),
            inputs: SecondaryMap::new(),
            add_task_input: String::new(),
            config: config,
            store: storage,
            search_bar_visible: false,
            search_query: String::new(),
            sort_type: SortType::DateAsc,
        }
    }

    /// Creates the main list view with tasks
    pub fn list_view<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let mut column = widget::column::with_capacity(3);
        column = column.push(self.list_header(list));

        if self.search_bar_visible {
            column = column.push(self.create_search_input(&spacing));
        }

        let sorted_tasks = self.sort_tasks();
        let filtered_tasks = self.filter_and_render_tasks(list, sorted_tasks);

        if filtered_tasks.is_empty() && self.search_query.is_empty() {
            return self.empty(list);
        }

        let items = widget::column::with_children(filtered_tasks).spacing(spacing.space_s);

        widget::scrollable(
            widget::container(column.push(items).spacing(spacing.space_s)).height(Length::Shrink),
        )
        .height(Length::Fill)
        .into()
    }

    /// Creates the header row for a list with title and action buttons
    fn list_header<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let hide_completed_button = self.create_hide_completed_button(list, &spacing);
        let search_button = self.create_search_button(&spacing);
        let list_icon = self.create_list_icon(list, &spacing);

        widget::row::with_capacity(4)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(list_icon)
            .push(widget::text::body(&list.name).size(24).width(Length::Fill))
            .push(hide_completed_button)
            .push(search_button)
            .into()
    }

    /// Creates the search input field
    fn create_search_input<'a>(&'a self, spacing: &Spacing) -> Element<'a, Message> {
        widget::text_input(fl!("search-tasks"), &self.search_query)
            .id(widget::Id::new("search-tasks-input"))
            .on_input(Message::SearchQueryChanged)
            .on_unfocus(Message::ToggleSearchBar)
            .width(Length::Fill)
            .padding([spacing.space_xxs, spacing.space_xxs])
            .into()
    }

    /// Creates the hide completed tasks button
    fn create_hide_completed_button<'a>(
        &'a self,
        list: &'a List,
        spacing: &Spacing,
    ) -> Element<'a, Message> {
        let is_active = list.hide_completed || self.config.hide_completed;
        let mut button =
            widget::button::icon(widget::icon::from_name("checkbox-checked-symbolic").size(18))
                .selected(is_active)
                .padding(spacing.space_xxs);

        if is_active {
            button = button.class(cosmic::style::Button::Suggested);
        }

        if !self.config.hide_completed {
            button = button.on_press(Message::ToggleHideCompleted);
        }

        button.into()
    }

    /// Creates the search toggle button
    fn create_search_button<'a>(&'a self, spacing: &Spacing) -> Element<'a, Message> {
        widget::button::icon(widget::icon::from_name("edit-find-symbolic").size(18))
            .selected(self.search_bar_visible)
            .padding(spacing.space_xxs)
            .on_press_maybe((!self.search_bar_visible).then_some(Message::ToggleSearchBar))
            .into()
    }

    /// Creates the list icon
    fn create_list_icon<'a>(&'a self, list: &'a List, spacing: &Spacing) -> Element<'a, Message> {
        widget::icon::from_name(list.icon.as_deref().unwrap_or("view-list-symbolic"))
            .size(spacing.space_m)
            .into()
    }

    /// Sorts tasks according to the current sort type
    fn sort_tasks(&self) -> Vec<(DefaultKey, &model::Task)> {
        let mut tasks: Vec<_> = self.tasks.iter().collect();

        match self.sort_type {
            SortType::NameAsc => {
                tasks.sort_by(|a, b| a.1.title.to_lowercase().cmp(&b.1.title.to_lowercase()))
            }
            SortType::NameDesc => {
                tasks.sort_by(|a, b| b.1.title.to_lowercase().cmp(&a.1.title.to_lowercase()))
            }
            SortType::DateAsc => tasks.sort_by(|a, b| a.1.creation_date.cmp(&b.1.creation_date)),
            SortType::DateDesc => tasks.sort_by(|a, b| b.1.creation_date.cmp(&a.1.creation_date)),
        }

        tasks
    }

    /// Filters tasks and renders them as UI elements
    fn filter_and_render_tasks<'a>(
        &'a self,
        list: &'a List,
        sorted_tasks: Vec<(DefaultKey, &'a model::Task)>,
    ) -> Vec<Element<'a, Message>> {
        sorted_tasks
            .into_iter()
            .filter(|(_, task)| self.should_show_task(list, task))
            .map(|(id, task)| self.task_view(id, task))
            .collect()
    }

    /// Determines if a task should be shown based on filters
    fn should_show_task(&self, list: &List, task: &model::Task) -> bool {
        // Only show top-level tasks (no parent)
        let is_top_level = task.parent_id.is_none();

        // Check search filter
        let matches_search = !self.search_bar_visible
            || self.search_query.is_empty()
            || task
                .title
                .to_lowercase()
                .contains(&self.search_query.to_lowercase());

        // Check hide completed filter
        let should_hide_completed = list.hide_completed || self.config.hide_completed;
        let show_despite_completion = !should_hide_completed || task.status != Status::Completed;

        is_top_level && matches_search && show_despite_completion
    }

    /// Creates the view for a single task (with optional subtasks)
    pub fn task_view<'a>(&'a self, id: DefaultKey, task: &'a model::Task) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let sub_tasks = self.get_subtasks(task, self.selected_list.as_ref());
        let task_row = self.create_task_row(id, task, &sub_tasks, &spacing);

        let mut column = widget::column::with_capacity(2).push(task_row);

        if task.expanded && !sub_tasks.is_empty() {
            column = column.push(self.create_subtasks_view(&sub_tasks, &spacing));
        }

        widget::container(column)
            .class(cosmic::style::Container::ContextDrawer)
            .into()
    }

    /// Gets all direct subtasks of a task
    fn get_subtasks(
        &self,
        task: &model::Task,
        list: Option<&List>,
    ) -> Vec<(DefaultKey, &model::Task)> {
        let should_hide_completed = list
            .map(|l| l.hide_completed || self.config.hide_completed)
            .unwrap_or(false);

        self.tasks
            .iter()
            .filter(|(_, sub_task)| {
                let is_child = sub_task.parent_id == Some(task.id);
                let show_despite_completion =
                    !should_hide_completed || sub_task.status != Status::Completed;
                is_child && show_despite_completion
            })
            .collect()
    }

    /// Creates the main row for a task with all controls
    fn create_task_row<'a>(
        &'a self,
        id: DefaultKey,
        task: &'a model::Task,
        sub_tasks: &[(DefaultKey, &model::Task)],
        spacing: &Spacing,
    ) -> Element<'a, Message> {
        let checkbox = self.create_task_checkbox(id, task);
        let title_input = self.create_task_title_input(id, task);
        let expand_button = self.create_expand_button(id, task, sub_tasks, spacing);
        let subtask_count = self.create_subtask_counter(sub_tasks);
        let menu = self.create_task_menu(id);

        widget::row::with_capacity(5)
            .align_y(Alignment::Center)
            .spacing(spacing.space_xxxs)
            .padding([spacing.space_xxxs, spacing.space_xs])
            .push(checkbox)
            .push(title_input)
            .push_maybe(expand_button)
            .push_maybe(subtask_count)
            .push(menu)
            .into()
    }

    /// Creates a checkbox for marking a task as complete
    fn create_task_checkbox<'a>(
        &'a self,
        id: DefaultKey,
        task: &'a model::Task,
    ) -> Element<'a, Message> {
        widget::checkbox(task.status == Status::Completed)
            .on_toggle(move |value| Message::TaskComplete(id, value))
            .into()
    }

    /// Creates the editable title input for a task
    fn create_task_title_input<'a>(
        &'a self,
        id: DefaultKey,
        task: &'a model::Task,
    ) -> Element<'a, Message> {
        let is_editing = matches!(
            self.editing.get(id),
            Some(EditState::Entering) | Some(EditState::Editing)
        );

        widget::editable_input("", &task.title, is_editing, move |editing| {
            Message::TaskToggleTitleEditMode(id, editing)
        })
        .size(13)
        .trailing_icon(widget::column(vec![]).into())
        .id(self.inputs[id].clone())
        .on_submit(move |_| Message::TaskTitleSubmit(id))
        .on_input(move |text| Message::TaskTitleUpdate(id, text))
        .into()
    }

    /// Creates an expand/collapse button for tasks with subtasks
    fn create_expand_button<'a>(
        &'a self,
        id: DefaultKey,
        task: &'a model::Task,
        sub_tasks: &[(DefaultKey, &model::Task)],
        spacing: &Spacing,
    ) -> Option<Element<'a, Message>> {
        if sub_tasks.is_empty() {
            return None;
        }

        let icon = if task.expanded {
            "go-up-symbolic"
        } else {
            "go-down-symbolic"
        };

        Some(
            widget::button::icon(widget::icon::from_name(icon).size(18))
                .padding(spacing.space_xxs)
                .on_press(Message::TaskExpand(id))
                .into(),
        )
    }

    /// Creates a counter showing completed/total subtasks
    fn create_subtask_counter<'a>(
        &'a self,
        sub_tasks: &[(DefaultKey, &model::Task)],
    ) -> Option<Element<'a, Message>> {
        if sub_tasks.is_empty() {
            return None;
        }

        let (completed, total) = sub_tasks.iter().fold((0, 0), |acc, (_, subtask)| {
            if subtask.status == Status::Completed {
                (acc.0 + 1, acc.1 + 1)
            } else {
                (acc.0, acc.1 + 1)
            }
        });

        Some(widget::text(format!("{}/{}", completed, total)).into())
    }

    /// Creates the context menu for task actions
    fn create_task_menu<'a>(&'a self, id: DefaultKey) -> Element<'a, Message> {
        widget::menu::MenuBar::new(vec![widget::menu::Tree::with_children(
            Element::from(
                cosmic::widget::button::icon(
                    widget::icon::from_name("view-more-symbolic").size(18),
                )
                .on_press(Message::Empty),
            ),
            widget::menu::items(
                &HashMap::new(),
                vec![
                    widget::menu::Item::Button(fl!("edit"), None, TaskAction::Edit(id)),
                    widget::menu::Item::Button(
                        fl!("add-sub-task"),
                        None,
                        TaskAction::AddSubTask(id),
                    ),
                    widget::menu::Item::Button(fl!("move-to-trash"), None, TaskAction::Delete(id)),
                ],
            ),
        )])
        .item_height(widget::menu::ItemHeight::Dynamic(40))
        .item_width(widget::menu::ItemWidth::Uniform(260))
        .spacing(4.0)
        .into()
    }

    /// Creates the view for rendering subtasks
    fn create_subtasks_view<'a>(
        &'a self,
        sub_tasks: &[(DefaultKey, &'a model::Task)],
        spacing: &Spacing,
    ) -> Element<'a, Message> {
        let subtask_elements = sub_tasks
            .iter()
            .map(|(sub_id, sub_task)| {
                widget::container(self.task_view(*sub_id, sub_task))
                    .padding([0, 0, 0, spacing.space_xs])
                    .into()
            })
            .collect::<Vec<_>>();

        widget::column::with_children(subtask_elements).into()
    }

    /// Creates an empty state view when a list has no tasks
    pub fn empty<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let empty_state = self.create_empty_state_content();

        widget::column::with_capacity(2)
            .push(self.list_header(list))
            .push(empty_state)
            .padding([spacing.space_none, spacing.space_l])
            .spacing(spacing.space_s)
            .into()
    }

    /// Creates the empty state content
    fn create_empty_state_content<'a>(&'a self) -> Element<'a, Message> {
        widget::container(
            widget::column::with_children(vec![
                widget::icon::from_name("task-past-due-symbolic")
                    .size(56)
                    .into(),
                widget::text::title1(fl!("no-tasks")).into(),
                widget::text(fl!("no-tasks-suggestion")).into(),
            ])
            .spacing(10)
            .align_x(Alignment::Center),
        )
        .align_y(Vertical::Center)
        .align_x(Horizontal::Center)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
    }

    /// Creates the input field for adding new tasks
    pub fn new_task_view(&self) -> Element<'_, Message> {
        let spacing = theme::active().cosmic().spacing;

        let input = widget::text_input(fl!("add-new-task"), &self.add_task_input)
            .id(widget::Id::new("new-task-input"))
            .on_input(Message::TaskTitleInput)
            .on_submit(|_| Message::TaskAdd)
            .width(Length::Fill);

        let submit_button =
            widget::button::icon(widget::icon::from_name("mail-send-symbolic").size(18))
                .padding(spacing.space_xxs)
                .class(cosmic::style::Button::Suggested)
                .on_press(Message::TaskAdd);

        widget::row(vec![input.into(), submit_button.into()])
            .spacing(spacing.space_xxs)
            .align_y(Alignment::Center)
            .into()
    }

    fn populate_task_slotmap(&mut self, tasks: Vec<model::Task>) {
        for task in tasks {
            let task_id = self.tasks.insert(task);
            self.inputs.insert(task_id, widget::Id::unique());
            self.editing.insert(task_id, EditState::Idle);
        }
    }

    /// Creates the view shown when no list is selected
    fn create_no_list_selected_view<'a>(&'a self) -> Element<'a, Message> {
        widget::container(
            widget::column::with_children(vec![
                widget::icon::from_name("applications-office-symbolic")
                    .size(56)
                    .into(),
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
        .into()
    }
}
