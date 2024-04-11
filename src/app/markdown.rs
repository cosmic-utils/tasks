use done_core::models::list::List;
use done_core::models::status::Status;
use done_core::models::task::Task;

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
        let mut task = format!(
            "- [{}] {}\n",
            if self.status == Status::Completed {
                "x"
            } else {
                " "
            },
            self.title
        );
        let sub_tasks = self.sub_tasks.iter().fold(String::new(), |acc, sub_task| {
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
