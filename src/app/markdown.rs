use cosmic::Application;

use crate::core::{
    models::{List, Status, Task},
    storage::LocalStorage,
};

use super::Tasks;

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
        let storage = LocalStorage::new(Tasks::APP_ID).expect("Failed to initialize local storage");

        let mut task = format!(
            "- [{}] {}\n",
            if self.status == Status::Completed {
                "x"
            } else {
                " "
            },
            self.title
        );

        let sub_tasks = storage
            .get_sub_tasks(&self.parent, &self.id)
            .unwrap_or_default();
        let sub_tasks = sub_tasks.iter().fold(String::new(), |acc, sub_task| {
            format!(
                "{}  - [{}] {}\n",
                acc,
                if sub_task.status == Status::Completed {
                    "x"
                } else {
                    " "
                },
                sub_task.title
            )
        });
        task.push_str(&sub_tasks);
        task
    }
}
