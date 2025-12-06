//! Integration tests for domain models
//!
//! Tests model behavior and validation logic.

use chrono::Utc;
use unsubmail::domain::models::{
    EmailAccount, OAuth2Token, UnsubscribeMethod, CleanupResult, ActionType,
};

#[test]
fn test_email_account_creation() {
    let account = EmailAccount {
        email: "test@gmail.com".to_string(),
        added_at: Utc::now(),
    };

    assert_eq!(account.email, "test@gmail.com");
}

#[test]
fn test_oauth2_token_expired() {
    let past = Utc::now() - chrono::Duration::hours(2);
    let token = OAuth2Token {
        access_token: "token123".to_string(),
        refresh_token: "refresh123".to_string(),
        expires_at: past,
    };

    assert!(token.is_expired());
}

#[test]
fn test_oauth2_token_not_expired() {
    let future = Utc::now() + chrono::Duration::hours(2);
    let token = OAuth2Token {
        access_token: "token123".to_string(),
        refresh_token: "refresh123".to_string(),
        expires_at: future,
    };

    assert!(!token.is_expired());
}

#[test]
fn test_unsubscribe_method_is_one_click() {
    let one_click = UnsubscribeMethod::OneClick {
        url: "https://example.com".to_string(),
    };
    assert!(one_click.is_one_click());

    let http = UnsubscribeMethod::HttpLink {
        url: "https://example.com".to_string(),
    };
    assert!(!http.is_one_click());
}

#[test]
fn test_unsubscribe_method_is_available() {
    let one_click = UnsubscribeMethod::OneClick {
        url: "https://example.com".to_string(),
    };
    assert!(one_click.is_available());

    let http = UnsubscribeMethod::HttpLink {
        url: "https://example.com".to_string(),
    };
    assert!(http.is_available());

    let mailto = UnsubscribeMethod::Mailto {
        address: "unsub@example.com".to_string(),
    };
    assert!(mailto.is_available());

    let none = UnsubscribeMethod::None;
    assert!(!none.is_available());
}

#[test]
fn test_cleanup_result_success() {
    let result = CleanupResult::success(
        "newsletter@example.com".to_string(),
        ActionType::UnsubscribeAndDelete,
        42,
        Some(true),
    );

    assert_eq!(result.sender_email, "newsletter@example.com");
    assert_eq!(result.messages_deleted, 42);
    assert_eq!(result.unsubscribe_success, Some(true));
    assert!(result.error.is_none());
}

#[test]
fn test_cleanup_result_failure() {
    let result = CleanupResult::failure(
        "spam@example.com".to_string(),
        ActionType::SpamAndDelete,
        "Network timeout".to_string(),
    );

    assert_eq!(result.sender_email, "spam@example.com");
    assert_eq!(result.messages_deleted, 0);
    assert_eq!(result.error, Some("Network timeout".to_string()));
}

#[test]
fn test_unsubscribe_method_variants() {
    // Test OneClick variant
    let one_click = UnsubscribeMethod::OneClick {
        url: "https://example.com/unsub".to_string(),
    };
    match one_click {
        UnsubscribeMethod::OneClick { url } => {
            assert_eq!(url, "https://example.com/unsub");
        }
        _ => panic!("Expected OneClick variant"),
    }

    // Test HttpLink variant
    let http = UnsubscribeMethod::HttpLink {
        url: "https://example.com/unsubscribe".to_string(),
    };
    match http {
        UnsubscribeMethod::HttpLink { url } => {
            assert_eq!(url, "https://example.com/unsubscribe");
        }
        _ => panic!("Expected HttpLink variant"),
    }

    // Test Mailto variant
    let mailto = UnsubscribeMethod::Mailto {
        address: "unsub@example.com".to_string(),
    };
    match mailto {
        UnsubscribeMethod::Mailto { address } => {
            assert_eq!(address, "unsub@example.com");
        }
        _ => panic!("Expected Mailto variant"),
    }

    // Test None variant
    let none = UnsubscribeMethod::None;
    assert!(matches!(none, UnsubscribeMethod::None));
}
