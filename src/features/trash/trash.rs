use std::collections::{HashMap, HashSet};

use cosmic::{
    cosmic_theme::Spacing,
    iced::{
        alignment::{Horizontal, Vertical},
        Alignment, Length,
    },
    theme,
    widget::{self, icon::Named},
    Apply, Element,
};
use uuid::Uuid;

use crate::{
    features::{
        lists::list::{List, TrashedList},
        tasks::task::{Task, TrashedTask},
    },
    fl,
    shared::{store::Store, widgets::collapsible_section},
};

struct PendingDeletion {
    tasks: Vec<TrashedTask>,
    seconds_remaining: u8,
}

pub struct Trash {
    pub tasks: Vec<TrashedTask>,
    pub lists: Vec<List>,
    pub trashed_lists: Vec<TrashedList>,
    pub trashed_list_tasks: HashMap<Uuid, Vec<Task>>,
    store: Store,
    pending_deletion: Option<PendingDeletion>,
    collapsed_sections: HashSet<Uuid>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Load,
    Loaded(
        Vec<TrashedTask>,
        Vec<List>,
        Vec<TrashedList>,
        HashMap<Uuid, Vec<Task>>,
    ),
    ToggleSection(Uuid),
    RestoreTask(Uuid),
    RequestDeleteTask(Uuid),
    DeleteTask(Uuid),
    RequestRestoreList(Uuid),
    RequestDeleteList(Uuid),
    DeleteListConfirmed(Uuid),
    RequestRestoreTaskFromList(Uuid, Uuid),
    RequestDeleteTaskFromList(Uuid, Uuid),
    DeleteTaskFromListConfirmed(Uuid, Uuid),
    TaskDeletionTick,
    TaskDeletionUndo,
    EmptyTrash,
    EmptyTrashConfirmed,
    RestoreAll,
}

pub enum Output {
    EmptyTrashRequested,
    DeleteTaskRequested(Uuid, String),
    RestoreListRequested(Uuid),
    DeleteListRequested(Uuid, String),
    RestoreTaskFromListRequested(Uuid, Uuid),
    DeleteTaskFromListRequested(Uuid, Uuid, String),
}

impl Trash {
    pub fn new(store: Store) -> Self {
        Self {
            tasks: Vec::new(),
            lists: Vec::new(),
            trashed_lists: Vec::new(),
            trashed_list_tasks: HashMap::new(),
            store,
            pending_deletion: None,
            collapsed_sections: HashSet::new(),
        }
    }

    pub fn has_pending_deletion(&self) -> bool {
        self.pending_deletion.is_some()
    }

