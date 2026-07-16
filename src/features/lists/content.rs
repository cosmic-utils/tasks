use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use cosmic::{
    cosmic_theme::Spacing,
    iced::{
        alignment::{Horizontal, Vertical},
        clipboard::mime::{AllowedMimeTypes, AsMimeTypes},
        Alignment, Length,
    },
    theme,
    widget::{self, menu::Action as MenuAction},
    Apply, Element,
};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use uuid::Uuid;

use crate::{
    config,
    features::{
        lists::list::List,
        tasks::{
            state::TaskState,
            task::{Task, TrashedTask},
        },
    },
    fl,
    shared::{store::Store, widgets::collapsible_section},
};

const TASK_DRAG_MIME: &str = "application/x-cosmic-tasks-item";

#[derive(Debug, Clone)]
struct TaskDrag {
    uuid: Uuid,
}

impl AsMimeTypes for TaskDrag {
    fn available(&self) -> Cow<'static, [String]> {
        Cow::Owned(vec![TASK_DRAG_MIME.to_string()])
    }

    fn as_bytes(&self, mime_type: &str) -> Option<Cow<'static, [u8]>> {
        if mime_type == TASK_DRAG_MIME {
            Some(Cow::Owned(self.uuid.as_bytes().to_vec()))
        } else {
            None
        }
    }
}

impl TryFrom<(Vec<u8>, String)> for TaskDrag {
    type Error = ();

    fn try_from((data, mime): (Vec<u8>, String)) -> Result<Self, Self::Error> {
        if mime == TASK_DRAG_MIME {
            let arr: [u8; 16] = data.try_into().map_err(|_| ())?;
            Ok(Self {
                uuid: Uuid::from_bytes(arr),
            })
        } else {
            Err(())
        }
    }
}

impl AllowedMimeTypes for TaskDrag {
    fn allowed() -> Cow<'static, [String]> {
        Cow::Owned(vec![TASK_DRAG_MIME.to_string()])
    }
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
enum EditState {
    #[default]
    Idle,
    Entering,
    Editing,
}

pub struct Content {
    selected_list: Option<List>,
    tasks: SlotMap<DefaultKey, Task>,
    editing: SecondaryMap<DefaultKey, EditState>,
    inputs: SecondaryMap<DefaultKey, widget::Id>,
    config: config::AppConfig,
    store: Store,
    states: Vec<TaskState>,
    collapsed_sections: HashSet<Uuid>,

    search_bar_visible: bool,
    add_task_input: String,
    search_query: String,
    drag_hover: Option<DefaultKey>,
}

pub use crate::config::SortBy;

#[derive(Debug, Clone)]
pub enum Message {
    TaskAdd,

    TaskExpand(DefaultKey),
    TaskAddSubTask(DefaultKey),
    TaskComplete(DefaultKey, bool),
    TaskToggleFavorite(DefaultKey),
    TaskToggleTitleEditMode(DefaultKey, bool),
    TaskTitleInput(String),
    TaskOpenDetails(DefaultKey),
    TaskTitleSubmit(DefaultKey),
    TaskTitleUpdate(DefaultKey, String),

    SetList(Option<List>),
    SetTasks(Vec<Task>),
    SyncTasks(Vec<Task>),
    SetConfig(config::AppConfig),
    RefreshTask(Task),
    Empty,
    OpenTaskDeletionDialog(DefaultKey),
    RestoreTask(Uuid, Uuid),

    ToggleSearchBar,
    SearchQueryChanged(String),
    SetSort(SortBy),
    DragStarted(DefaultKey),
    DragEntered(DefaultKey),
    DragLeft,
    TaskDropped {
        from: Option<Uuid>,
        onto: DefaultKey,
    },
    ToggleSection(Uuid),
}

