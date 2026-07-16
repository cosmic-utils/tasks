use std::collections::HashSet;

use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Alignment, Length,
    },
    theme, widget, Apply, Element,
};

use uuid::Uuid;

use crate::{
    features::{lists::list::List, tasks::task::Task},
    fl,
    shared::{store::Store, widgets::collapsible_section},
};

#[derive(Debug, Clone)]
pub struct SearchEntry {
    pub task: Task,
    pub list_id: Uuid,
    pub list_name: String,
}

pub struct Search {
    query: String,
    entries: Vec<SearchEntry>,
    collapsed_sections: HashSet<Uuid>,
    store: Store,
}

#[derive(Debug, Clone)]
pub enum Message {
    QueryChanged(String),
    Load,
    Loaded(Vec<SearchEntry>),
    Open(Uuid),
    ToggleSection(Uuid),
}

pub enum Output {
    OpenTask { task: Task, list_id: Uuid },
}

impl Search {
    pub fn new(store: Store) -> Self {
        Self {
            query: String::new(),
            entries: Vec::new(),
            collapsed_sections: HashSet::new(),
            store,
        }
    }

    pub fn has_query(&self) -> bool {
        !self.query.trim().is_empty()
    }

    pub fn update(&mut self, message: Message) -> Option<Output> {
        match message {
            Message::QueryChanged(query) => {
                self.query = query;
            }
            Message::Load => {
                let lists: Vec<List> = self.store.lists().load_all().unwrap_or_else(|e| {
                    tracing::error!("Failed to load lists for search: {e}");
                    vec![]
                });

                let mut entries = Vec::new();
                for list in &lists {
                    let tasks = self.store.tasks(list.id).load_all().unwrap_or_else(|e| {
                        tracing::error!("Failed to load tasks for list {}: {e}", list.id);
                        vec![]
                    });
                    for task in tasks {
                        entries.push(SearchEntry {
                            task,
                            list_id: list.id,
                            list_name: list.name.clone(),
                        });
                    }
                }

                return self.update(Message::Loaded(entries));
            }
            Message::Loaded(entries) => {
                self.entries = entries;
            }
            Message::Open(task_id) => {
                if let Some(entry) = self.entries.iter().find(|e| e.task.id == task_id) {
                    return Some(Output::OpenTask {
                        task: entry.task.clone(),
                        list_id: entry.list_id,
                    });
                }
            }
            Message::ToggleSection(list_id) => {
                if !self.collapsed_sections.remove(&list_id) {
                    self.collapsed_sections.insert(list_id);
                }
            }
        }
        None
    }

    pub fn header_view(&self) -> Element<'_, Message> {
        widget::search_input(fl!("search-all-tasks"), &self.query)
            .id(widget::Id::new("global-search-input"))
            .on_input(Message::QueryChanged)
            .on_clear(Message::QueryChanged(String::new()))
            .width(Length::Fixed(240.0))
            .into()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let spacing = theme::active().cosmic().spacing;

        let query = self.query.to_lowercase();
        let matches: Vec<&SearchEntry> = self
            .entries
            .iter()
            .filter(|e| e.task.title.to_lowercase().contains(&query))
            .collect();

        if matches.is_empty() {
            return self.no_results_view();
        }

        let header = self.header_row(matches.len());
        let sections = self.list_sections(matches);
        let list = widget::column::with_children(sections).spacing(spacing.space_xxs);

        let content = widget::column::with_capacity(2)
            .push(header)
            .push(widget::scrollable(list).height(Length::Fill))
            .spacing(spacing.space_s)
            .padding([spacing.space_xxs, spacing.space_xxxs]);

        widget::container(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .center_x(Length::Fill)
            .max_width(800.)
            .apply(widget::container)
            .height(Length::Fill)
            .width(Length::Fill)
            .center(Length::Fill)
            .into()
    }

    fn header_row(&self, count: usize) -> Element<'_, Message> {
        let spacing = theme::active().cosmic().spacing;

        let icon = widget::icon::from_name("edit-find-symbolic").size(spacing.space_m);
        let title = widget::text::title4(fl!("search-results", count = count)).width(Length::Fill);

        widget::row::with_capacity(2)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(icon)
            .push(title)
            .into()
    }

    fn list_sections<'a>(&'a self, matches: Vec<&'a SearchEntry>) -> Vec<Element<'a, Message>> {
        let mut list_ids: Vec<Uuid> = Vec::new();
        for e in &matches {
            if !list_ids.contains(&e.list_id) {
                list_ids.push(e.list_id);
            }
        }

        let mut sections: Vec<(String, Element<'_, Message>)> = list_ids
            .into_iter()
            .map(|list_id| {
                let entries: Vec<&SearchEntry> = matches
                    .iter()
                    .copied()
                    .filter(|e| e.list_id == list_id)
                    .collect();
                let name = entries
                    .first()
                    .map(|e| e.list_name.clone())
                    .unwrap_or_else(|| fl!("unknown-list"));

                let section = self.list_section(list_id, name.clone(), entries);
                (name, section)
            })
            .collect();

        sections.sort_by(|a, b| a.0.cmp(&b.0));
        sections.into_iter().map(|(_, section)| section).collect()
    }

    fn list_section<'a>(
        &'a self,
        list_id: Uuid,
        name: String,
        entries: Vec<&'a SearchEntry>,
    ) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;
        let collapsed = self.collapsed_sections.contains(&list_id);
        let count = entries.len();

        let header = collapsible_section::section_header(
            name,
            None,
            count,
            collapsed,
            Vec::new(),
            Message::ToggleSection(list_id),
            &spacing,
        );

        let rows = entries
            .into_iter()
            .map(|entry| self.entry_row(entry))
            .collect();

        collapsible_section::section(header, rows, collapsed)
    }

    fn entry_row<'a>(&'a self, entry: &'a SearchEntry) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let title = widget::text::body(entry.task.title.as_str()).width(Length::Fill);

        let open_button =
            widget::button::icon(widget::icon::from_name("go-next-symbolic").size(16))
                .padding(spacing.space_xxs)
                .on_press(Message::Open(entry.task.id));

        let row = widget::row::with_capacity(2)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_xxs, spacing.space_xxs])
            .push(title)
            .push(open_button);

        collapsible_section::row_item(row.into())
    }

    fn no_results_view(&self) -> Element<'_, Message> {
        widget::container(
            widget::column::with_children(vec![
                widget::icon::from_name("edit-find-symbolic")
                    .size(56)
                    .into(),
                widget::text::title1(fl!("no-search-results")).into(),
                widget::text(fl!("no-search-results-suggestion")).into(),
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
