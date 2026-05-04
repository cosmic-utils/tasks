use jiff::civil::Date;

use cosmic::{
    iced::{Alignment, Length},
    theme,
    widget::{
        self,
        segmented_button::{self, Entity},
        text_editor,
    },
    Element,
};
use slotmap::DefaultKey;
use uuid::Uuid;

use crate::{
    fl,
    model::{self, Priority},
    services::store::Store,
};

pub struct Details {
    pub task: model::Task,
    pub task_key: DefaultKey,
    pub selected_list: Option<Uuid>,
    pub priority_model: segmented_button::Model<segmented_button::SingleSelect>,
    pub text_editor_content: widget::text_editor::Content,
    pub store: Store,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetTask(DefaultKey, model::Task, Uuid),
    SetTitle(String),
    Editor(text_editor::Action),
    Favorite(bool),
    PriorityActivate(Entity),
    Delete,
    OpenCalendarDialog,
    SetDueDate(Date),
}

pub enum Output {
    OpenCalendarDialog,
    RefreshTask(model::Task),
    /// Request deletion of the currently displayed task; routed to content's
    /// undo-timer flow by the app model.
    DeleteTask(DefaultKey),
}

impl Details {
    pub fn new(storage: Store) -> Self {
        let priority_model = segmented_button::ModelBuilder::default()
            .insert(|entity| {
                entity
                    .icon(widget::icon::from_name("security-low-symbolic").size(14))
                    .data(Priority::Low)
            })
            .insert(|entity| {
                entity
                    .icon(widget::icon::from_name("security-medium-symbolic").size(14))
                    .data(Priority::Normal)
            })
            .insert(|entity| {
                entity
                    .icon(widget::icon::from_name("security-high-symbolic").size(14))
                    .data(Priority::High)
            })
            .build();

        Self {
            task: model::Task::default(),
            task_key: DefaultKey::default(),
            selected_list: None,
            priority_model,
            text_editor_content: widget::text_editor::Content::new(),
            store: storage,
        }
    }

    pub fn update(&mut self, message: Message) -> Option<Output> {
        match message {
            Message::SetTask(key, task, list_id) => {
                self.task_key = key;
                self.task = task.clone();
                self.selected_list = Some(list_id);

                let entity = self.priority_model.entity_at(task.priority as u16);
                if let Some(entity) = entity {
                    self.priority_model.activate(entity);
                }
                self.task = task.clone();
                self.text_editor_content = widget::text_editor::Content::with_text(&task.notes);
            }
            Message::Editor(action) => {
                self.text_editor_content.perform(action);
                self.task.notes.clone_from(&self.text_editor_content.text());
            }
            Message::SetTitle(title) => {
                self.task.title.clone_from(&title);
            }
            Message::Favorite(favorite) => {
                self.task.favorite = favorite;
            }
            Message::PriorityActivate(entity) => {
                self.priority_model.activate(entity);
                let priority = self.priority_model.data::<Priority>(entity);
                if let Some(priority) = priority {
                    self.task.priority = *priority;
                }
            }
            Message::Delete => {
                return Some(Output::DeleteTask(self.task_key));
            }
            Message::OpenCalendarDialog => {
                return Some(Output::OpenCalendarDialog);
            }
            Message::SetDueDate(date) => {
                self.task.due_date = Some(date);
            }
        }

        if let Some(list_id) = self.selected_list {
            if let Err(e) = self
                .store
                .tasks(list_id)
                .update(self.task.id, |t| *t = self.task.clone())
            {
                tracing::error!("Failed to update task: {}", e);
            }
        }

        return Some(Output::RefreshTask(self.task.clone()));
    }

    pub fn view(&self) -> Element<'_, Message> {
        let spacing = theme::active().cosmic().spacing;

        widget::settings::view_column(vec![
            widget::settings::section()
                .add(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("title")).into(),
                        widget::text_input(fl!("title"), &self.task.title)
                            .on_input(Message::SetTitle)
                            .size(13)
                            .into(),
                    ])
                    .padding([
                        spacing.space_s,
                        spacing.space_none,
                        spacing.space_s,
                        spacing.space_none,
                    ])
                    .spacing(spacing.space_xxs),
                )
                .add(
                    widget::settings::item::builder(fl!("favorite"))
                        .control(widget::checkbox(self.task.favorite).on_toggle(Message::Favorite)),
                )
                .add(
                    widget::settings::item::builder(fl!("priority")).control(
                        widget::segmented_control::horizontal(&self.priority_model)
                            .button_alignment(Alignment::Center)
                            .width(Length::Shrink)
                            .on_activate(Message::PriorityActivate),
                    ),
                )
                .add(
                    widget::settings::item::builder(fl!("due-date")).control(
                        widget::button::text(if self.task.due_date.is_some() {
                            self.task
                                .due_date
                                .as_ref()
                                .unwrap()
                                .strftime("%m-%d-%Y")
                                .to_string()
                        } else {
                            fl!("select-date")
                        })
                        .on_press(Message::OpenCalendarDialog),
                    ),
                )
                .add(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("notes")).into(),
                        widget::text_editor(&self.text_editor_content)
                            .class(crate::app::ui::style::text_editor())
                            .padding(spacing.space_xxs)
                            .placeholder(fl!("add-notes"))
                            .height(100.0)
                            .size(13)
                            .on_action(Message::Editor)
                            .into(),
                    ])
                    .spacing(spacing.space_xxs)
                    .padding([
                        spacing.space_s,
                        spacing.space_none,
                        spacing.space_s,
                        spacing.space_none,
                    ]),
                )
                .into(),
            widget::settings::section()
                .add(
                    widget::settings::item::builder(fl!("created-at"))
                        .control(widget::text::caption(self.task.creation_date_local())),
                )
                .add_maybe(self.task.completion_date_local().map(|completion_date| {
                    widget::settings::item::builder(fl!("completed-at"))
                        .control(widget::text::caption(completion_date))
                }))
                .into(),
            widget::button::destructive(fl!("delete"))
                .trailing_icon(widget::icon::from_name("edit-delete-symbolic").size(14))
                .on_press(Message::Delete)
                .into(),
        ])
        .padding([
            spacing.space_none,
            spacing.space_s,
            spacing.space_none,
            spacing.space_s,
        ])
        .into()
    }
}
