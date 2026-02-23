use std::collections::HashMap;

use crate::{
    actions::task::TaskAction,
    app::config::Config,
    fl,
    model::{List, Status, Task},
    services::store::Store,
};
use cosmic::{
    Apply, Element,
    iced::{
        Alignment, Length,
        alignment::{Horizontal, Vertical},
    },
    widget,
};
use rust_extensions::Toggle;
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Content {
    config: Config,
    store: Store,
    selected_list: Option<List>,
    tasks: SlotMap<DefaultKey, Task>,
    inputs: SecondaryMap<DefaultKey, widget::Id>,
    editing: SecondaryMap<DefaultKey, bool>,
    new_task_title: String,
    search_query: String,
    sort_type: SortType,
    search_bar_visible: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Task(TaskAction),
    SetSelectedList(Option<List>),
    ToggleSearchBar,
    ToggleHideCompleted,
    SetSort(SortType),
    SetNewTaskTitle(String),
    SearchQueryChanged(String),
    SetConfig(Config),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SortType {
    NameAsc,
    NameDesc,
    DateAsc,
    DateDesc,
}

pub enum Output {
    ToggleHideCompleted(List),
    ToggleTaskDetails(Uuid),
}

impl Content {
    pub fn new(store: &Store) -> Self {
        Self {
            config: Config::default(),
            store: store.clone(),
            selected_list: None,
            tasks: SlotMap::new(),
            inputs: SecondaryMap::new(),
            editing: SecondaryMap::new(),
            new_task_title: String::new(),
            sort_type: SortType::NameAsc,
            search_bar_visible: false,
            search_query: String::new(),
        }
    }

    pub fn view(&self) -> impl Into<Element<'_, Message>> {
        let spacing = cosmic::theme::spacing();

        let Some(ref list) = self.selected_list else {
            return widget::container(
                widget::column()
                    .push(widget::icon::from_name("applications-office-symbolic").size(56))
                    .push(widget::text::title1(fl!("no-list-selected")))
                    .push(widget::text(fl!("no-list-suggestion")))
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
            .push(self.new_task_field())
            .spacing(spacing.space_xxs)
            .max_width(800.)
            .apply(widget::container)
            .height(Length::Fill)
            .width(Length::Fill)
            .padding([spacing.space_xxs, spacing.space_none])
    }

    pub fn update(&mut self, message: Message) -> Option<crate::app::Message> {
        match message {
            Message::Task(action) => match action {
                TaskAction::Add => {
                    if let Some(list) = &self.selected_list
                        && !self.new_task_title.is_empty()
                    {
                        let task = Task::new(&self.new_task_title);
                        match self.store.tasks(list.id.clone()).save(&task) {
                            Ok(_) => {
                                self.new_task_title.clear();
                                self.tasks.insert(task);
                            }
                            Err(error) => {
                                tracing::error!("Failed to create task: {:?}", error);
                            }
                        }
                    }
                }
                TaskAction::Complete(id, complete) => {
                    if let Some(task) = self.tasks.get_mut(id) {
                        let list = self.selected_list.as_ref()?;

                        task.status = if complete {
                            Status::Completed
                        } else {
                            Status::NotStarted
                        };

                        if let Err(error) = self
                            .store
                            .tasks(list.id)
                            .update(task.id, |t| t.status = task.status)
                        {
                            tracing::error!("Failed to update task: {:?}", error);
                        }
                    }
                }
                TaskAction::Expand(id) => {
                    if let Some(task) = self.tasks.get_mut(id) {
                        let list = self.selected_list.clone()?;
                        match self
                            .store
                            .tasks(list.id)
                            .update(task.id, |t| t.expanded.toggle())
                        {
                            Ok(_) => task.expanded.toggle(),
                            Err(error) => tracing::error!("Failed to expand task: {:?}", error),
                        }
                    }
                }
                TaskAction::Edit(id) => match self.tasks.get(id) {
                    Some(task) => {
                        return Some(crate::app::Message::Details(
                            crate::pages::details::Message::Open(
                                self.selected_list.map(|l| l.id),
                                task.id,
                            ),
                        ));
                    }
                    None => tracing::error!("Task not found for editing: {:?}", id),
                },
                TaskAction::Delete(id) => {
                    if let Some(task) = self.tasks.get(id) {
                        let list = self.selected_list.clone()?;
                        match self.store.tasks(list.id).delete(task.id) {
                            Ok(_) => {
                                self.tasks.remove(id);
                                self.inputs.remove(id);
                                self.editing.remove(id);
                            }
                            Err(error) => tracing::error!("Failed to delete task: {:?}", error),
                        }
                    }
                }
                TaskAction::AddSubTask(id) => {
                    // This is a placeholder for the actual implementation of adding a sub-task.
                    // You would likely want to show a dialog or input field to enter the sub-task details, and then save it to the store and update the UI accordingly.
                    tracing::info!("Add sub-task for task ID: {:?}", id);
                }
                TaskAction::ToggleEditMode(id, editing) => {
                    self.editing.insert(id, editing);
                    if editing {
                        return Some(crate::app::Message::Focus(self.inputs[id].clone()));
                    } else if let Some(task) = self.tasks.get(id) {
                        if let Err(error) = self
                            .store
                            .tasks(self.selected_list?.id)
                            .update(task.id, |t| t.title = task.title.clone())
                        {
                            tracing::error!("Failed to update task: {:?}", error);
                        }
                    }
                }
                TaskAction::TitleSubmit(id) => {
                    if let Some(task) = self.tasks.get(id) {
                        let list = self.selected_list.clone()?;
                        match self
                            .store
                            .tasks(list.id)
                            .update(task.id, |t| t.title = task.title.clone())
                        {
                            Ok(_) => {
                                self.editing.insert(id, false);
                            }
                            Err(error) => {
                                tracing::error!("Failed to update task title: {:?}", error)
                            }
                        }
                    }
                }
                TaskAction::TitleUpdate(id, title) => {
                    if let Some(task) = self.tasks.get_mut(id) {
                        task.title = title;
                    }
                }
            },
            Message::SetConfig(config) => {
                self.config = config;
            }
            Message::SetSelectedList(list) => {
                self.selected_list = list;
                self.tasks.clear();

                let Some(ref list) = self.selected_list else {
                    return None;
                };

                if let Ok(tasks) = self.store.tasks(list.id.clone()).load_all() {
                    for task in tasks {
                        let task_id = self.tasks.insert(task.clone());
                        self.inputs.insert(task_id, widget::Id::unique());
                        self.editing.insert(task_id, false);
                        if !task.sub_tasks.is_empty() {
                            self.populate_sub_tasks(task.sub_tasks);
                        }
                    }
                }
            }
            Message::ToggleSearchBar => {
                self.search_bar_visible.toggle();
                if !self.search_bar_visible {
                    self.search_query.clear();
                }
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
            }
            Message::ToggleHideCompleted => {
                if let Some(ref mut list) = self.selected_list {
                    list.hide_completed.toggle();
                    return Some(crate::app::Message::ToggleHideCompleted(list.clone()));
                }
            }
            Message::SetSort(sort_type) => self.sort_type = sort_type,
            Message::SetNewTaskTitle(title) => self.new_task_title = title,
        }
        None
    }

    pub fn list_view<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = cosmic::theme::spacing();

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

        let filtered_tasks: Vec<_> = tasks
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
            .padding([spacing.space_none, spacing.space_xs])
            .spacing(spacing.space_s)
            .apply(widget::container)
            .height(Length::Shrink)
            .apply(widget::scrollable)
            .height(Length::Fill)
            .into()
    }

    fn list_header<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = cosmic::theme::spacing();

        let hide_completed_active = list.hide_completed || self.config.hide_completed;
        let mut hide_completed_button =
            widget::button::icon(widget::icon::from_name("check-round-outline-symbolic").size(18))
                .selected(hide_completed_active)
                .padding(spacing.space_xxs);

        if hide_completed_active {
            hide_completed_button = hide_completed_button.class(cosmic::style::Button::Suggested);
        }

        hide_completed_button = hide_completed_button.on_press(Message::ToggleHideCompleted);

        let search_button =
            widget::button::icon(widget::icon::from_name("edit-find-symbolic").size(18))
                .selected(self.search_bar_visible)
                .padding(spacing.space_xxs)
                .on_press(Message::ToggleSearchBar);

        let icon = list.icon.as_deref().unwrap_or("view-list-symbolic");

        widget::row::with_capacity(4)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(widget::icon::from_name(icon).size(24))
            .push(widget::text::body(&list.name).size(24).width(Length::Fill))
            .push(hide_completed_button)
            .push(search_button)
            .into()
    }

    pub fn task_view<'a>(&'a self, id: DefaultKey, task: &'a Task) -> Element<'a, Message> {
        let spacing = cosmic::theme::spacing();

        let is_completed = task.status == Status::Completed;
        let icon = if task.expanded {
            "go-up-symbolic"
        } else {
            "go-down-symbolic"
        };
        let (completed, total) = task.sub_tasks.iter().fold((0, 0), |acc, subtask| {
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

        let more_button = widget::menu::MenuBar::new(vec![widget::menu::Tree::with_children(
            Element::from(cosmic::widget::button::icon(
                widget::icon::from_name("view-more-symbolic").size(18),
            )),
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

        widget::container(
            widget::row()
                .push(
                    widget::checkbox(is_completed).on_toggle(move |complete| {
                        Message::Task(TaskAction::Complete(id, complete))
                    }),
                )
                .push(
                    widget::editable_input(
                        "",
                        &task.title,
                        *self.editing.get(id).unwrap_or(&false),
                        move |editing| Message::Task(TaskAction::ToggleEditMode(id, editing)),
                    )
                    .size(13)
                    .trailing_icon(widget::column().into())
                    .id(self.inputs[id].clone())
                    .on_submit(move |_| Message::Task(TaskAction::TitleSubmit(id)))
                    .on_input(move |text| Message::Task(TaskAction::TitleUpdate(id, text))),
                )
                .push_maybe((!task.sub_tasks.is_empty()).then(|| {
                    widget::button::icon(widget::icon::from_name(icon).size(18))
                        .padding(spacing.space_xxs)
                        .on_press(Message::Task(TaskAction::Expand(id)))
                }))
                .push_maybe(subtask_count)
                .push(more_button)
                .align_y(Alignment::Center)
                .spacing(spacing.space_s)
                .padding([spacing.space_xxs, spacing.space_s]),
        )
        .padding(spacing.space_xxs)
        .class(cosmic::style::Container::ContextDrawer)
        .into()
    }

    pub fn empty<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = cosmic::theme::spacing();

        let container = widget::container(
            widget::column()
                .push(widget::icon::from_name("task-past-due-symbolic").size(56))
                .push(widget::text::title1(fl!("no-tasks")))
                .push(widget::text(fl!("no-tasks-suggestion")))
                .spacing(spacing.space_xs)
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

    pub fn new_task_field(&self) -> Element<'_, Message> {
        let spacing = cosmic::theme::spacing();

        widget::row()
            .push(
                widget::text_input(fl!("add-new-task"), &self.new_task_title)
                    .id(widget::Id::new("new-task-input"))
                    .on_input(Message::SetNewTaskTitle)
                    .on_submit(|_| Message::Task(TaskAction::Add))
                    .width(Length::Fill),
            )
            .push(
                widget::button::icon(widget::icon::from_name("mail-send-symbolic").size(18))
                    .padding(spacing.space_xxs)
                    .class(cosmic::style::Button::Suggested)
                    .on_press(Message::Task(TaskAction::Add)),
            )
            .padding(spacing.space_xxs)
            .spacing(spacing.space_xxs)
            .align_y(Alignment::Center)
            .into()
    }

    fn populate_sub_tasks(&mut self, tasks: Vec<Task>) {
        for task in tasks {
            let task_id = self.tasks.insert(task.clone());
            self.inputs.insert(task_id, widget::Id::unique());
            self.editing.insert(task_id, false);
            if !task.sub_tasks.is_empty() {
                self.populate_sub_tasks(task.sub_tasks);
            }
        }
    }
}
