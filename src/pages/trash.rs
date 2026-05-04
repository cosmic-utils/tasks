use cosmic::{
    cosmic_theme::Spacing,
    iced::{
        alignment::{Horizontal, Vertical},
        Alignment, Length,
    },
    theme, widget, Apply, Element,
};
use uuid::Uuid;

use crate::{
    fl,
    model::{List, TrashedTask},
    services::store::Store,
};

/// One or more trashed tasks pending permanent deletion (with undo window).
struct PendingDeletion {
    tasks: Vec<TrashedTask>,
    seconds_remaining: u8,
}

pub struct Trash {
    pub tasks: Vec<TrashedTask>,
    pub lists: Vec<List>,
    store: Store,
    pending_deletion: Option<PendingDeletion>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Load,
    Loaded(Vec<TrashedTask>, Vec<List>),
    RestoreTask(Uuid),
    /// Begin the permanent-deletion countdown for a task.
    DeleteTask(Uuid),
    /// Advance the permanent-deletion countdown by one second.
    TaskDeletionTick,
    /// Cancel the pending permanent deletion (undo).
    TaskDeletionUndo,
    EmptyTrash,
    /// Restore every task in the trash back to its original list.
    RestoreAll,
}

impl Trash {
    pub fn new(store: Store) -> Self {
        Self {
            tasks: Vec::new(),
            lists: Vec::new(),
            store,
            pending_deletion: None,
        }
    }

