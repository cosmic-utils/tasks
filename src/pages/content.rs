use cosmic::{
    Apply, Element,
    iced::{
        Alignment, Length,
        alignment::{Horizontal, Vertical},
    },
    widget,
};
use slotmap::{DefaultKey, SlotMap};

use crate::{
    app::config::Config,
    fl,
    model::{List, Status, Task},
    services::store::Store,
};

#[derive(Debug, Clone)]
pub struct Content {
    config: Config,
    store: Store,
    selected_list: Option<List>,
    tasks: SlotMap<DefaultKey, Task>,
    new_task_title: String,
    search_query: String,
    sort_type: SortType,
    search_bar_visible: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddTask,
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
}

impl Content {
    pub fn new(store: Store) -> Self {
        Self {
            config: Config::default(),
            store,
            selected_list: None,
            tasks: SlotMap::new(),
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

    pub fn update(&mut self, message: Message) -> Option<Output> {
        match message {
            Message::AddTask => {
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
                        // self.task_input_ids.insert(task_id, widget::Id::unique());
                        // self.task_editing.insert(task_id, false);
                        // if !task.sub_tasks.is_empty() {
                        //     self.populate_sub_task_slotmap(task.sub_tasks);
                        // }
                    }
                }
            }
            Message::ToggleSearchBar => {
                self.search_bar_visible = !self.search_bar_visible;
                if !self.search_bar_visible {
                    self.search_query.clear();
                }
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
            }
            Message::ToggleHideCompleted => {
                if let Some(ref mut list) = self.selected_list {
                    list.hide_completed = !list.hide_completed;
                    return Some(Output::ToggleHideCompleted(list.clone()));
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

    pub fn task_view<'a>(&'a self, _id: DefaultKey, task: &'a Task) -> Element<'a, Message> {
        let spacing = cosmic::theme::spacing();

        let is_completed = task.status == Status::Completed;
        let item_checkbox = widget::checkbox(is_completed);

        let task_item_text = widget::text(&task.title).size(14);

        let row = widget::row::with_capacity(2)
            .push(item_checkbox)
            .push(task_item_text)
            .push(widget::space::horizontal())
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_xxs, spacing.space_s]);

        widget::container(row)
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
                    .on_submit(|_| Message::AddTask)
                    .width(Length::Fill),
            )
            .push(
                widget::button::icon(widget::icon::from_name("mail-send-symbolic").size(18))
                    .padding(spacing.space_xxs)
                    .class(cosmic::style::Button::Suggested)
                    .on_press(Message::AddTask),
            )
            .padding(spacing.space_xxs)
            .spacing(spacing.space_xxs)
            .align_y(Alignment::Center)
            .into()
    }
}
