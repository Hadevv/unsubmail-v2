//! Action planning logic

use super::models::{ActionType, CleanupAction, SenderInfo};

/// Plan cleanup action for a sender
/// 
/// Strategy:
/// 1. If one-click unsubscribe available → UnsubscribeAndDelete
/// 2. Otherwise → SpamAndDelete
pub fn plan_action(sender: SenderInfo) -> CleanupAction {
    let action_type = if sender.unsubscribe_method.is_one_click() {
        ActionType::UnsubscribeAndDelete
    } else {
        ActionType::SpamAndDelete
    };
    
    CleanupAction {
        sender,
        action_type,
    }
}

/// Plan actions for multiple senders
pub fn plan_actions(senders: Vec<SenderInfo>) -> Vec<CleanupAction> {
    senders.into_iter().map(plan_action).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::UnsubscribeMethod;

    #[test]
    fn test_plan_action_one_click() {
        let sender = SenderInfo {
            email: "news@example.com".to_string(),
            display_name: Some("Example News".to_string()),
            message_count: 10,
            message_uids: vec![1, 2, 3],
            unsubscribe_method: UnsubscribeMethod::OneClick {
                url: "https://example.com/unsub".to_string(),
            },
            heuristic_score: 0.8,
            sample_subjects: vec![],
        };
        
        let action = plan_action(sender);
        assert_eq!(action.action_type, ActionType::UnsubscribeAndDelete);
    }

    #[test]
    fn test_plan_action_no_unsubscribe() {
        let sender = SenderInfo {
            email: "spam@example.com".to_string(),
            display_name: None,
            message_count: 5,
            message_uids: vec![1, 2],
            unsubscribe_method: UnsubscribeMethod::None,
            heuristic_score: 0.3,
            sample_subjects: vec![],
        };
        
        let action = plan_action(sender);
        assert_eq!(action.action_type, ActionType::SpamAndDelete);
    }
}
