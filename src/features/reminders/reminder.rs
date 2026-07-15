use std::collections::HashSet;

use jiff::Timestamp;
use notify_rust::Timeout;
use uuid::Uuid;

use crate::shared::store::Store;

/// Messages emitted by the reminder subscription.
#[derive(Debug, Clone)]
pub enum ReminderMessage {
    /// A timer tick: check all tasks for due reminders.
    Tick,
}

/// Scan every task across all lists and fire a desktop notification for each
/// one whose `reminder_date` falls within `[window_start, now]`.  Returns the
/// IDs of tasks for which a notification was successfully sent so the caller
/// can record them in `sent_reminders` and avoid duplicates.
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

            // Skip tasks whose reminder has already been sent or is outside
            // the current window.
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
