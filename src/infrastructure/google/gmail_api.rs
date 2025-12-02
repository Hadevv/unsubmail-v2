//! Gmail API client
//!
//! Handles all Gmail API operations.

use anyhow::{Context, Result};
use google_gmail1::{Gmail, api::Message};
use crate::domain::models::MessageHeaders;
use crate::infrastructure::google::auth::Auth;
use chrono::{DateTime, Utc};

/// Gmail API client
pub struct GmailClient {
    hub: Gmail<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>,
}

impl GmailClient {
    /// Create new Gmail client
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

    /// List message IDs with pagination
    pub async fn list_message_ids(&self, user_id: &str, max_results: u32, page_token: Option<String>) -> Result<(Vec<String>, Option<String>)> {
        let mut req = self.hub.users().messages_list(user_id)
            .max_results(max_results);

        if let Some(token) = page_token {
            req = req.page_token(&token);
        }

        let (_, result) = req.doit().await
            .context("Failed to list messages")?;

        let ids = result.messages
            .unwrap_or_default()
            .into_iter()
            .filter_map(|m| m.id)
            .collect();

        Ok((ids, result.next_page_token))
    }

    /// Get message headers only
    pub async fn get_message_headers(&self, user_id: &str, message_id: &str) -> Result<MessageHeaders> {
        let (_, message) = self.hub
            .users()
            .messages_get(user_id, message_id)
            .format("metadata")
            .add_metadata_headers("From")
            .add_metadata_headers("Subject")
            .add_metadata_headers("List-Unsubscribe")
            .add_metadata_headers("List-Unsubscribe-Post")
            .add_metadata_headers("Date")
            .doit()
            .await
            .context("Failed to get message")?;

        Ok(parse_message_headers(message))
    }

    /// Batch get message headers
    pub async fn batch_get_headers(&self, user_id: &str, message_ids: &[String]) -> Result<Vec<MessageHeaders>> {
        let mut headers = Vec::new();

        // TODO: Use batch API for better performance
        for id in message_ids {
            match self.get_message_headers(user_id, id).await {
                Ok(h) => headers.push(h),
                Err(e) => tracing::warn!("Failed to get headers for {}: {}", id, e),
            }
        }

        Ok(headers)
    }
}

/// Parse Gmail API message into MessageHeaders
fn parse_message_headers(message: Message) -> MessageHeaders {
    let id = message.id.unwrap_or_default();
    let mut from = String::new();
    let mut subject = None;
    let mut list_unsubscribe = None;
    let mut list_unsubscribe_post = None;
    let mut date = None;

    if let Some(payload) = message.payload {
        if let Some(headers) = payload.headers {
            for header in headers {
                let name = header.name.unwrap_or_default();
                let value = header.value.unwrap_or_default();

                match name.to_lowercase().as_str() {
                    "from" => from = value,
                    "subject" => subject = Some(value),
                    "list-unsubscribe" => list_unsubscribe = Some(value),
                    "list-unsubscribe-post" => list_unsubscribe_post = Some(value),
                    "date" => {
                        // Parse date if needed
                        date = parse_email_date(&value);
                    }
                    _ => {}
                }
            }
        }
    }

    MessageHeaders {
        id,
        from,
        subject,
        list_unsubscribe,
        list_unsubscribe_post,
        date,
    }
}

/// Parse email date header
fn parse_email_date(_date_str: &str) -> Option<DateTime<Utc>> {
    // TODO: Implement proper RFC 2822 date parsing
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gmail_client_creation() {
        // TODO: Add unit tests with mock authenticator
    }
}