pub enum Output {
    Focus(widget::Id),
    OpenTaskDetails(DefaultKey, Uuid),
    TaskDeleted {
        task_id: Uuid,
        list_id: Uuid,
        title: String,
    },
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
            Message::SyncTasks(tasks) => {
                self.reconcile_tasks(tasks);
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
                    let title = self.add_task_input.trim();
                    if !title.is_empty() {
                        let task = Task::new(title);
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

                    let trashed = TrashedTask::new(task.clone(), list_id);
                    if let Err(err) = self.store.trash().save(&trashed) {
                        tracing::error!("Error moving task to trash: {err}");
                    }
                    if let Err(err) = self.store.tasks(list_id).delete(task.id) {
                        tracing::error!("Error removing task from list after trashing: {err}");
                    }
                    output = Some(Output::TaskDeleted {
                        task_id: task.id,
                        list_id,
                        title: task.title.clone(),
                    });
                }
            }
            Message::RestoreTask(task_id, list_id) => {
                let trashed = self.store.trash().load_all().unwrap_or_else(|err| {
                    tracing::error!("Failed to load trash for restore: {err}");
                    Vec::new()
                });
                if let Some(trashed) = trashed.into_iter().find(|t| t.task.id == task_id) {
                    if let Err(err) = self
                        .store
                        .tasks(trashed.original_list_id)
                        .save(&trashed.task)
                    {
                        tracing::error!("Error restoring task from trash: {err}");
                    } else if let Err(err) = self.store.trash().delete(task_id) {
                        tracing::error!("Error removing restored task from trash: {err}");
                    } else if self
                        .selected_list
                        .as_ref()
                        .is_some_and(|list| list.id == list_id)
                    {
                        if let Ok(tasks) = self.store.tasks(list_id).load_all() {
                            self.update(Message::SetTasks(tasks));
                        }
                    }
                }
            }
            Message::TaskComplete(id, complete) => {
                let Some(list) = &self.selected_list else {
                    tracing::warn!("No list selected");
                    return None;
                };

                let task = self.tasks.get_mut(id);
                if let Some(task) = task {
                    task.completion_date = if complete {
                        Some(jiff::Timestamp::now())
                    } else {
                        None
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
            Message::TaskToggleFavorite(id) => {
                let Some(list) = &self.selected_list else {
                    tracing::warn!("No list selected");
                    return None;
                };

                if let Some(task) = self.tasks.get_mut(id) {
                    task.favorite = !task.favorite;
                    if let Err(error) = self
                        .store
                        .tasks(list.id)
                        .update(task.id, |t| t.favorite = task.favorite)
                    {
                        tracing::error!("Failed to update task favorite: {:?}", error);
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
                    let mut sub_task = Task::new("".to_string());
                    sub_task.parent_id = Some(task.id);

                    match self.store.tasks(list.id).save(&sub_task) {
                        Ok(_) => {
                            task.sub_task_ids.push(sub_task.id);
                            if let Err(error) = self
                                .store
                                .tasks(list.id)
                                .update(task.id, |t| *t = task.clone())
                            {
                                tracing::error!("Failed to update task with sub-task: {:?}", error);
                            }

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
            Message::SetSort(sort_by) => {
                self.config.sort_by = sort_by;
            }
            Message::DragStarted(_id) => {
                self.drag_hover = None;
            }
            Message::DragEntered(id) => {
                self.drag_hover = Some(id);
            }
            Message::DragLeft => {
                self.drag_hover = None;
            }
            Message::TaskDropped { from, onto } => {
                self.drag_hover = None;
                if let Some(from_uuid) = from {
                    self.reorder_tasks(from_uuid, onto);
                }
            }
            Message::ToggleSection(state_id) => {
                if !self.collapsed_sections.remove(&state_id) {
                    self.collapsed_sections.insert(state_id);
                }
            }
        }
        output
    }
}

impl Content {
    pub fn new(storage: Store, config: config::AppConfig) -> Self {
        let states = storage.states().load_all().unwrap_or_else(|err| {
            tracing::error!("Failed to load task states: {err}");
            Vec::new()
        });

        Self {
            selected_list: None,
            tasks: SlotMap::new(),
            editing: SecondaryMap::new(),
            inputs: SecondaryMap::new(),
            add_task_input: String::new(),
            config: config,
            store: storage,
            states,
            collapsed_sections: HashSet::new(),
            search_bar_visible: false,
            search_query: String::new(),
            drag_hover: None,
        }
    }

    pub fn find_task_key(&self, task_id: uuid::Uuid) -> Option<DefaultKey> {
        self.tasks
            .iter()
            .find(|(_, t)| t.id == task_id)
            .map(|(k, _)| k)
    }

    fn reorder_tasks(&mut self, from_uuid: Uuid, onto_key: DefaultKey) {
        let Some(list) = &self.selected_list else {
            tracing::warn!("reorder_tasks: no list selected");
            return;
        };
        let list_id = list.id;

        let Some(from_key) = self.find_task_key(from_uuid) else {
            tracing::warn!("reorder_tasks: source task {from_uuid} not found");
            return;
        };

        if from_key == onto_key {
            return;
        }

        let mut ordered: Vec<DefaultKey> = {
            let mut tasks: Vec<_> = self
                .tasks
                .iter()
                .filter(|(_, t)| t.parent_id.is_none())
                .collect();
            tasks.sort_by(|a, b| {
                a.1.sort_order
                    .cmp(&b.1.sort_order)
                    .then(a.1.creation_date.cmp(&b.1.creation_date))
            });
            tasks.into_iter().map(|(k, _)| k).collect()
        };

        let Some(from_pos) = ordered.iter().position(|k| *k == from_key) else {
            tracing::warn!("reorder_tasks: from_key not found in ordered list");
            return;
        };
        let Some(onto_pos) = ordered.iter().position(|k| *k == onto_key) else {
            tracing::warn!("reorder_tasks: onto_key not found in ordered list");
            return;
        };

        ordered.remove(from_pos);
        ordered.insert(onto_pos, from_key);

        for (idx, key) in ordered.iter().enumerate() {
            let new_order = idx as u32;
            if let Some(task) = self.tasks.get_mut(*key) {
                if task.sort_order != new_order {
                    task.sort_order = new_order;
                    let task_id = task.id;
                    if let Err(e) = self
                        .store
                        .tasks(list_id)
                        .update(task_id, |t| t.sort_order = new_order)
                    {
                        tracing::error!("Failed to persist sort_order for {task_id}: {e}");
                    }
                }
            }
        }
    }

    pub fn list_view<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let mut column = widget::column::with_capacity(3);
        column = column.push(self.list_header(list));

        if self.search_bar_visible {
            column = column.push(self.create_search_input(&spacing));
        }

        let sorted_tasks = self.sort_tasks();
        let visible_tasks: Vec<_> = sorted_tasks
            .into_iter()
            .filter(|(_, task)| self.should_show_task(task))
            .collect();

        if visible_tasks.is_empty() && self.search_query.is_empty() {
            return self.empty(list);
        }

        let sections = self.section_views(visible_tasks);
        let items = widget::column::with_children(sections).spacing(spacing.space_s);

        widget::scrollable(
            widget::container(column.push(items).spacing(spacing.space_s)).height(Length::Shrink),
        )
        .spacing(spacing.space_xxs)
        .height(Length::Fill)
        .into()
    }

    fn list_header<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let title = widget::text::title4(&list.name).width(Length::Fill);
        let search_button = self.create_search_button(&spacing);
        let list_icon = self.create_list_icon(list, &spacing);

        widget::row::with_capacity(3)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(list_icon)
            .push(title)
            .push(search_button)
            .into()
    }

    fn create_search_input<'a>(&'a self, spacing: &Spacing) -> Element<'a, Message> {
        let placeholder = match &self.selected_list {
            Some(list) => fl!("search-list", list = list.name.as_str()),
            None => fl!("search-tasks"),
        };

        widget::text_input(placeholder, &self.search_query)
            .id(widget::Id::new("search-tasks-input"))
            .on_input(Message::SearchQueryChanged)
            .width(Length::Fill)
            .padding([spacing.space_xxs, spacing.space_xxs])
            .into()
    }

    fn create_search_button<'a>(&'a self, spacing: &Spacing) -> Element<'a, Message> {
        widget::button::icon(widget::icon::from_name("edit-find-symbolic").size(18))
            .selected(self.search_bar_visible)
            .padding(spacing.space_xxs)
            .on_press(Message::ToggleSearchBar)
            .into()
    }

