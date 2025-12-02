//! Email analysis logic
//!
//! Heuristics for detecting newsletters and unsubscribe capabilities.

use crate::domain::models::{MessageHeaders, SenderInfo};
use std::collections::HashMap;

/// Analyze multiple message headers and group by sender
pub fn analyze_messages(messages: Vec<MessageHeaders>) -> Vec<SenderInfo> {
    let mut senders: HashMap<String, SenderInfo> = HashMap::new();

    for msg in messages {
        let sender_email = extract_email(&msg.from);
        let sender = senders.entry(sender_email.clone()).or_insert_with(|| {
            SenderInfo::new(sender_email)
        });

        sender.message_count += 1;
        sender.sample_message_ids.push(msg.id.clone());

        // Extract display name from first message
        if sender.display_name.is_none() {
            sender.display_name = extract_display_name(&msg.from);
        }

        // Check for unsubscribe headers
        if let Some(unsub) = &msg.list_unsubscribe {
            sender.has_unsubscribe = true;
            sender.unsubscribe_url = Some(unsub.clone());
        }

        if let Some(unsub_post) = &msg.list_unsubscribe_post {
            sender.has_one_click = is_one_click_unsubscribe(unsub_post);
            sender.unsubscribe_post_url = Some(unsub_post.clone());
        }

        // Calculate heuristic score
        sender.score = calculate_newsletter_score(sender, &msg);
    }

    let mut result: Vec<SenderInfo> = senders.into_values().collect();
    result.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    result
}

/// Extract email address from "Name <email@domain.com>" format
fn extract_email(from: &str) -> String {
    if let Some(start) = from.find('<') {
        if let Some(end) = from.find('>') {
            return from[start + 1..end].to_string();
        }
    }
    from.trim().to_string()
}

/// Extract display name from "Name <email@domain.com>" format
fn extract_display_name(from: &str) -> Option<String> {
    if let Some(pos) = from.find('<') {
        let name = from[..pos].trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

/// Check if unsubscribe is one-click capable
fn is_one_click_unsubscribe(header: &str) -> bool {
    header.to_lowercase().contains("one-click")
}

/// Calculate newsletter probability score (0.0 - 1.0)
fn calculate_newsletter_score(sender: &SenderInfo, msg: &MessageHeaders) -> f32 {
    let mut score = 0.0;

    // High message count increases score
    if sender.message_count > 5 {
        score += 0.3;
    }
    if sender.message_count > 20 {
        score += 0.2;
    }

    // Unsubscribe header is strong signal
    if sender.has_unsubscribe {
        score += 0.4;
    }

    // Newsletter keywords in email address
    let email_lower = sender.email.to_lowercase();
    let newsletter_keywords = ["newsletter", "noreply", "no-reply", "notification", "promo", "marketing"];
    for keyword in newsletter_keywords {
        if email_lower.contains(keyword) {
            score += 0.2;
            break;
        }
    }

    // Subject line patterns
    if let Some(subject) = &msg.subject {
        let subject_lower = subject.to_lowercase();
        if subject_lower.contains("unsubscribe") || subject_lower.contains("newsletter") {
            score += 0.1;
        }
    }

    score.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_email() {
        assert_eq!(extract_email("John Doe <john@example.com>"), "john@example.com");
        assert_eq!(extract_email("john@example.com"), "john@example.com");
    }

    #[test]
    fn test_extract_display_name() {
        assert_eq!(extract_display_name("John Doe <john@example.com>"), Some("John Doe".to_string()));
        assert_eq!(extract_display_name("john@example.com"), None);
    }

    #[test]
    fn test_is_one_click() {
        assert!(is_one_click_unsubscribe("List-Unsubscribe=One-Click"));
        assert!(!is_one_click_unsubscribe("mailto:unsub@example.com"));
    }

    #[test]
    fn test_newsletter_score() {
        let mut sender = SenderInfo::new("newsletter@example.com".to_string());
        sender.message_count = 10;
        sender.has_unsubscribe = true;

        let msg = MessageHeaders {
            id: "1".to_string(),
            from: "newsletter@example.com".to_string(),
            subject: Some("Weekly Newsletter".to_string()),
            list_unsubscribe: Some("https://example.com/unsub".to_string()),
            list_unsubscribe_post: None,
            date: None,
        };

        let score = calculate_newsletter_score(&sender, &msg);
        assert!(score > 0.5);
    }
}
