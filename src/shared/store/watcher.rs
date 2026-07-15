use std::path::PathBuf;
use std::time::Duration;

use cosmic::iced::futures::channel::mpsc::Sender;
use cosmic::iced::futures::{SinkExt, StreamExt};
use cosmic::iced::{stream, Subscription};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::app::Message;
use crate::shared::navigation::nav::TasksAction;

const DEBOUNCE: Duration = Duration::from_millis(300);

/// Watches the store directory for changes made from other windows or
/// processes and emits a debounced sync message once the tree settles.
pub fn subscription(base_dir: PathBuf) -> Subscription<Message> {
    Subscription::run_with(base_dir.clone(), move |base_dir| {
        let base_dir = base_dir.clone();
        stream::channel(10, move |mut output: Sender<Message>| async move {
            let (tx, mut rx) = cosmic::iced::futures::channel::mpsc::channel(100);

            let mut watcher = match RecommendedWatcher::new(
                move |res: notify::Result<notify::Event>| {
                    if res.is_ok() {
                        let _ = tx.clone().try_send(());
                    }
                },
                notify::Config::default(),
            ) {
                Ok(watcher) => watcher,
                Err(err) => {
                    tracing::error!("Failed to create file watcher: {err}");
                    return;
                }
            };

            if let Err(err) = watcher.watch(&base_dir, RecursiveMode::Recursive) {
                tracing::error!("Failed to watch store directory: {err}");
                return;
            }

            loop {
                if rx.next().await.is_none() {
                    break;
                }

                while tokio::time::timeout(DEBOUNCE, rx.next())
                    .await
                    .is_ok_and(|event| event.is_some())
                {}

                if output
                    .send(Message::Tasks(TasksAction::SyncFromDisk))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        })
    })
}