    fn create_list_icon<'a>(&'a self, list: &'a List, spacing: &Spacing) -> Element<'a, Message> {
        widget::icon::from_name(list.icon.as_deref().unwrap_or("view-list-symbolic"))
            .size(spacing.space_m)
            .into()
    }

    fn sort_tasks(&self) -> Vec<(DefaultKey, &Task)> {
        let mut tasks: Vec<_> = self.tasks.iter().collect();

        match self.config.sort_by {
            SortBy::NameAsc => {
                tasks.sort_by(|a, b| a.1.title.to_lowercase().cmp(&b.1.title.to_lowercase()))
            }
            SortBy::NameDesc => {
                tasks.sort_by(|a, b| b.1.title.to_lowercase().cmp(&a.1.title.to_lowercase()))
            }
            SortBy::DateAsc => tasks.sort_by(|a, b| a.1.creation_date.cmp(&b.1.creation_date)),
            SortBy::DateDesc => tasks.sort_by(|a, b| b.1.creation_date.cmp(&a.1.creation_date)),
            SortBy::Manual => {
                tasks.sort_by(|a, b| {
                    a.1.sort_order
                        .cmp(&b.1.sort_order)
                        .then(a.1.creation_date.cmp(&b.1.creation_date))
                });
            }
        }

        tasks
    }

