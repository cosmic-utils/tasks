use core_done::models::priority::Priority;
use core_done::models::task::Task;
use cosmic::{Element, widget};
use cosmic::widget::segmented_button;
use cosmic::widget::segmented_button::Entity;

pub struct Details {
    pub task: Option<Task>,
    pub priority_model: segmented_button::Model<segmented_button::SingleSelect>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Rename(String),
    Delete(String),
    Complete(bool),
    Favorite(bool),
    PriorityActivate(Entity),
}

pub enum Command {
    Update(Task),
    Rename(String, String),
    Delete(String),
    Complete(String, bool),
    Favorite(String, bool),
    PriorityActivate(String, Priority),
}

impl Details {
    pub fn new() -> Self {
        let priority_model = segmented_button::ModelBuilder::default()
            .insert(|entity| {
                entity
                    .icon(widget::icon(
                        widget::icon::from_name("cosmic-applet-battery-level-10-symbolic").handle(),
                    ))
                    .data(Priority::Low)
            })
            .insert(|entity| {
                entity
                    .icon(widget::icon(
                        widget::icon::from_name("cosmic-applet-battery-level-50-symbolic").handle(),
                    ))
                    .data(Priority::Normal)
            })
            .insert(|entity| {
                entity
                    .icon(widget::icon(
                        widget::icon::from_name("cosmic-applet-battery-level-100-symbolic")
                            .handle(),
                    ))
                    .data(Priority::High)
            })
            .build();

        Self {
            task: None,
            priority_model,
        }
    }

    pub fn update(&mut self, message: Message) -> Vec<Command> {
        let mut commands = vec![];
        match message {
            Message::Rename(title) => {
                if let Some(ref mut task) = &mut self.task {
                    task.title = title.clone();
                    commands.push(Command::Rename(task.id.clone(), title));
                }
            }
            Message::Delete(_) => {}
            Message::Complete(_) => {}
            Message::Favorite(favorite) => {
                if let Some(ref mut task) = &mut self.task {
                    task.favorite = favorite;
                    commands.push(Command::Favorite(task.id.clone(), favorite));
                }
            }
            Message::PriorityActivate(entity) => {
                self.priority_model.activate(entity);
                let priority = self.priority_model.data::<Priority>(entity);
                if let Some(task) = &self.task {
                    if let Some(priority) = priority {
                        commands.push(Command::PriorityActivate(task.id.clone(), priority.clone()));
                    }
                }
            }
        }
        commands
    }

    pub fn view(&self) -> Element<Message> {
        if let Some(task) = self.task.as_ref().clone() {
            return widget::settings::view_column(vec![widget::settings::view_section("Details")
                .add(
                    widget::container(widget::text_input("Title", &task.title).on_input(|value| {
                        Message::Rename(value)
                    }))
                        .padding([0, 10, 0, 10]),
                )
                .add(
                    widget::settings::item::builder("Favorite").control(widget::checkbox(
                        "",
                        task.favorite,
                        |value| Message::Favorite(value),
                    )),
                )
                .add(
                    widget::settings::item::builder("Priority").control(
                        widget::segmented_control::horizontal(&self.priority_model)
                            .on_activate(Message::PriorityActivate),
                    ),
                )
                .into()])
                .into();
        }
        widget::settings::view_column(vec![widget::settings::view_section("Details").into()]).into()
    }
}