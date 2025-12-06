//! Newsletter detection and email analysis

use super::models::{SenderInfo, UnsubscribeMethod};
use regex::Regex;
use std::sync::OnceLock;

/// Parse List-Unsubscribe header to extract HTTP URLs
/// 
/// Format: `<http://example.com/unsub>, <mailto:unsub@example.com>`
pub fn parse_list_unsubscribe(header: &str) -> Vec<String> {
    static URL_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = URL_REGEX.get_or_init(|| {
        Regex::new(r"<(https?://[^>]+)>").expect("Invalid regex")
    });
    
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
/// - List-Unsubscribe header present: +0.4
/// - Message count > 5: +0.3
/// - Message count > 20: +0.5 (additional)
pub fn calculate_heuristic_score(
    email: &str,
    has_unsubscribe: bool,
    message_count: usize,
) -> f32 {
    let mut score = 0.0;
    
    // Email pattern matching
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
    
    // List-Unsubscribe header
    if has_unsubscribe {
        score += 0.4;
    }
    
    // Message count
    if message_count > 5 {
        score += 0.3;
    }
    if message_count > 20 {
        score += 0.5;
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
    // Parse unsubscribe URLs
    let unsubscribe_urls = list_unsubscribe
        .as_ref()
        .map(|h| parse_list_unsubscribe(h))
        .unwrap_or_default();
    
    let has_one_click = detect_one_click(list_unsubscribe_post.as_deref());
    
    // Determine unsubscribe method
    let unsubscribe_method = if has_one_click && !unsubscribe_urls.is_empty() {
        UnsubscribeMethod::OneClick {
            url: unsubscribe_urls[0].clone(),
        }
    } else if !unsubscribe_urls.is_empty() {
        UnsubscribeMethod::HttpLink {
            url: unsubscribe_urls[0].clone(),
        }
    } else if let Some(ref header) = list_unsubscribe {
        // Check for mailto
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
    let heuristic_score = calculate_heuristic_score(
        &email,
        list_unsubscribe.is_some(),
        message_count,
    );
    
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
        let score = calculate_heuristic_score("newsletter@example.com", true, 25);
        assert!(score > 1.0);
        
        // Regular email
        let score = calculate_heuristic_score("john@example.com", false, 2);
        assert!(score < 0.5);
    }
}
