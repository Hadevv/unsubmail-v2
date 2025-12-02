//! Core domain models
//!
//! Data structures representing the business domain.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Represents a Gmail account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAccount {
    pub email: String,
    pub display_name: Option<String>,
    pub added_at: DateTime<Utc>,
    pub last_scanned: Option<DateTime<Utc>>,
}

/// Information about an email sender
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderInfo {
    pub email: String,
    pub display_name: Option<String>,
    pub message_count: usize,
    pub has_unsubscribe: bool,
    pub has_one_click: bool,
    pub unsubscribe_url: Option<String>,
    pub unsubscribe_post_url: Option<String>,
    pub score: f32,
    pub sample_message_ids: Vec<String>,
}

impl SenderInfo {
    pub fn new(email: String) -> Self {
        Self {
            email,
            display_name: None,
            message_count: 0,
            has_unsubscribe: false,
            has_one_click: false,
            unsubscribe_url: None,
            unsubscribe_post_url: None,
            score: 0.0,
            sample_message_ids: Vec::new(),
        }
    }
}

/// Represents an email message (headers only)
#[derive(Debug, Clone)]
pub struct MessageHeaders {
    pub id: String,
    pub from: String,
    pub subject: Option<String>,
    pub list_unsubscribe: Option<String>,
    pub list_unsubscribe_post: Option<String>,
    pub date: Option<DateTime<Utc>>,
}

/// Result of a cleanup action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub sender_email: String,
    pub unsubscribed: bool,
    pub blocked: bool,
    pub messages_deleted: usize,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sender_info_creation() {
        let sender = SenderInfo::new("test@example.com".to_string());
        assert_eq!(sender.email, "test@example.com");
        assert_eq!(sender.message_count, 0);
    }

    #[test]
    fn test_email_account_serialization() {
        let account = EmailAccount {
            email: "user@gmail.com".to_string(),
            display_name: Some("Test User".to_string()),
            added_at: Utc::now(),
            last_scanned: None,
        };
        let json = serde_json::to_string(&account).unwrap();
        assert!(json.contains("user@gmail.com"));
    }
}
