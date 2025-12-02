//! Action planning logic
//!
//! Determines the optimal cleanup strategy for each sender.

use crate::domain::models::SenderInfo;

/// Action plan for cleaning up a sender
#[derive(Debug, Clone)]
pub struct ActionPlan {
    pub sender_email: String,
    pub should_unsubscribe: bool,
    pub should_block: bool,
    pub should_delete: bool,
}

/// Generate action plan for a sender
pub fn plan_cleanup(sender: &SenderInfo) -> ActionPlan {
    let should_unsubscribe = sender.has_one_click && sender.unsubscribe_post_url.is_some();
    let should_block = !should_unsubscribe; // Block if we can't unsubscribe
    let should_delete = true; // Always delete after unsubscribe/block

    ActionPlan {
        sender_email: sender.email.clone(),
        should_unsubscribe,
        should_block,
        should_delete,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_with_one_click() {
        let mut sender = SenderInfo::new("test@example.com".to_string());
        sender.has_one_click = true;
        sender.unsubscribe_post_url = Some("https://example.com/unsub".to_string());

        let plan = plan_cleanup(&sender);
        assert!(plan.should_unsubscribe);
        assert!(!plan.should_block);
        assert!(plan.should_delete);
    }

    #[test]
    fn test_plan_without_unsubscribe() {
        let sender = SenderInfo::new("test@example.com".to_string());

        let plan = plan_cleanup(&sender);
        assert!(!plan.should_unsubscribe);
        assert!(plan.should_block);
        assert!(plan.should_delete);
    }
}
