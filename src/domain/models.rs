//! Core data structures for UnsubMail

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Email account metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAccount {
    /// Email address
    pub email: String,

    /// When the account was added
    pub added_at: DateTime<Utc>,
}

/// Information about a unique sender
#[derive(Debug, Clone)]
pub struct SenderInfo {
    /// Sender email address
    pub email: String,

    /// Display name (if available)
    pub display_name: Option<String>,

    /// Number of messages from this sender
    pub message_count: usize,

    /// Message UIDs from this sender
    pub message_uids: Vec<u32>,

    /// Unsubscribe method available
    pub unsubscribe_method: UnsubscribeMethod,

    /// Heuristic score (0.0 - 1.0+)
    pub heuristic_score: f32,

    /// Sample subject lines
    pub sample_subjects: Vec<String>,
}

/// Unsubscribe method
#[derive(Debug, Clone, PartialEq)]
pub enum UnsubscribeMethod {
    /// One-click HTTP POST unsubscribe
    OneClick { url: String },

    /// HTTP link (requires manual click)
    HttpLink { url: String },

    /// Mailto link (not supported)
    Mailto { address: String },

    /// No unsubscribe method found
    None,
}

impl UnsubscribeMethod {
    /// Check if one-click unsubscribe is available
    pub fn is_one_click(&self) -> bool {
        matches!(self, UnsubscribeMethod::OneClick { .. })
    }

    /// Check if any unsubscribe method is available
    pub fn is_available(&self) -> bool {
        !matches!(self, UnsubscribeMethod::None)
    }
}

/// Planned cleanup action for a sender
#[derive(Debug, Clone)]
pub struct CleanupAction {
    /// Sender being cleaned
    pub sender: SenderInfo,

    /// Action to take
    pub action_type: ActionType,
}

/// Type of cleanup action
#[derive(Debug, Clone, PartialEq)]

pub enum ActionType {
    /// Unsubscribe via one-click, then delete
    UnsubscribeAndDelete,

    /// Move to spam, then delete
    SpamAndDelete,

    /// Just delete (user choice)
    DeleteOnly,
}

/// Result of a cleanup operation
#[derive(Debug, Clone)]
pub struct CleanupResult {
    /// Sender email
    pub sender_email: String,

    /// Action taken
    pub action: ActionType,

    /// Number of messages deleted
    pub messages_deleted: usize,

    /// Whether unsubscribe succeeded (if attempted)
    pub unsubscribe_success: Option<bool>,

    /// Error message if any
    pub error: Option<String>,
}

impl CleanupResult {
    /// Create a successful result
    pub fn success(
        sender_email: String,
        action: ActionType,
        messages_deleted: usize,
        unsubscribe_success: Option<bool>,
    ) -> Self {
        Self {
            sender_email,
            action,
            messages_deleted,
            unsubscribe_success,
            error: None,
        }
    }

    /// Create a failed result
    pub fn failure(sender_email: String, action: ActionType, error: String) -> Self {
        Self {
            sender_email,
            action,
            messages_deleted: 0,
            unsubscribe_success: None,
            error: Some(error),
        }
    }
}

/// OAuth2 token storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Token {
    /// Access token
    pub access_token: String,

    /// Refresh token
    pub refresh_token: String,

    /// Token expiry time
    pub expires_at: DateTime<Utc>,
}

impl OAuth2Token {
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }
}
