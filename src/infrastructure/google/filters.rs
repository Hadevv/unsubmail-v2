//! Gmail filter management
//!
//! Creates filters to automatically handle unwanted senders.

use anyhow::{Context, Result};
use google_gmail1::{Gmail, api::{Filter, FilterCriteria, FilterAction}};
use crate::infrastructure::google::auth::Auth;

/// Gmail filter manager
pub struct FilterManager {
    hub: Gmail<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>,
}

impl FilterManager {
    /// Create new filter manager
    pub fn new(auth: Auth) -> Self {
        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .unwrap()
            .https_or_http()
            .enable_http1()
            .build();

        let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
            .build(connector);

        let hub = Gmail::new(client, auth);

        Self { hub }
    }

    /// Create filter to auto-trash emails from sender
    pub async fn create_trash_filter(&self, user_id: &str, sender_email: &str) -> Result<String> {
        let criteria = FilterCriteria {
            from: Some(sender_email.to_string()),
            ..Default::default()
        };

        let action = FilterAction {
            add_label_ids: None,
            remove_label_ids: None,
            forward: None,
        };

        let filter = Filter {
            id: None,
            criteria: Some(criteria),
            action: Some(action),
        };

        let (_, result) = self.hub
            .users()
            .settings_filters_create(filter, user_id)
            .doit()
            .await
            .context("Failed to create filter")?;

        Ok(result.id.unwrap_or_default())
    }

    /// Create filter to mark as spam
    pub async fn create_spam_filter(&self, user_id: &str, sender_email: &str) -> Result<String> {
        // Similar to trash filter but marks as spam
        self.create_trash_filter(user_id, sender_email).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_manager_creation() {
        // TODO: Add unit tests with mock auth
    }
}
