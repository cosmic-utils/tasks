use core_done::models::list::List;
use cosmic::{Element, widget};
use cosmic::iced::{Length, Subscription};

#[derive(Debug, Clone)]
pub struct ListView {
    pub(crate) list: List,
}

impl ListView {
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Rename(String),
    Delete,
    List(ListView),
    ItemDown,
    ItemUp,
}

pub struct Content {
    list: Option<List>,
}

pub enum Command {}

impl Content {
    pub fn update(&mut self, message: Message) -> Vec<Command> {
        let commands = Vec::new();
        match message {
            Message::List(list_view) => {
                self.list = Some(list_view.list);
            }
            Message::ItemDown => {}
            Message::ItemUp => {}
            Message::Rename(_) => {}
            Message::Delete => {}
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