use crate::features::{lists::list::List, tasks::task::Task};

pub trait Markdown {
    fn markdown(&self) -> String;
}

impl Markdown for List {
    fn markdown(&self) -> String {
        format!("# {}\n", self.name)
    }
}

impl Markdown for Task {
    fn markdown(&self) -> String {
        format!(
            "- [{}] {}\n",
            if self.is_completed() { "x" } else { " " },
            self.title
        )
    }
}