    fn section_views<'a>(
        &'a self,
        tasks: Vec<(DefaultKey, &'a Task)>,
    ) -> Vec<Element<'a, Message>> {
        let mut states: Vec<&TaskState> = self.states.iter().collect();
        states.sort_by_key(|state| state.position);

        states
            .into_iter()
            .filter_map(|state| {
                let section_tasks: Vec<_> = tasks
                    .iter()
                    .copied()
                    .filter(|(_, task)| task.effective_state_id() == state.id)
                    .collect();

                (!section_tasks.is_empty()).then(|| self.section_view(state, section_tasks))
            })
            .collect()
    }

    fn section_view<'a>(
        &'a self,
        state: &'a TaskState,
        tasks: Vec<(DefaultKey, &'a Task)>,
    ) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;
        let collapsed = self.collapsed_sections.contains(&state.id);

        let header = collapsible_section::section_header(
            state.name.clone(),
            None,
            tasks.len(),
            collapsed,
            Vec::new(),
            Message::ToggleSection(state.id),
            &spacing,
        );

        let rows = tasks
            .into_iter()
            .map(|(id, task)| self.task_view(id, task))
            .collect();

        collapsible_section::section(header, rows, collapsed)
    }

    fn should_show_task(&self, task: &Task) -> bool {
        let is_top_level = task.parent_id.is_none();

        let matches_search = !self.search_bar_visible
            || self.search_query.is_empty()
            || task
                .title
                .to_lowercase()
                .contains(&self.search_query.to_lowercase());

        let show_despite_completion = !self.config.hide_completed || !task.is_completed();

        is_top_level && matches_search && show_despite_completion
    }

    pub fn task_view<'a>(&'a self, id: DefaultKey, task: &'a Task) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let sub_tasks = self.get_subtasks(task);
        let task_row = self.create_task_row(id, task, &sub_tasks, &spacing);

        let mut column = widget::column::with_capacity(2).push(task_row);

        if task.expanded && !sub_tasks.is_empty() {
            column = column.push(self.create_subtasks_view(&sub_tasks, &spacing));
        }

        let uuid = task.id;
        let is_hover = self.drag_hover == Some(id);

        let inner: Element<'_, Message> = if is_hover {
            widget::column::with_capacity(2)
                .push(
                    cosmic::iced::widget::rule::horizontal(4).class(theme::Rule::custom(|theme| {
                        cosmic::iced::widget::rule::Style {
                            color: theme.cosmic().accent_color().into(),
                            radius: 0.0.into(),
                            fill_mode: cosmic::iced::widget::rule::FillMode::Full,
                            snap: false,
                        }
                    })),
                )
                .push(column)
                .into()
        } else {
            column.into()
        };

        widget::dnd_destination::dnd_destination_for_data::<TaskDrag, Message>(
            widget::dnd_source::<Message, TaskDrag>(inner)
                .drag_content(move || TaskDrag { uuid })
                .on_start(Some(Message::DragStarted(id))),
            move |data, _action| Message::TaskDropped {
                from: data.map(|d| d.uuid),
                onto: id,
            },
        )
        .on_enter(move |_x, _y, _mimes| Message::DragEntered(id))
        .on_leave(move || Message::DragLeft)
        .into()
    }

    fn get_subtasks(&self, task: &Task) -> Vec<(DefaultKey, &Task)> {
        let should_hide_completed = self.config.hide_completed;

        self.tasks
            .iter()
            .filter(|(_, sub_task)| {
                let is_child = sub_task.parent_id == Some(task.id);
                let show_despite_completion = !should_hide_completed || !sub_task.is_completed();
                is_child && show_despite_completion
            })
            .collect()
    }

    fn create_task_row<'a>(
        &'a self,
        id: DefaultKey,
        task: &'a Task,
        sub_tasks: &[(DefaultKey, &Task)],
        spacing: &Spacing,
    ) -> Element<'a, Message> {
        let checkbox = self.create_task_checkbox(id, task);
        let title_input = self.create_task_title_input(id, task);
        let expand_button = self.create_expand_button(id, task, sub_tasks, spacing);
        let subtask_count = self.create_subtask_counter(sub_tasks);
        let favorite_button = self.create_favorite_button(id, task, spacing);
        let menu = self.create_task_menu(id);

        let drag_handle: Option<Element<'_, Message>> = (self.config.sort_by == SortBy::Manual)
            .then(|| {
                if task.parent_id.is_none() {
                    widget::icon::from_name("grip-lines-symbolic")
                        .size(16)
                        .into()
                } else {
                    widget::Space::new().width(Length::Fixed(16.0)).into()
                }
            });

        widget::row::with_capacity(7)
            .align_y(Alignment::Center)
            .spacing(spacing.space_xxxs)
            .padding([spacing.space_xxxs, spacing.space_s])
            .push_maybe(drag_handle)
            .push(checkbox)
            .push(title_input)
            .push_maybe(expand_button)
            .push_maybe(subtask_count)
            .push(favorite_button)
            .push(menu)
            .into()
    }

    fn create_task_checkbox<'a>(&'a self, id: DefaultKey, task: &'a Task) -> Element<'a, Message> {
        widget::checkbox(task.is_completed())
            .on_toggle(move |value| Message::TaskComplete(id, value))
            .into()
    }

    fn create_favorite_button<'a>(
        &'a self,
        id: DefaultKey,
        task: &'a Task,
        spacing: &Spacing,
    ) -> Element<'a, Message> {
        let icon_name = if task.favorite {
            "starred-symbolic"
        } else {
            "non-starred-symbolic"
        };
        widget::button::icon(widget::icon::from_name(icon_name).size(16))
            .padding(spacing.space_xxs)
            .on_press(Message::TaskToggleFavorite(id))
            .into()
    }

    fn create_task_title_input<'a>(
        &'a self,
        id: DefaultKey,
        task: &'a Task,
    ) -> Element<'a, Message> {
        let is_editing = matches!(
            self.editing.get(id),
            Some(EditState::Entering) | Some(EditState::Editing)
        );

        widget::editable_input("", &task.title, is_editing, move |editing| {
            Message::TaskToggleTitleEditMode(id, editing)
        })
        .trailing_icon(widget::column(vec![]).into())
        .id(self.inputs[id].clone())
        .on_submit(move |_| Message::TaskTitleSubmit(id))
        .on_input(move |text| Message::TaskTitleUpdate(id, text))
        .into()
    }

    fn create_expand_button<'a>(
        &'a self,
        id: DefaultKey,
        task: &'a Task,
        sub_tasks: &[(DefaultKey, &Task)],
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

    fn create_subtask_counter<'a>(
        &'a self,
        sub_tasks: &[(DefaultKey, &Task)],
    ) -> Option<Element<'a, Message>> {
        if sub_tasks.is_empty() {
            return None;
        }

        let (completed, total) = sub_tasks.iter().fold((0, 0), |acc, (_, subtask)| {
            if subtask.is_completed() {
                (acc.0 + 1, acc.1 + 1)
            } else {
                (acc.0, acc.1 + 1)
            }
        });

        Some(widget::text(format!("{}/{}", completed, total)).into())
    }

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

    fn create_subtasks_view<'a>(
        &'a self,
        sub_tasks: &[(DefaultKey, &'a Task)],
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

    fn populate_task_slotmap(&mut self, tasks: Vec<Task>) {
        for task in tasks {
            let task_id = self.tasks.insert(task);
            self.inputs.insert(task_id, widget::Id::unique());
            self.editing.insert(task_id, EditState::Idle);
        }
    }

    fn reconcile_tasks(&mut self, tasks: Vec<Task>) {
        let disk_ids: HashSet<Uuid> = tasks.iter().map(|t| t.id).collect();

        let stale_keys: Vec<DefaultKey> = self
            .tasks
            .iter()
            .filter(|(_, t)| !disk_ids.contains(&t.id))
            .map(|(key, _)| key)
            .collect();
        for key in stale_keys {
            self.tasks.remove(key);
            self.inputs.remove(key);
            self.editing.remove(key);
        }

        for task in tasks {
            let existing_key = self
                .tasks
                .iter()
                .find(|(_, t)| t.id == task.id)
                .map(|(k, _)| k);
            match existing_key {
                Some(key) => {
                    let is_editing = matches!(
                        self.editing.get(key),
                        Some(EditState::Editing) | Some(EditState::Entering)
                    );
                    if !is_editing {
                        if let Some(existing) = self.tasks.get_mut(key) {
                            *existing = task;
                        }
                    }
                }
                None => {
                    let key = self.tasks.insert(task);
                    self.inputs.insert(key, widget::Id::unique());
                    self.editing.insert(key, EditState::Idle);
                }
            }
        }
    }

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
