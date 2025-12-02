//! Email scanning CLI commands
//!
//! Handles scanning Gmail inbox and analyzing senders.

use anyhow::Result;
use crate::domain::models::{SenderInfo, EmailAccount};
use crate::infrastructure::google::gmail_api::GmailClient;

/// Scan inbox and return analyzed senders
pub async fn scan_inbox(
    client: &GmailClient,
    account: &EmailAccount,
    max_messages: usize,
) -> Result<Vec<SenderInfo>> {
    todo!("Fetch messages via Gmail API, analyze headers, return unique senders")
}

/// Display scan progress
pub fn display_scan_progress(current: usize, total: usize) {
    todo!("Show progress bar using indicatif")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_operations() {
        // TODO: Add unit tests
    }
}
