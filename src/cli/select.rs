//! Sender selection CLI interface
//!
//! Provides interactive TUI for selecting senders to clean.

use anyhow::Result;
use crate::domain::models::SenderInfo;
use dialoguer::MultiSelect;

/// Present senders to user and return selected ones
pub fn select_senders(senders: &[SenderInfo]) -> Result<Vec<usize>> {
    if senders.is_empty() {
        return Ok(Vec::new());
    }

    let items: Vec<String> = senders
        .iter()
        .map(|s| format_sender_display(s))
        .collect();

    let selections = MultiSelect::new()
        .with_prompt("Select senders to clean (Space to select, Enter to confirm)")
        .items(&items)
        .interact()?;

    Ok(selections)
}

/// Format sender for display in selection list
pub fn format_sender_display(sender: &SenderInfo) -> String {
    let unsub_status = if sender.has_one_click {
        "one-click unsubscribe"
    } else if sender.has_unsubscribe {
        "manual unsubscribe"
    } else {
        "no unsubscribe"
    };

    format!(
        "{} ({} msgs) [{}] - score: {:.2}",
        sender.display_name.as_ref().unwrap_or(&sender.email),
        sender.message_count,
        unsub_status,
        sender.score
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_sender() {
        // TODO: Add unit tests
    }
}
