use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Alignment, Length,
    },
    theme, widget, Apply, Element,
};

use uuid::Uuid;

use crate::{
    fl,
    model::{List, Task},
    services::store::Store,
};

/// A favorite task together with the name and id of the list it belongs to.
pub struct FavoriteEntry {
    pub task: Task,
    pub list_id: Uuid,
    pub list_name: String,
}

pub struct Favorites {
    entries: Vec<FavoriteEntry>,
    store: Store,
}

#[derive(Debug, Clone)]
pub enum Message {
    Load,
    Loaded(Vec<FavoriteEntry>),
    Unfavorite(Uuid),
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
        }
    }

    pub fn update(&mut self, message: Message) {
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

                // Sort by list name then task title for a stable, predictable order.
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
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.entries.is_empty() {
            return self.empty_view();
        }

        let spacing = theme::active().cosmic().spacing;

        let header = self.header_view();

        let rows: Vec<Element<'_, Message>> =
            self.entries.iter().map(|e| self.entry_row(e)).collect();

        let list = widget::column::with_children(rows).spacing(spacing.space_xxs);

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
        let title = widget::text::body(fl!("favorites"))
            .size(24)
            .width(Length::Fill);

        widget::row::with_capacity(2)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(icon)
            .push(title)
            .into()
    }

    fn entry_row<'a>(&'a self, entry: &'a FavoriteEntry) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let task_id = entry.task.id;
        let star_button =
            widget::button::icon(widget::icon::from_name("starred-symbolic").size(16))
                .padding(spacing.space_xxs)
                .on_press(Message::Unfavorite(task_id));

        let title = widget::text::body(entry.task.title.as_str()).width(Length::Fill);
        let list_label = widget::text(entry.list_name.as_str())
            .size(12)
            .class(cosmic::style::Text::Default);

        let text_col = widget::column::with_capacity(2)
            .push(title)
            .push(list_label)
            .width(Length::Fill);

        let row = widget::row::with_capacity(2)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_xxxs, spacing.space_xs])
            .push(star_button)
            .push(text_col);

        widget::container(row)
            .class(cosmic::style::Container::ContextDrawer)
            .width(Length::Fill)
            .into()
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
