use crate::core::models::{List, Status, Task};

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

        // Recursively format sub-tasks with proper indentation
        if !self.sub_tasks.is_empty() {
            task.push_str(&format_sub_tasks(&self.sub_tasks, 1));
        }

        task
    }
}

// Helper function to recursively format sub-tasks with proper indentation
fn format_sub_tasks(sub_tasks: &[Task], indent_level: usize) -> String {
    let mut result = String::new();
    let indent = "  ".repeat(indent_level);

    for sub_task in sub_tasks {
        // Add the sub-task with proper indentation
        result.push_str(&format!(
            "{}- [{}] {}\n",
            indent,
            if sub_task.status == Status::Completed {
                "x"
            } else {
                " "
            },
            sub_task.title
        ));

        // Recursively process nested sub-tasks if any
        if !sub_task.sub_tasks.is_empty() {
            result.push_str(&format_sub_tasks(&sub_task.sub_tasks, indent_level + 1));
        }
    }

    result
}