    /// Returns `true` when a permanent deletion countdown is active.
    pub fn has_pending_deletion(&self) -> bool {
        self.pending_deletion.is_some()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Load => {
                let tasks = self.store.trash().load_all().unwrap_or_else(|e| {
                    tracing::error!("Failed to load trash: {e}");
                    vec![]
                });
                let lists = self.store.lists().load_all().unwrap_or_else(|e| {
                    tracing::error!("Failed to load lists for trash: {e}");
                    vec![]
                });
                return self.update(Message::Loaded(tasks, lists));
            }
            Message::Loaded(tasks, lists) => {
                self.tasks = tasks;
                self.lists = lists;
            }
            Message::RestoreTask(task_id) => {
                if let Some(pos) = self.tasks.iter().position(|t| t.task.id == task_id) {
                    let trashed = self.tasks.remove(pos);
                    if let Err(e) = self
                        .store
                        .tasks(trashed.original_list_id)
                        .save(&trashed.task)
                    {
                        tracing::error!("Failed to restore task: {e}");
                    } else if let Err(e) = self.store.trash().delete(task_id) {
                        tracing::error!("Failed to remove task from trash after restore: {e}");
                    }
                }
            }
            Message::DeleteTask(task_id) => {
                // Commit any previous pending permanent deletion first.
                if let Some(existing) = self.pending_deletion.take() {
                    for t in existing.tasks {
                        if let Err(e) = self.store.trash().delete(t.task.id) {
                            tracing::error!("Error committing previous permanent deletion: {e}");
                        }
                    }
                }

                if let Some(pos) = self.tasks.iter().position(|t| t.task.id == task_id) {
                    let trashed = self.tasks.remove(pos);
                    self.pending_deletion = Some(PendingDeletion {
                        tasks: vec![trashed],
                        seconds_remaining: 5,
                    });
                }
            }
            Message::TaskDeletionTick => {
                if let Some(pending) = &mut self.pending_deletion {
                    if pending.seconds_remaining > 0 {
                        pending.seconds_remaining -= 1;
                    }
                    if pending.seconds_remaining == 0 {
                        let pending = self.pending_deletion.take().unwrap();
                        for t in pending.tasks {
                            if let Err(e) = self.store.trash().delete(t.task.id) {
                                tracing::error!("Error permanently deleting task from trash: {e}");
                            }
                        }
                    }
                }
            }
            Message::TaskDeletionUndo => {
                if let Some(mut pending) = self.pending_deletion.take() {
                    // Restore tasks in their original order (oldest first).
                    pending.tasks.reverse();
                    for t in pending.tasks {
                        self.tasks.insert(0, t);
                    }
                }
            }
            Message::EmptyTrash => {
                // Commit any previous pending permanent deletion first.
                if let Some(existing) = self.pending_deletion.take() {
                    for t in existing.tasks {
                        if let Err(e) = self.store.trash().delete(t.task.id) {
                            tracing::error!("Error committing previous permanent deletion: {e}");
                        }
                    }
                }

                if !self.tasks.is_empty() {
                    let all_tasks = std::mem::take(&mut self.tasks);
                    self.pending_deletion = Some(PendingDeletion {
                        tasks: all_tasks,
                        seconds_remaining: 5,
                    });
                }
            }
            Message::RestoreAll => {
                // Commit any in-flight permanent deletion first.
                self.pending_deletion.take();
                let tasks = std::mem::take(&mut self.tasks);
                for trashed in tasks {
                    let task_id = trashed.task.id;
                    if let Err(e) = self
                        .store
                        .tasks(trashed.original_list_id)
                        .save(&trashed.task)
                    {
                        tracing::error!("Failed to restore task during RestoreAll: {e}");
                    } else if let Err(e) = self.store.trash().delete(task_id) {
                        tracing::error!("Failed to remove task from trash during RestoreAll: {e}");
                    }
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let spacing = theme::active().cosmic().spacing;

        if self.tasks.is_empty() && self.pending_deletion.is_none() {
            return self.empty_view();
        }

        let header = self.header_view();

        let task_rows: Vec<Element<'_, Message>> =
            self.tasks.iter().map(|t| self.task_row(t)).collect();

        let list = widget::column::with_children(task_rows).spacing(spacing.space_xxs);

        let mut content = widget::column::with_capacity(3)
            .push(header)
            .push(widget::scrollable(list).height(Length::Fill))
            .spacing(spacing.space_s)
            .padding([spacing.space_xxs, spacing.space_xxxs]);

        if let Some(ref pending) = self.pending_deletion {
            content = content.push(self.deletion_banner(pending, &spacing));
        }

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

        let icon = widget::icon::from_name("user-trash-full-symbolic").size(spacing.space_m);

        let title = widget::text::body(fl!("trash"))
            .size(24)
            .width(Length::Fill);

        // Disable the button while a countdown is already running.
        let empty_button = widget::button::destructive(fl!("empty-trash")).on_press_maybe(
            self.pending_deletion
                .is_none()
                .then_some(Message::EmptyTrash),
        );

        widget::row::with_capacity(3)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(icon)
            .push(title)
            .push(empty_button)
            .into()
    }

    fn task_row<'a>(&'a self, trashed: &'a TrashedTask) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let list_name = self
            .lists
            .iter()
            .find(|l| l.id == trashed.original_list_id)
            .map(|l| l.name.clone())
            .unwrap_or_else(|| fl!("unknown-list"));

        let title = widget::text::body(trashed.task.title.as_str()).width(Length::Fill);

        let subtitle = widget::text(fl!("deleted-from", list = list_name.as_str()))
            .size(12)
            .width(Length::Fill);

        let task_id = trashed.task.id;
        let deleted_at = trashed.deleted_at_local();

        let date = widget::text(fl!("deleted-at", date = deleted_at.as_str())).size(11);

        let restore_button =
            widget::button::standard(fl!("restore")).on_press(Message::RestoreTask(task_id));

        let delete_button = widget::button::destructive(fl!("delete-permanently"))
            .on_press(Message::DeleteTask(task_id));

        let text_col = widget::column::with_capacity(3)
            .push(title)
            .push(subtitle)
            .push(date)
            .width(Length::Fill);

        let row = widget::row::with_capacity(3)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_xxxs, spacing.space_xs])
            .push(text_col)
            .push(restore_button)
            .push(delete_button);

        widget::container(row)
            .class(cosmic::style::Container::ContextDrawer)
            .width(Length::Fill)
            .into()
    }

    fn deletion_banner<'a>(
        &'a self,
        pending: &'a PendingDeletion,
        spacing: &Spacing,
    ) -> Element<'a, Message> {
        let label: String = if pending.tasks.len() == 1 {
            fl!("task-deleted", title = pending.tasks[0].task.title.as_str())
        } else {
            fl!("trash-emptied")
        };

        widget::container(
            widget::row::with_capacity(3)
                .push(widget::text(label).width(Length::Fill))
                .push(widget::text(fl!(
                    "deletion-countdown",
                    seconds = pending.seconds_remaining
                )))
                .push(widget::button::standard(fl!("undo")).on_press(Message::TaskDeletionUndo))
                .align_y(Alignment::Center)
                .spacing(spacing.space_s),
        )
        .class(cosmic::style::Container::Primary)
        .padding(spacing.space_xxxs)
        .width(Length::Fill)
        .into()
    }

    fn empty_view(&self) -> Element<'_, Message> {
        widget::container(
            widget::column::with_children(vec![
                widget::icon::from_name("user-trash-symbolic")
                    .size(56)
                    .into(),
                widget::text::title1(fl!("no-trash")).into(),
                widget::text(fl!("no-trash-suggestion")).into(),
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
