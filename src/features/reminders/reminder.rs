use std::collections::HashSet;

use jiff::Timestamp;
use notify_rust::Timeout;
use uuid::Uuid;

use crate::shared::store::Store;

#[derive(Debug, Clone)]
pub enum ReminderMessage {
    Tick,
}

pub fn check_and_notify(
    store: &Store,
    now: Timestamp,
    window_start: Timestamp,
    sent: &HashSet<(Uuid, i64)>,
) -> Vec<(Uuid, i64)> {
    let mut notified = Vec::new();

    let lists = match store.lists().load_all() {
        Ok(lists) => lists,
        Err(err) => {
            tracing::error!("reminder: failed to load lists: {err}");
            return notified;
        }
    };

    for list in lists {
        let tasks = match store.tasks(list.id).load_all() {
            Ok(tasks) => tasks,
            Err(err) => {
                tracing::error!("reminder: failed to load tasks for list {}: {err}", list.id);
                continue;
            }
        };

        for task in tasks {
            let Some(reminder) = task.reminder_date else {
                continue;
            };

            let key = (task.id, reminder.as_second());

            if sent.contains(&key) || reminder < window_start || reminder > now {
                continue;
            }

            let result = notify_rust::Notification::new()
                .summary("Task Reminder")
                .body(&task.title)
                .icon("dev.edfloreshz.Tasks")
                .timeout(Timeout::Milliseconds(5000))
                .show();

            match result {
                Ok(_) => {
                    tracing::info!(
                        "reminder: sent notification for task \"{}\" ({})",
                        task.title,
                        task.id
                    );
                    notified.push(key);
                }
                Err(err) => {
                    tracing::error!(
                        "reminder: failed to send notification for task \"{}\": {err}",
                        task.title
                    );
                }
            }
        }
    }

    notified
}
