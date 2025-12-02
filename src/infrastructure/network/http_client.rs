//! HTTP client for unsubscribe requests
//!
//! Handles one-click unsubscribe HTTP POST requests.

use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

/// HTTP client for unsubscribe operations
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    /// Create new HTTP client
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("unsubmail/0.1.0")
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Execute one-click unsubscribe POST request
    pub async fn one_click_unsubscribe(&self, url: &str) -> Result<bool> {
        let response = self.client
            .post(url)
            .header("List-Unsubscribe", "One-Click")
            .send()
            .await
            .context("Failed to send unsubscribe request")?;

        let success = response.status().is_success();

        if success {
            tracing::info!("Successfully unsubscribed via one-click: {}", url);
        } else {
            tracing::warn!("Unsubscribe request failed with status: {}", response.status());
        }

        Ok(success)
    }

    /// Validate unsubscribe URL (basic safety check)
    pub fn is_safe_url(url: &str) -> bool {
        // Only allow HTTPS URLs
        if !url.starts_with("https://") {
            return false;
        }

        // Reject mailto: links
        if url.contains("mailto:") {
            return false;
        }

        // TODO: Add more safety checks if needed
        true
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_url_validation() {
        assert!(HttpClient::is_safe_url("https://example.com/unsubscribe"));
        assert!(!HttpClient::is_safe_url("http://example.com/unsubscribe"));
        assert!(!HttpClient::is_safe_url("mailto:unsubscribe@example.com"));
    }

    #[test]
    fn test_http_client_creation() {
        let client = HttpClient::new();
        assert!(client.is_ok());
    }
}
