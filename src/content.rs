use crate::app::config;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Color, Length, Subscription};
use cosmic::iced_widget::row;
use cosmic::prelude::CollectionWidget;
use cosmic::{cosmic_theme, theme, widget, Apply, Element};
use done_core::models::list::List;
use done_core::models::status::Status;
use done_core::models::task::Task;
use slotmap::{DefaultKey, SecondaryMap, SlotMap};

use crate::fl;

pub struct Content {
    list: Option<List>,
    tasks: SlotMap<DefaultKey, Task>,
    editing: SecondaryMap<DefaultKey, bool>,
    task_input_ids: SecondaryMap<DefaultKey, widget::Id>,
    input: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddTask,
    Complete(DefaultKey, bool),
    Delete(DefaultKey),
    EditMode(DefaultKey, bool),
    Export(Vec<Task>),
    Input(String),
    ItemDown,
    ItemUp,
    List(Option<List>),
    Select(Task),
    SetItems(Vec<Task>),
    TitleSubmit(DefaultKey),
    TitleUpdate(DefaultKey, String),
    UpdateTask(Task),
}

pub enum Command {
    Iced(cosmic::app::Command<super::app::Message>),
    GetTasks(String),
    DisplayTask(Task),
    UpdateTask(Task),
    Delete(String),
    CreateTask(Task),
    Export(Vec<Task>),
}

impl Content {
    pub fn new() -> Self {
        Self {
            list: None,
            tasks: SlotMap::new(),
            editing: SecondaryMap::new(),
            task_input_ids: SecondaryMap::new(),
            input: String::new(),
        }
    }

    fn list_header<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let cosmic_theme::Spacing {
            space_none,
            space_xxs,
            space_s,
            ..
        } = theme::active().cosmic().spacing;
        let export_button = widget::button(config::get_icon("share-symbolic", 18))
            .style(theme::Button::Suggested)
            .padding(space_xxs)
            .on_press(Message::Export(self.tasks.values().cloned().collect()));

