use cosmic::iced::futures::channel::mpsc::Sender;
use cosmic::iced::futures::{SinkExt, StreamExt};
use cosmic::iced::{stream, Subscription};

use crate::app::Message;

use super::AccountsAction;

/// Listens for account-added/removed/changed signals from accounts-daemon
/// (e.g. the user disabling Todo, or disconnecting an account, from the
/// Accounts app) and asks Tasks to reload its account list whenever one
/// fires, so a toggle flipped elsewhere is reflected here promptly instead
/// of waiting for the next periodic sync.
pub fn subscription() -> Subscription<Message> {
    Subscription::run(|| {
        stream::channel(10, |mut output: Sender<Message>| async move {
            let client = match accounts::AccountsClient::new().await {
                Ok(client) => client,
                Err(err) => {
                    tracing::debug!("accounts signal subscription: daemon unreachable: {err}");
                    return;
                }
            };

            let (mut added, mut removed, mut changed) = match (
                client.receive_account_added().await,
                client.receive_account_removed().await,
                client.receive_account_changed().await,
            ) {
                (Ok(added), Ok(removed), Ok(changed)) => (added, removed, changed),
                _ => {
                    tracing::error!("accounts signal subscription: failed to subscribe to signals");
                    return;
                }
            };

            loop {
                let more = tokio::select! {
                    item = added.next() => item.is_some(),
                    item = removed.next() => item.is_some(),
                    item = changed.next() => item.is_some(),
                };
                if !more {
                    break;
                }
                if output
                    .send(Message::Accounts(AccountsAction::AccountsChanged))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        })
    })
}
