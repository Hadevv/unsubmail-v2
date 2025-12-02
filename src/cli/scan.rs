//! Email scanning CLI commands
//!
//! Handles scanning Gmail inbox and analyzing senders.

use anyhow::Result;
use crate::domain::models::{SenderInfo, EmailAccount};
use crate::infrastructure::google::gmail_api::GmailClient;
use crate::domain::analysis;
use std::io::{self, Write};

/// Scan inbox and return analyzed senders
pub async fn scan_inbox(
    client: &GmailClient,
    account: &EmailAccount,
    max_messages: usize,
) -> Result<Vec<SenderInfo>> {
    use indicatif::{ProgressBar, ProgressStyle};

    let user_id = &account.email;
    let mut all_message_ids = Vec::new();
    let mut page_token: Option<String> = None;

    // Fetch message IDs with pagination
    println!("Fetching message list...");
    loop {
        let (ids, next_token) = client.list_message_ids(user_id, 500, page_token).await?;
        all_message_ids.extend(ids);

        if all_message_ids.len() >= max_messages || next_token.is_none() {
            break;
        }

        page_token = next_token;
    }

    let total = all_message_ids.len().min(max_messages);
    all_message_ids.truncate(total);

    println!("Fetching headers for {} messages...", total);

    // Progress bar
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-")
    );

    // Fetch headers in batches
    let batch_size = 50;
    let mut all_headers = Vec::new();

    for chunk in all_message_ids.chunks(batch_size) {
        let headers = client.batch_get_headers(user_id, chunk).await?;
        all_headers.extend(headers);
        pb.inc(chunk.len() as u64);
    }

    pb.finish_with_message("Done!");

    // Analyze and group by sender
    let senders = analysis::analyze_messages(all_headers);

    Ok(senders)
}

/// Display scan progress
#[allow(dead_code)]
pub fn display_scan_progress(current: usize, total: usize) {
    print!("\rScanning: {}/{}", current, total);
    io::stdout().flush().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_operations() {
        // TODO: Add unit tests
    }
}