    pub fn update(&mut self, message: Message) -> Option<Output> {
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
                let trashed_lists = self.store.trash().load_all_lists().unwrap_or_else(|e| {
                    tracing::error!("Failed to load trashed lists: {e}");
                    vec![]
                });
                let mut trashed_list_tasks = HashMap::new();
                for trashed_list in &trashed_lists {
                    let list_tasks = self
                        .store
                        .trash()
                        .load_trashed_list_tasks(trashed_list.list.id)
                        .unwrap_or_else(|e| {
                            tracing::error!("Failed to load tasks for trashed list: {e}");
                            vec![]
                        });
                    trashed_list_tasks.insert(trashed_list.list.id, list_tasks);
                }
                return self.update(Message::Loaded(
                    tasks,
                    lists,
                    trashed_lists,
                    trashed_list_tasks,
                ));
            }
            Message::Loaded(tasks, lists, trashed_lists, trashed_list_tasks) => {
                self.tasks = tasks;
                self.lists = lists;
                self.trashed_lists = trashed_lists;
                self.trashed_list_tasks = trashed_list_tasks;
            }
            Message::ToggleSection(list_id) => {
                if !self.collapsed_sections.remove(&list_id) {
                    self.collapsed_sections.insert(list_id);
                }
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
            Message::RequestDeleteTask(task_id) => {
                if let Some(trashed) = self.tasks.iter().find(|t| t.task.id == task_id) {
                    return Some(Output::DeleteTaskRequested(
                        task_id,
                        trashed.task.title.clone(),
                    ));
                }
            }
            Message::DeleteTask(task_id) => {
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
            Message::RequestRestoreList(list_id) => {
                return Some(Output::RestoreListRequested(list_id));
            }
            Message::RequestDeleteList(list_id) => {
                if let Some(trashed) = self.trashed_lists.iter().find(|t| t.list.id == list_id) {
                    return Some(Output::DeleteListRequested(
                        list_id,
                        trashed.list.name.clone(),
                    ));
                }
            }
            Message::DeleteListConfirmed(list_id) => {
                if let Err(e) = self.store.trash().delete_list(list_id) {
                    tracing::error!("Error permanently deleting list from trash: {e}");
                }
                self.trashed_lists.retain(|t| t.list.id != list_id);
                self.trashed_list_tasks.remove(&list_id);
            }
            Message::RequestRestoreTaskFromList(list_id, task_id) => {
                return Some(Output::RestoreTaskFromListRequested(list_id, task_id));
            }
            Message::RequestDeleteTaskFromList(list_id, task_id) => {
                if let Some(task) = self
                    .trashed_list_tasks
                    .get(&list_id)
                    .and_then(|tasks| tasks.iter().find(|t| t.id == task_id))
                {
                    return Some(Output::DeleteTaskFromListRequested(
                        list_id,
                        task_id,
                        task.title.clone(),
                    ));
                }
            }
            Message::DeleteTaskFromListConfirmed(list_id, task_id) => {
                if let Err(e) = self.store.trash().delete_task_from_list(list_id, task_id) {
                    tracing::error!("Error permanently deleting task from trashed list: {e}");
                }
                if let Some(tasks) = self.trashed_list_tasks.get_mut(&list_id) {
                    tasks.retain(|t| t.id != task_id);
                    if tasks.is_empty() {
                        self.trashed_list_tasks.remove(&list_id);
                        self.trashed_lists.retain(|t| t.list.id != list_id);
                    }
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
                    pending.tasks.reverse();
                    for t in pending.tasks {
                        self.tasks.insert(0, t);
                    }
                }
            }
            Message::EmptyTrash => {
                return Some(Output::EmptyTrashRequested);
            }
            Message::EmptyTrashConfirmed => {
                let mut tasks = std::mem::take(&mut self.tasks);
                if let Some(pending) = self.pending_deletion.take() {
                    tasks.extend(pending.tasks);
                }
                for t in tasks {
                    if let Err(e) = self.store.trash().delete(t.task.id) {
                        tracing::error!("Error emptying trash: {e}");
                    }
                }
                for trashed in std::mem::take(&mut self.trashed_lists) {
                    if let Err(e) = self.store.trash().delete_list(trashed.list.id) {
                        tracing::error!("Error emptying trashed list: {e}");
                    }
                }
                self.trashed_list_tasks.clear();
            }
            Message::RestoreAll => {
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

        None
    }

    pub fn view(&self) -> Element<'_, Message> {
        let spacing = theme::active().cosmic().spacing;

        if self.is_empty() {
            return self.empty_view();
        }

        let header = self.header_view();

        let sections = self.list_sections();
        let list = widget::column::with_children(sections).spacing(spacing.space_xxs);

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

        let title = widget::text::title4(fl!("trash")).width(Length::Fill);

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

    fn list_sections(&self) -> Vec<Element<'_, Message>> {
        let mut list_ids: Vec<Uuid> = Vec::new();
        for t in &self.tasks {
            if !list_ids.contains(&t.original_list_id) {
                list_ids.push(t.original_list_id);
            }
        }
        for t in &self.trashed_lists {
            if !list_ids.contains(&t.list.id) {
                list_ids.push(t.list.id);
            }
        }

        let mut sections: Vec<(String, Element<'_, Message>)> = list_ids
            .into_iter()
            .map(|list_id| {
                let trashed_list = self.trashed_lists.iter().find(|t| t.list.id == list_id);
                let name = trashed_list
                    .map(|t| t.list.name.clone())
                    .or_else(|| {
                        self.lists
                            .iter()
                            .find(|l| l.id == list_id)
                            .map(|l| l.name.clone())
                    })
                    .unwrap_or_else(|| fl!("unknown-list"));

                let individually: Vec<&TrashedTask> = self
                    .tasks
                    .iter()
                    .filter(|t| t.original_list_id == list_id)
                    .collect();
                let from_list: Vec<&Task> = self
                    .trashed_list_tasks
                    .get(&list_id)
                    .map(|tasks| tasks.iter().collect())
                    .unwrap_or_default();

                let section =
                    self.list_section(list_id, name.clone(), trashed_list, individually, from_list);
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
        trashed_list: Option<&'a TrashedList>,
        individually: Vec<&'a TrashedTask>,
        from_list: Vec<&'a Task>,
    ) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;
        let collapsed = self.collapsed_sections.contains(&list_id);
        let count = individually.len() + from_list.len();

        // Once any single task has been restored out of a trashed list, the
        // list itself becomes active again (see `restore_task_from_list`),
        // even though some of its tasks may remain in trash. In that case
        // whole-list actions no longer apply — only the per-task ones do.
        let list_is_trashed = trashed_list.is_some() && !self.lists.iter().any(|l| l.id == list_id);

        let mut extra_buttons: Vec<Element<'a, Message>> = Vec::new();
        if list_is_trashed {
            extra_buttons.push(
                widget::button::icon(widget::icon::from_name("edit-undo-symbolic"))
                    .tooltip(fl!("restore-list"))
                    .class(theme::Button::Standard)
                    .on_press(Message::RequestRestoreList(list_id))
                    .into(),
            );
            extra_buttons.push(
                widget::button::icon(widget::icon::from_name("edit-delete-symbolic"))
                    .tooltip(fl!("delete-permanently"))
                    .class(theme::Button::Destructive)
                    .on_press(Message::RequestDeleteList(list_id))
                    .into(),
            );
        }

        let subtitle = trashed_list.map(|trashed_list| {
            let deleted_at = trashed_list.deleted_at_local();
            fl!("deleted-at", date = deleted_at.as_str())
        });

        let header = collapsible_section::section_header(
            name,
            subtitle,
            count,
            collapsed,
            extra_buttons,
            Message::ToggleSection(list_id),
            &spacing,
        );

        let rows = individually
            .into_iter()
            .map(|trashed| {
                self.task_row(
                    trashed.task.title.as_str(),
                    Some(trashed.deleted_at_local()),
                    Message::RestoreTask(trashed.task.id),
                    Message::RequestDeleteTask(trashed.task.id),
                )
            })
            .chain(from_list.into_iter().map(|task| {
                self.task_row(
                    task.title.as_str(),
                    None,
                    Message::RequestRestoreTaskFromList(list_id, task.id),
                    Message::RequestDeleteTaskFromList(list_id, task.id),
                )
            }))
            .collect();

        collapsible_section::section(header, rows, collapsed)
    }

    fn task_row<'a>(
        &'a self,
        title: &'a str,
        deleted_at: Option<String>,
        restore_message: Message,
        delete_message: Message,
    ) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let mut text_col = widget::column::with_capacity(2)
            .push(widget::text::body(title))
            .width(Length::Fill);
        if let Some(deleted_at) = deleted_at {
            text_col = text_col.push(widget::text::caption(fl!(
                "deleted-at",
                date = deleted_at.as_str()
            )));
        }

        let restore_button = widget::button::icon(widget::icon::from_name("edit-undo-symbolic"))
            .tooltip(fl!("restore"))
            .class(theme::Button::Standard)
            .on_press(restore_message);

        let delete_button = widget::button::icon(widget::icon::from_name("edit-delete-symbolic"))
            .tooltip(fl!("delete-permanently"))
            .class(theme::Button::Destructive)
            .on_press(delete_message);

        let row = widget::row::with_capacity(3)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_xxs, spacing.space_xs])
            .push(text_col)
            .push(restore_button)
            .push(delete_button);

        collapsible_section::row_item(row.into())
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

    pub fn icon(&self) -> Named {
        if self.is_empty() {
            widget::icon::from_name("user-trash-symbolic").size(16)
        } else {
            widget::icon::from_name("user-trash-full-symbolic").size(16)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty() && self.trashed_lists.is_empty() && self.pending_deletion.is_none()
    }
}
