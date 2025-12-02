//! Sender selection CLI interface
//!
//! Provides interactive TUI for selecting senders to clean.

use anyhow::Result;
use crate::domain::models::SenderInfo;
use dialoguer::MultiSelect;

/// Present senders to user and return selected ones
pub fn select_senders(senders: &[SenderInfo]) -> Result<Vec<usize>> {
    todo!("Display interactive checkbox list using dialoguer")
}

/// Format sender for display in selection list
pub fn format_sender_display(sender: &SenderInfo) -> String {
    todo!("Format: 'Email (count messages) [unsubscribe: yes/no]'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_sender() {
        // TODO: Add unit tests
    }
}