        widget::row::with_capacity(3)
            .align_items(Alignment::Center)
            .spacing(space_s)
            .padding([space_none, space_xxs])
            .push_maybe(
                list.icon
                    .as_deref()
                    .map(|icon| widget::icon::from_name(icon).size(24).icon()),
            )
            .push(widget::text::title3(&list.name).width(Length::Fill))
            .push(export_button)
            .into()
    }

    pub fn list_view<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let cosmic_theme::Spacing {
            space_none,
            space_xxxs,
            space_xxs,
            ..
        } = theme::active().cosmic().spacing;

        if self.tasks.is_empty() {
            return self.empty(list);
        }

        let mut items = widget::list::list_column()
            .style(theme::Container::ContextDrawer)
            .spacing(space_xxxs)
            .padding([space_none, space_xxs]);

        for (id, item) in &self.tasks {
            let item_checkbox =
                widget::checkbox("", item.status == Status::Completed, move |value| {
                    Message::Complete(id, value)
                });

            let delete_button = widget::button(config::get_icon("user-trash-full-symbolic", 18))
                .padding(space_xxs)
                .style(theme::Button::Destructive)
                .on_press(Message::Delete(id));

            let details_button = widget::button(config::get_icon("info-outline-symbolic", 18))
                .padding(space_xxs)
                .style(theme::Button::Standard)
                .on_press(Message::Select(item.clone()));

            let task_item_text =
                widget::editable_input("", &item.title, *self.editing.get(id).unwrap_or(&false), {
                    let id = id.clone();
                    move |editing| Message::EditMode(id, editing)
                })
                .id(self.task_input_ids[id].clone())
                .on_submit(Message::TitleSubmit(id.clone()))
                .on_input(move |text| Message::TitleUpdate(id, text))
                .width(Length::Fill);

            let row = widget::row::with_capacity(4)
                .align_items(Alignment::Center)
                .spacing(space_xxs)
                .padding([space_xxxs, space_xxs])
                .push(item_checkbox)
                .push(task_item_text)
                .push(details_button)
                .push(delete_button);

            // let button = widget::button(row)
            //     .padding([space_xxs, space_xs])
            //     .width(Length::Fill)
            //     .height(Length::Shrink)
            //     .style(button_style(false, true))
            //     .on_press(Message::Select(item.clone()));

            items = items.add(row);
        }

        widget::column::with_capacity(2)
            .spacing(space_xxs)
            .push(self.list_header(list))
            .push(items)
            .apply(widget::container)
            .height(Length::Shrink)
            .padding([0, space_xxs, 0, space_xxs])
            .apply(widget::scrollable)
            .height(Length::Fill)
            .into()
    }

    pub fn empty<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let container = widget::container(
            widget::column::with_children(vec![
                config::get_icon("task-past-due-symbolic", 56).into(),
                widget::text::title1(fl!("no-tasks")).into(),
                widget::text(fl!("no-tasks-suggestion")).into(),
            ])
            .spacing(10)
            .align_items(Alignment::Center),
        )
        .align_y(Vertical::Center)
        .align_x(Horizontal::Center)
        .height(Length::Fill)
        .width(Length::Fill);

        widget::column::with_capacity(2)
            .spacing(space_xxs)
            .padding([0, space_xxs, 0, space_xxs])
            .push(self.list_header(list))
            .push(container)
            .into()
    }

    pub fn new_task_view(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
        row(vec![
            widget::text_input(fl!("add-new-task"), &self.input)
                .on_input(Message::Input)
                .on_submit(Message::AddTask)
                .width(Length::Fill)
                .into(),
            widget::button(config::get_icon("mail-send-symbolic", 18))
                .padding(space_xxs)
                .style(theme::Button::Suggested)
                .on_press(Message::AddTask)
                .into(),
        ])
        .padding([space_xxs, space_xxs, 0, space_xxs])
        .spacing(space_xxs)
        .align_items(Alignment::Center)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Vec<Command> {
        let mut commands = Vec::new();
        match message {
            Message::List(list) => {
                self.list = list.clone();
                if let Some(list) = list {
                    commands.push(Command::GetTasks(list.id));
                }
            }
            Message::ItemDown => {}
            Message::ItemUp => {}
            Message::TitleUpdate(id, title) => {
                if let Some(task) = self.tasks.get_mut(id) {
                    task.title = title;
                }
            }
            Message::TitleSubmit(id) => {
                if let Some(task) = self.tasks.get(id) {
                    commands.push(Command::UpdateTask(task.clone()));
                    self.editing.insert(id, false);
                }
            }
            Message::Delete(id) => {
                if let Some(task) = self.tasks.remove(id) {
                    commands.push(Command::Delete(task.id));
                }
            }
            Message::EditMode(id, editing) => {
                self.editing.insert(id, editing);
                if editing {
                    commands.push(Command::Iced(widget::text_input::focus(
                        self.task_input_ids[id].clone(),
                    )));
                } else if let Some(task) = self.tasks.get(id) {
                    commands.push(Command::UpdateTask(task.clone()));
                }
            }
            Message::SetItems(tasks) => {
                self.tasks.clear();
                tasks.into_iter().for_each(|task| {
                    let id = self.tasks.insert(task);
                    self.task_input_ids.insert(id, widget::Id::unique());
                });
            }
            Message::Select(task) => {
                commands.push(Command::DisplayTask(task));
            }
            Message::Complete(id, complete) => {
                let task = self.tasks.get_mut(id);
                if let Some(task) = task {
                    task.status = if complete {
                        Status::Completed
                    } else {
                        Status::NotStarted
                    };
                    commands.push(Command::UpdateTask(task.clone()));
                }
            }
            Message::Input(input) => self.input = input,
            Message::AddTask => {
                if let Some(list) = &self.list {
                    if !self.input.is_empty() {
                        let task = Task::new(self.input.clone(), list.id.clone());
                        commands.push(Command::CreateTask(task.clone()));
                        let id = self.tasks.insert(task);
                        self.task_input_ids.insert(id, widget::Id::unique());
                        self.input.clear();
                    }
                }
            }
            Message::UpdateTask(updated_task) => {
                let task = self.tasks.values_mut().find(|t| t.id == updated_task.id);
                if let Some(task) = task {
                    *task = updated_task.clone();
                    commands.push(Command::UpdateTask(task.clone()));
                }
            }
            Message::Export(tasks) => {
                commands.push(Command::Export(tasks));
            }
        }
        commands
    }

    pub fn view(&self) -> Element<Message> {
        let Some(ref list) = self.list else {
            return widget::container(
                widget::column::with_children(vec![
                    config::get_icon("applications-office-symbolic", 56).into(),
                    widget::text::title1(fl!("no-list-selected")).into(),
                    widget::text(fl!("no-list-suggestion")).into(),
                ])
                .spacing(10)
                .align_items(Alignment::Center),
            )
            .align_y(Vertical::Center)
            .align_x(Horizontal::Center)
            .height(Length::Fill)
            .width(Length::Fill)
            .into();
        };

        widget::column::with_capacity(2)
            .push(self.list_view(list))
            .push(self.new_task_view())
            .apply(widget::container)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

fn button_appearance(
    theme: &theme::Theme,
    selected: bool,
    focused: bool,
    accent: bool,
    hovered: bool,
) -> widget::button::Appearance {
    let cosmic = theme.cosmic();
    // TODO: Use this instead when it's working properly.
    // let container = theme.current_container();

    let mut appearance = widget::button::Appearance::new();

    if selected {
        if accent {
            appearance.background = Some(Color::from(cosmic.primary_component_color()).into());
            appearance.icon_color = Some(Color::from(cosmic.on_accent_color()));
            appearance.text_color = Some(Color::from(cosmic.on_accent_color()));
        } else {
            appearance.background = Some(Color::from(cosmic.bg_component_color()).into());
        }
    }

    if hovered {
        appearance.background = Some(Color::from(cosmic.secondary_component_color()).into());
        appearance.icon_color = Some(Color::from(cosmic.on_secondary_component_color()));
        appearance.text_color = Some(Color::from(cosmic.on_secondary_component_color()));
    }

    if focused && accent {
        appearance.outline_width = 1.0;
        appearance.outline_color = Color::from(cosmic.accent_color());
        appearance.border_width = 2.0;
        appearance.border_color = Color::TRANSPARENT;
    }
    appearance.border_radius = cosmic.radius_s().into();
    appearance
}

fn button_style(selected: bool, accent: bool) -> theme::Button {
    //TODO: move to libcosmic?
    theme::Button::Custom {
        active: Box::new(move |focused, theme| {
            button_appearance(theme, selected, focused, accent, false)
        }),
        disabled: Box::new(move |theme| button_appearance(theme, selected, false, accent, false)),
        hovered: Box::new(move |focused, theme| {
            button_appearance(theme, selected, focused, accent, true)
        }),
        pressed: Box::new(move |focused, theme| {
            button_appearance(theme, selected, focused, accent, false)
        }),
    }
}
