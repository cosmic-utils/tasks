use cosmic::{Element, widget};
use cosmic::iced::Length;

#[derive(Debug, Clone)]
pub enum Message {
    List(List),
    ItemDown,
    ItemUp,
}

#[derive(Debug, Clone)]
pub struct List {
    pub title: String,
}

pub struct Content {
    list: Option<List>,
}

pub enum Command {}

impl Content {
    pub fn update(&mut self, message: Message) -> Vec<Command> {
        let commands = Vec::new();
        match message {
            Message::List(list) => {
                self.list = Some(list);
            }
            Message::ItemDown => {}
            Message::ItemUp => {}
        }
        commands
    }

    pub fn view(&self) -> Element<Message> {
        widget::container(widget::column::with_children(vec![]))
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }
}