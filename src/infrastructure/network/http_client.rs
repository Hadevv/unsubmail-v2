//! HTTP client for one-click unsubscribe

use anyhow::{bail, Context, Result};
use reqwest::Client;
use std::time::Duration;
use url::Url;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Perform one-click unsubscribe via HTTP POST
/// 
/// Security: Only HTTPS URLs are allowed
pub async fn unsubscribe_one_click(url: &str) -> Result<bool> {
    // Validate URL
    let parsed_url = Url::parse(url).context("Invalid unsubscribe URL")?;
    
    // Security: Only HTTPS
    if parsed_url.scheme() != "https" {
        bail!("Only HTTPS unsubscribe URLs are allowed");
    }
    
    // Create HTTP client
    let client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .context("Failed to create HTTP client")?;
    
    // Send POST request
    let response = client
        .post(url)
        .header("List-Unsubscribe", "One-Click")
        .send()
        .await
        .context("Failed to send unsubscribe request")?;
    
    // Check if successful
    Ok(response.status().is_success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reject_http() {
        let result = unsubscribe_one_click("http://example.com/unsub").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reject_invalid_url() {
        let result = unsubscribe_one_click("not-a-url").await;
        assert!(result.is_err());
    }
}
