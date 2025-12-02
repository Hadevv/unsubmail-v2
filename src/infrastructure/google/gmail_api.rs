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
        // Gmail API uses "me" as special identifier for authenticated user
        let user_id = if user_id.contains('@') { "me" } else { user_id };

        tracing::debug!("Listing messages for user_id: {}, max_results: {}", user_id, max_results);

        let mut req = self.hub.users().messages_list(user_id)
            .max_results(max_results);

        if let Some(token) = page_token {
            req = req.page_token(&token);
        }

        let (_, result) = req.doit().await
            .with_context(|| format!("Failed to list messages for user: {}", user_id))?;

        let ids = result.messages
            .unwrap_or_default()
            .into_iter()
            .filter_map(|m| m.id)
            .collect();

        tracing::debug!("Found {} message IDs", ids.len());

        Ok((ids, result.next_page_token))
    }

    /// Get message headers only
    #[allow(dead_code)]
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

    /// Batch get message headers (parallel fetching with rate limiting)
    pub async fn batch_get_headers(&self, user_id: &str, message_ids: &[String]) -> Result<Vec<MessageHeaders>> {
        use futures::future::join_all;
        use std::sync::Arc;
        use tokio::sync::Semaphore;

        // Limit concurrent requests to avoid rate limits (max 10 concurrent)
        let semaphore = Arc::new(Semaphore::new(10));

        // Fetch headers in parallel with concurrency limit
        let tasks: Vec<_> = message_ids
            .iter()
            .map(|id| {
                let user_id = user_id.to_string();
                let message_id = id.clone();
                let hub = self.hub.clone();
                let semaphore = semaphore.clone();

                tokio::spawn(async move {
                    // Acquire semaphore permit
                    let _permit = semaphore.acquire().await.ok()?;

                    // Exponential backoff retry logic
                    let mut retries = 0;
                    let max_retries = 3;
                    let mut delay_ms = 100;

                    loop {
                        // Fetch headers
                        let result = hub
                            .users()
                            .messages_get(&user_id, &message_id)
                            .format("metadata")
                            .add_metadata_headers("From")
                            .add_metadata_headers("Subject")
                            .add_metadata_headers("List-Unsubscribe")
                            .add_metadata_headers("List-Unsubscribe-Post")
                            .add_metadata_headers("Date")
                            .doit()
                            .await;

                        match result {
                            Ok((_, message)) => return Some(parse_message_headers(message)),
                            Err(e) => {
                                // Check if rate limited or transient error
                                let should_retry = retries < max_retries
                                    && (e.to_string().contains("429")
                                        || e.to_string().contains("503")
                                        || e.to_string().contains("timeout"));

                                if should_retry {
                                    tracing::debug!("Retrying {} after {}ms (attempt {})", message_id, delay_ms, retries + 1);
                                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                                    delay_ms *= 2; // Exponential backoff
                                    retries += 1;
                                } else {
                                    tracing::warn!("Failed to get headers for {}: {}", message_id, e);
                                    return None;
                                }
                            }
                        }
                    }
                })
            })
            .collect();

        // Wait for all tasks to complete
        let results = join_all(tasks).await;

        // Collect successful results
        let headers: Vec<MessageHeaders> = results
            .into_iter()
            .filter_map(|result| result.ok().flatten())
            .collect();

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

/// Parse email date header (RFC 2822 format)
fn parse_email_date(date_str: &str) -> Option<DateTime<Utc>> {
    use mailparse::dateparse;

    // Use mailparse to parse RFC 2822 date
    match dateparse(date_str) {
        Ok(timestamp) => {
            // Convert Unix timestamp to DateTime<Utc>
            DateTime::from_timestamp(timestamp, 0)
        }
        Err(e) => {
            tracing::debug!("Failed to parse date '{}': {}", date_str, e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gmail_client_creation() {
        // TODO: Add unit tests with mock authenticator
    }
}
