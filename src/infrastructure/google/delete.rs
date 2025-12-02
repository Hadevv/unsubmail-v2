//! Gmail message deletion
//!
//! Handles batch deletion of messages.

use anyhow::{Context, Result};
use google_gmail1::{Gmail, api::BatchDeleteMessagesRequest};
use crate::infrastructure::google::auth::Auth;

const BATCH_SIZE: usize = 1000; // Gmail API limit

/// Message deleter
pub struct MessageDeleter {
    hub: Gmail<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>,
}

impl MessageDeleter {
    /// Create new message deleter
    pub fn new(auth: Auth) -> Self {
        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .unwrap()
            .https_or_http()
            .enable_http1()
            .build();

        let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
            .build(connector);

        let hub = Gmail::new(client, auth);

        Self { hub }
    }

    /// Delete messages in batches
    pub async fn batch_delete(&self, user_id: &str, message_ids: &[String]) -> Result<usize> {
        let mut deleted = 0;

        for chunk in message_ids.chunks(BATCH_SIZE) {
            let request = BatchDeleteMessagesRequest {
                ids: Some(chunk.to_vec()),
            };

            self.hub
                .users()
                .messages_batch_delete(request, user_id)
                .doit()
                .await
                .context("Failed to batch delete messages")?;

            deleted += chunk.len();
            tracing::info!("Deleted {} messages", chunk.len());
        }

        Ok(deleted)
    }

    /// Delete all messages from a specific sender
    pub async fn delete_from_sender(&self, user_id: &str, message_ids: &[String]) -> Result<usize> {
        self.batch_delete(user_id, message_ids).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_size() {
        assert_eq!(BATCH_SIZE, 1000);
    }
}
