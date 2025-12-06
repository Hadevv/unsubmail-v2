//! IMAP actions (delete, move to spam)

use super::connection::ImapSession;
use anyhow::{Context, Result};
use futures::TryStreamExt;

/// Delete messages by UIDs
pub async fn delete_messages(session: &mut ImapSession, uids: &[u32]) -> Result<usize> {
    if uids.is_empty() {
        return Ok(0);
    }

    let uid_set = format_uid_set(uids);
    
    // Mark as deleted and consume stream
    {
        let _: Vec<_> = session
            .uid_store(&uid_set, "+FLAGS.SILENT (\\Deleted)")
            .await
            .context("Failed to mark messages as deleted")?
            .try_collect()
            .await?;
    }
    
    // Expunge to permanently delete and consume stream
    {
        let _: Vec<_> = session
            .expunge()
            .await
            .context("Failed to expunge deleted messages")?
            .try_collect()
            .await?;
    }
    
    Ok(uids.len())
}

/// Move messages to spam folder
pub async fn move_to_spam(session: &mut ImapSession, uids: &[u32]) -> Result<usize> {
    if uids.is_empty() {
        return Ok(0);
    }

    let uid_set = format_uid_set(uids);
    
    // Gmail uses [Gmail]/Spam
    {
        session
            .uid_copy(&uid_set, "[Gmail]/Spam")
            .await
            .context("Failed to copy messages to spam")?;
            
        // uid_copy returns Result<()>, not a stream
    }
    
    // Delete from inbox (inline to ensure no borrow issues)
    {
        let _: Vec<_> = session
            .uid_store(&uid_set, "+FLAGS.SILENT (\\Deleted)")
            .await
            .context("Failed to mark messages as deleted")?
            .try_collect()
            .await?;
    }
    
    {
        let _: Vec<_> = session
            .expunge()
            .await
            .context("Failed to expunge deleted messages")?
            .try_collect()
            .await?;
    }
    
    Ok(uids.len())
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
