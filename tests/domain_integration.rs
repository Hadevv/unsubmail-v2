//! Integration tests for domain layer
//!
//! These tests verify that domain logic works correctly in isolation.

use unsubmail::domain::analysis::{
    calculate_heuristic_score, parse_list_unsubscribe, detect_one_click, analyze_sender,
};
use unsubmail::domain::models::UnsubscribeMethod;
use unsubmail::domain::planner::{plan_action, plan_actions};

#[test]
fn test_parse_list_unsubscribe_with_multiple_urls() {
    let header = "<https://example.com/unsub?id=123>, <mailto:unsub@example.com>";
    let urls = parse_list_unsubscribe(header);

    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0], "https://example.com/unsub?id=123");
}

#[test]
fn test_parse_list_unsubscribe_no_urls() {
    let header = "<mailto:unsub@example.com>";
    let urls = parse_list_unsubscribe(header);

    assert!(urls.is_empty());
}

#[test]
fn test_detect_one_click_positive() {
    assert!(detect_one_click(Some("List-Unsubscribe=One-Click")));
    assert!(detect_one_click(Some("list-unsubscribe=one-click"))); // case insensitive
}

#[test]
fn test_detect_one_click_negative() {
    assert!(!detect_one_click(Some("List-Unsubscribe=Manual")));
    assert!(!detect_one_click(None));
}

#[test]
fn test_heuristic_score_newsletter_with_unsubscribe() {
    // Newsletter with List-Unsubscribe and high message count
    let score = calculate_heuristic_score("newsletter@example.com", true, 35);

    // Should get: 0.5 (unsubscribe) + 0.3 (pattern) + 0.2 (>10) + 0.3 (>30) = 1.3
    assert!(score > 1.0, "Expected score > 1.0, got {}", score);
}

#[test]
fn test_heuristic_score_personal_email_capped() {
    // Personal email with high message count but no List-Unsubscribe
    let score = calculate_heuristic_score("john.doe@example.com", false, 100);

    // Should be capped at 0.5 without List-Unsubscribe header
    assert_eq!(score, 0.5, "Personal email should be capped at 0.5");
}

#[test]
fn test_heuristic_score_low_volume_personal() {
    let score = calculate_heuristic_score("jane@example.com", false, 3);

    // Low volume, no patterns, no unsubscribe = 0.0
    assert_eq!(score, 0.0);
}

#[test]
fn test_analyze_sender_with_one_click() {
    let sender = analyze_sender(
        "news@example.com".to_string(),
        Some("Example News".to_string()),
        25,
        vec![1, 2, 3],
        Some("<https://example.com/unsub>".to_string()),
        Some("List-Unsubscribe=One-Click".to_string()),
        vec!["Subject 1".to_string(), "Subject 2".to_string()],
    );

    assert_eq!(sender.email, "news@example.com");
    assert_eq!(sender.message_count, 25);
    assert!(sender.unsubscribe_method.is_one_click());
    assert!(sender.heuristic_score > 0.5);
}

#[test]
fn test_analyze_sender_with_http_link() {
    let sender = analyze_sender(
        "promo@example.com".to_string(),
        None,
        10,
        vec![1, 2],
        Some("<https://example.com/unsubscribe>".to_string()),
        None, // No one-click
        vec![],
    );

    match &sender.unsubscribe_method {
        UnsubscribeMethod::HttpLink { url } => {
            assert_eq!(url, "https://example.com/unsubscribe");
        }
        _ => panic!("Expected HttpLink method"),
    }
}

#[test]
fn test_analyze_sender_mailto_only() {
    let sender = analyze_sender(
        "updates@example.com".to_string(),
        None,
        5,
        vec![1],
        Some("<mailto:unsub@example.com>".to_string()),
        None,
        vec![],
    );

    match &sender.unsubscribe_method {
        UnsubscribeMethod::Mailto { address } => {
            assert_eq!(address, "unsub@example.com");
        }
        _ => panic!("Expected Mailto method"),
    }
}

#[test]
fn test_plan_action_for_one_click() {
    let sender = analyze_sender(
        "news@example.com".to_string(),
        None,
        10,
        vec![1, 2],
        Some("<https://example.com/unsub>".to_string()),
        Some("List-Unsubscribe=One-Click".to_string()),
        vec![],
    );

    let action = plan_action(sender);

    assert_eq!(
        action.action_type,
        unsubmail::domain::models::ActionType::UnsubscribeAndDelete
    );
}

#[test]
fn test_plan_action_for_no_unsubscribe() {
    let sender = analyze_sender(
        "spam@example.com".to_string(),
        None,
        5,
        vec![1, 2],
        None,
        None,
        vec![],
    );

    let action = plan_action(sender);

    assert_eq!(
        action.action_type,
        unsubmail::domain::models::ActionType::SpamAndDelete
    );
}

#[test]
fn test_plan_actions_multiple_senders() {
    let sender1 = analyze_sender(
        "news@example.com".to_string(),
        None,
        10,
        vec![1],
        Some("<https://example.com/unsub>".to_string()),
        Some("List-Unsubscribe=One-Click".to_string()),
        vec![],
    );

    let sender2 = analyze_sender(
        "spam@example.com".to_string(),
        None,
        5,
        vec![2],
        None,
        None,
        vec![],
    );

    let actions = plan_actions(vec![sender1, sender2]);

    assert_eq!(actions.len(), 2);
    assert_eq!(
        actions[0].action_type,
        unsubmail::domain::models::ActionType::UnsubscribeAndDelete
    );
    assert_eq!(
        actions[1].action_type,
        unsubmail::domain::models::ActionType::SpamAndDelete
    );
}

#[test]
fn test_email_pattern_matching() {
    // Test various newsletter patterns
    let patterns = vec![
        ("newsletter@example.com", true),
        ("noreply@example.com", true),
        ("no-reply@example.com", true),
        ("notification@example.com", true),
        ("promo@example.com", true),
        ("marketing@example.com", true),
        ("news@example.com", true),
        ("info@example.com", true),
        ("updates@example.com", true),
        ("john.doe@example.com", false), // Personal email
        ("support@example.com", false),  // Support email
    ];

    for (email, should_match) in patterns {
        let score = calculate_heuristic_score(email, false, 5);
        if should_match {
            assert!(
                score >= 0.3,
                "Email '{}' should match pattern (score >= 0.3), got {}",
                email, score
            );
        } else {
            assert!(
                score < 0.3,
                "Email '{}' should not match pattern (score < 0.3), got {}",
                email, score
            );
        }
    }
}
