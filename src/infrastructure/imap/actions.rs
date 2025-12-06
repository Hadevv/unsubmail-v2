//! IMAP actions (delete, move to spam)

use super::connection::ImapSession;
use anyhow::{Context, Result};
use futures::TryStreamExt;

/// Delete messages by UIDs using Gmail's trash label
pub async fn delete_messages(session: &mut ImapSession, uids: &[u32]) -> Result<usize> {
    if uids.is_empty() {
        return Ok(0);
    }

    let uid_set = format_uid_set(uids);
    let count = uids.len();

    // Ensure INBOX is selected (critical for IMAP operations)
    session
        .select("INBOX")
        .await
        .context("Failed to select INBOX")?;

    // Move messages to Gmail's Trash folder (more reliable than \Deleted flag)
    session
        .uid_copy(&uid_set, "[Gmail]/Trash")
        .await
        .context("Failed to move messages to trash")?;

    // Mark as deleted in INBOX
    let _: Vec<_> = session
        .uid_store(&uid_set, "+FLAGS.SILENT (\\Deleted)")
        .await
        .context("Failed to mark messages as deleted")?
        .try_collect()
        .await?;

    // Expunge to permanently remove from INBOX
    let _: Vec<_> = session
        .expunge()
        .await
        .context("Failed to expunge deleted messages")?
        .try_collect()
        .await?;

    Ok(count)
}

/// Move messages to spam folder
pub async fn move_to_spam(session: &mut ImapSession, uids: &[u32]) -> Result<usize> {
    if uids.is_empty() {
        return Ok(0);
    }

    let uid_set = format_uid_set(uids);
    let count = uids.len();

    // Ensure INBOX is selected
    session
        .select("INBOX")
        .await
        .context("Failed to select INBOX")?;

    // Copy messages to Gmail's Spam folder
    session
        .uid_copy(&uid_set, "[Gmail]/Spam")
        .await
        .context("Failed to copy messages to spam")?;

    // Mark as deleted in INBOX
    let _: Vec<_> = session
        .uid_store(&uid_set, "+FLAGS.SILENT (\\Deleted)")
        .await
        .context("Failed to mark messages as deleted")?
        .try_collect()
        .await?;

    // Expunge to remove from INBOX
    let _: Vec<_> = session
        .expunge()
        .await
        .context("Failed to expunge deleted messages")?
        .try_collect()
        .await?;

    Ok(count)
}

/// Format UIDs for IMAP command
fn format_uid_set(uids: &[u32]) -> String {
    if uids.is_empty() {
        return String::new();
    }
    
    if uids.len() == 1 {
        return uids[0].to_string();
    }
    
    // Check if consecutive
    let is_consecutive = uids.windows(2).all(|w| w[1] == w[0] + 1);
    
    if is_consecutive {
        format!("{}:{}", uids[0], uids[uids.len() - 1])
    } else {
        uids.iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}
