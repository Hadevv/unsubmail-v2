//! Newsletter detection and email analysis

use super::models::{SenderInfo, UnsubscribeMethod};
use regex::Regex;
use std::sync::OnceLock;

/// Parse List-Unsubscribe header to extract HTTP URLs
///
/// Format: `<http://example.com/unsub>, <mailto:unsub@example.com>`
pub fn parse_list_unsubscribe(header: &str) -> Vec<String> {
    static URL_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = URL_REGEX.get_or_init(|| Regex::new(r"<(https?://[^>]+)>").expect("Invalid regex"));

    regex
        .captures_iter(header)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

/// Detect one-click unsubscribe from List-Unsubscribe-Post header
///
/// Format: `List-Unsubscribe=One-Click`
pub fn detect_one_click(header: Option<&str>) -> bool {
    header
        .map(|h| h.to_lowercase().contains("one-click"))
        .unwrap_or(false)
}

/// Calculate heuristic score for newsletter detection
///
/// Scoring:
/// - Email patterns (newsletter@, noreply@, etc.): +0.3
/// - List-Unsubscribe header present: +0.5 (strong signal)
/// - Message count > 10: +0.2
/// - Message count > 30: +0.3 (additional)
///
/// Note: Without List-Unsubscribe header, max score is capped at 0.5 to prevent
/// false positives on personal emails with high message counts.
pub fn calculate_heuristic_score(email: &str, has_unsubscribe: bool, message_count: usize) -> f32 {
    let mut score = 0.0;

    // List-Unsubscribe header is the strongest signal
    if has_unsubscribe {
        score += 0.5;
    }

    // Email pattern matching (secondary signal)
    let email_lower = email.to_lowercase();
    let newsletter_patterns = [
        "newsletter",
        "noreply",
        "no-reply",
        "notification",
        "promo",
        "marketing",
        "news@",
        "info@",
        "updates@",
    ];

    if newsletter_patterns.iter().any(|p| email_lower.contains(p)) {
        score += 0.3;
    }

    // Message count (use higher thresholds to avoid personal emails)
    if message_count > 10 {
        score += 0.2;
    }
    if message_count > 30 {
        score += 0.3;
    }

    // Cap score at 0.5 if no List-Unsubscribe header
    // This prevents personal emails from appearing even with high message counts
    if !has_unsubscribe && score > 0.5 {
        score = 0.5;
    }

    score
}

/// Analyze sender to determine unsubscribe method
pub fn analyze_sender(
    email: String,
    display_name: Option<String>,
    message_count: usize,
    message_uids: Vec<u32>,
    list_unsubscribe: Option<String>,
    list_unsubscribe_post: Option<String>,
    sample_subjects: Vec<String>,
) -> SenderInfo {
    // Parse unsubscribe URLs from List-Unsubscribe header
    let unsubscribe_urls = list_unsubscribe
        .as_ref()
        .map(|h| parse_list_unsubscribe(h))
        .unwrap_or_default();

    // Check for one-click unsubscribe support
    let has_one_click = detect_one_click(list_unsubscribe_post.as_deref());

    // Determine unsubscribe method based on available headers
    // Priority: OneClick > HttpLink > Mailto > None
    let unsubscribe_method = if has_one_click {
        // RFC 8058: One-click unsubscribe requires both headers
        if !unsubscribe_urls.is_empty() {
            UnsubscribeMethod::OneClick {
                url: unsubscribe_urls[0].clone(),
            }
        } else {
            // Invalid state: has one-click flag but no URL
            // This shouldn't happen with compliant senders
            UnsubscribeMethod::None
        }
    } else if !unsubscribe_urls.is_empty() {
        // Standard HTTP unsubscribe link (requires manual click)
        UnsubscribeMethod::HttpLink {
            url: unsubscribe_urls[0].clone(),
        }
    } else if let Some(ref header) = list_unsubscribe {
        // Check for mailto-only unsubscribe
        if header.contains("mailto:") {
            let mailto = header
                .split('<')
                .find(|s| s.contains("mailto:"))
                .and_then(|s| s.split('>').next())
                .unwrap_or("")
                .replace("mailto:", "");
            UnsubscribeMethod::Mailto { address: mailto }
        } else {
            UnsubscribeMethod::None
        }
    } else {
        UnsubscribeMethod::None
    };

    // Calculate heuristic score
    let heuristic_score =
        calculate_heuristic_score(&email, list_unsubscribe.is_some(), message_count);

    SenderInfo {
        email,
        display_name,
        message_count,
        message_uids,
        unsubscribe_method,
        heuristic_score,
        sample_subjects,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_list_unsubscribe() {
        let header = "<https://example.com/unsub?id=123>, <mailto:unsub@example.com>";
        let urls = parse_list_unsubscribe(header);
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://example.com/unsub?id=123");
    }

    #[test]
    fn test_detect_one_click() {
        assert!(detect_one_click(Some("List-Unsubscribe=One-Click")));
        assert!(!detect_one_click(Some("something else")));
        assert!(!detect_one_click(None));
    }

    #[test]
    fn test_heuristic_score() {
        // Newsletter email with unsubscribe and many messages
        // Expected: 0.5 (List-Unsubscribe) + 0.3 (pattern) + 0.2 (>10) + 0.3 (>30) = 1.3
        let score = calculate_heuristic_score("newsletter@example.com", true, 35);
        assert!(
            score > 1.0,
            "Newsletter with unsubscribe should score > 1.0, got {}",
            score
        );

        // Regular email without List-Unsubscribe but high message count
        // Expected: capped at 0.5 (no List-Unsubscribe)
        let score = calculate_heuristic_score("john@example.com", false, 50);
        assert_eq!(
            score, 0.5,
            "Personal email without unsubscribe should be capped at 0.5"
        );

        // Regular email with low message count
        // Expected: 0.0
        let score = calculate_heuristic_score("jane@example.com", false, 2);
        assert_eq!(score, 0.0, "Low-volume personal email should score 0.0");

        // Marketing email with List-Unsubscribe
        // Expected: 0.5 (List-Unsubscribe) + 0.3 (pattern) = 0.8
        let score = calculate_heuristic_score("marketing@example.com", true, 5);
        assert!(
            score >= 0.8,
            "Marketing email with unsubscribe should score >= 0.8, got {}",
            score
        );
    }
}
