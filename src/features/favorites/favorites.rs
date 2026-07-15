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

pub struct FavoriteEntry {
    pub task: Task,
    pub list_id: Uuid,
    pub list_name: String,
}

pub struct Favorites {
    entries: Vec<FavoriteEntry>,
    store: Store,
    collapsed_sections: HashSet<Uuid>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Load,
    Loaded(Vec<FavoriteEntry>),
    Unfavorite(Uuid),
    Open(Uuid),
    ToggleSection(Uuid),
}

pub enum Output {
    OpenTask { task: Task, list_id: Uuid },
}

impl std::fmt::Debug for FavoriteEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FavoriteEntry")
            .field("task_id", &self.task.id)
            .field("list_name", &self.list_name)
            .finish()
    }
}

impl Clone for FavoriteEntry {
    fn clone(&self) -> Self {
        Self {
            task: self.task.clone(),
            list_id: self.list_id,
            list_name: self.list_name.clone(),
        }
    }
}

impl Favorites {
    pub fn new(store: Store) -> Self {
        Self {
            entries: Vec::new(),
            store,
            collapsed_sections: HashSet::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Option<Output> {
        match message {
            Message::Load => {
                let lists: Vec<List> = self.store.lists().load_all().unwrap_or_else(|e| {
                    tracing::error!("Failed to load lists for favorites: {e}");
                    vec![]
                });

                let mut entries = Vec::new();
                for list in &lists {
                    let tasks = self.store.tasks(list.id).load_all().unwrap_or_else(|e| {
                        tracing::error!("Failed to load tasks for list {}: {e}", list.id);
                        vec![]
                    });
                    for task in tasks {
                        if task.favorite {
                            entries.push(FavoriteEntry {
                                task,
                                list_id: list.id,
                                list_name: list.name.clone(),
                            });
                        }
                    }
                }

                entries.sort_by(|a, b| {
                    a.list_name
                        .cmp(&b.list_name)
                        .then_with(|| a.task.title.cmp(&b.task.title))
                });

                self.update(Message::Loaded(entries));
            }
            Message::Loaded(entries) => {
                self.entries = entries;
            }
            Message::Unfavorite(task_id) => {
                if let Some(pos) = self.entries.iter().position(|e| e.task.id == task_id) {
                    let list_id = self.entries[pos].list_id;
                    if let Err(e) = self
                        .store
                        .tasks(list_id)
                        .update(task_id, |t| t.favorite = false)
                    {
                        tracing::error!("Failed to unfavorite task: {e}");
                    } else {
                        self.entries.remove(pos);
                    }
                }
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

    pub fn view(&self) -> Element<'_, Message> {
        if self.entries.is_empty() {
            return self.empty_view();
        }

        let spacing = theme::active().cosmic().spacing;

        let header = self.header_view();

        let sections = self.list_sections();
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

    fn header_view(&self) -> Element<'_, Message> {
        let spacing = theme::active().cosmic().spacing;

        let icon = widget::icon::from_name("starred-symbolic").size(spacing.space_m);
        let title = widget::text::title4(fl!("favorites")).width(Length::Fill);

        widget::row::with_capacity(2)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(icon)
            .push(title)
            .into()
    }

    fn list_sections(&self) -> Vec<Element<'_, Message>> {
        let mut list_ids: Vec<Uuid> = Vec::new();
        for e in &self.entries {
            if !list_ids.contains(&e.list_id) {
                list_ids.push(e.list_id);
            }
        }

        let mut sections: Vec<(String, Element<'_, Message>)> = list_ids
            .into_iter()
            .map(|list_id| {
                let entries: Vec<&FavoriteEntry> = self
                    .entries
                    .iter()
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
        entries: Vec<&'a FavoriteEntry>,
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

    fn entry_row<'a>(&'a self, entry: &'a FavoriteEntry) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let task_id = entry.task.id;
        let star_button =
            widget::button::icon(widget::icon::from_name("starred-symbolic").size(16))
                .padding(spacing.space_xxs)
                .on_press(Message::Unfavorite(task_id));

        let title = widget::text::body(entry.task.title.as_str()).width(Length::Fill);

        let open_button =
            widget::button::icon(widget::icon::from_name("go-next-symbolic").size(16))
                .padding(spacing.space_xxs)
                .on_press(Message::Open(task_id));

        let row = widget::row::with_capacity(3)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_xxs, spacing.space_xs])
            .push(star_button)
            .push(title)
            .push(open_button);

        collapsible_section::row_item(row.into())
    }

    fn empty_view(&self) -> Element<'_, Message> {
        widget::container(
            widget::column::with_children(vec![
                widget::icon::from_name("non-starred-symbolic")
                    .size(56)
                    .into(),
                widget::text::title1(fl!("no-favorites")).into(),
                widget::text(fl!("no-favorites-suggestion")).into(),
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
