//! Main application workflow orchestration
//!
//! Coordinates all operations: add account, scan, select, cleanup.

use anyhow::{Context, Result};
use crate::domain::models::{EmailAccount, SenderInfo, CleanupResult};
use crate::domain::analysis;
use crate::domain::planner;
use crate::infrastructure::google::auth::GoogleAuthenticator;
use crate::infrastructure::google::gmail_api::GmailClient;
use crate::infrastructure::google::filters::FilterManager;
use crate::infrastructure::google::delete::MessageDeleter;
use crate::infrastructure::storage::json_store::AccountStore;
use crate::infrastructure::network::http_client::HttpClient;
use crate::cli;
use chrono::Utc;
use std::path::PathBuf;

/// Main workflow orchestrator
pub struct Workflow {
    authenticator: GoogleAuthenticator,
    account_store: AccountStore,
    http_client: HttpClient,
}

impl Workflow {
    /// Create new workflow
    pub fn new(config_dir: PathBuf) -> Result<Self> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID")
            .context("GOOGLE_CLIENT_ID environment variable not set")?;
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .context("GOOGLE_CLIENT_SECRET environment variable not set")?;

        let token_dir = config_dir.join("tokens");
        let accounts_dir = config_dir.join("accounts");

        let authenticator = GoogleAuthenticator::new(client_id, client_secret, token_dir);
        let account_store = AccountStore::new(accounts_dir)?;
        let http_client = HttpClient::new()?;

        Ok(Self {
            authenticator,
            account_store,
            http_client,
        })
    }

    /// Add new account workflow
    pub async fn add_account(&self) -> Result<String> {
        tracing::info!("Starting add account workflow");

        // Prompt for email
        let email = dialoguer::Input::<String>::new()
            .with_prompt("Enter Gmail address")
            .interact_text()
            .context("Failed to get email input")?;

        // Check if already exists
        if self.account_store.account_exists(&email) {
            anyhow::bail!("Account {} already exists", email);
        }

        // Run OAuth2 flow
        println!("Opening browser for authentication...");
        let _auth = self.authenticator.authenticate(&email).await?;

        // Save account metadata
        let account = EmailAccount {
            email: email.clone(),
            display_name: None,
            added_at: Utc::now(),
            last_scanned: None,
        };

        self.account_store.save_account(&account)?;

        tracing::info!("Account {} added successfully", email);
        Ok(email)
    }

    /// Scan account workflow
    pub async fn scan_account(&self, email: &str) -> Result<Vec<SenderInfo>> {
        tracing::info!("Starting scan workflow for {}", email);

        // Load account
        let mut account = self.account_store.load_account(email)?;

        // Get authenticator
        let auth = self.authenticator.get_authenticator(email).await?;
        let gmail_client = GmailClient::new(auth);

        // Scan inbox
        println!("Scanning inbox...");
        let senders = cli::scan::scan_inbox(&gmail_client, &account, 2000).await?;

        // Update last scanned
        account.last_scanned = Some(Utc::now());
        self.account_store.save_account(&account)?;

        tracing::info!("Scan completed. Found {} unique senders", senders.len());
        Ok(senders)
    }

    /// Cleanup workflow
    pub async fn cleanup_account(&self, email: &str, sender_indices: Vec<usize>, all_senders: &[SenderInfo]) -> Result<Vec<CleanupResult>> {
        tracing::info!("Starting cleanup workflow for {}", email);

        let auth = self.authenticator.get_authenticator(email).await?;
        let gmail_client = GmailClient::new(auth.clone());
        let filter_manager = FilterManager::new(auth.clone());
        let deleter = MessageDeleter::new(auth);

        let mut results = Vec::new();

        for idx in sender_indices {
            let sender = &all_senders[idx];
            println!("\nProcessing: {} ({} messages)", sender.email, sender.message_count);

            let result = self.cleanup_sender(
                sender,
                &gmail_client,
                &filter_manager,
                &deleter,
                email,
            ).await;

            results.push(result);
        }

        Ok(results)
    }

    /// Cleanup single sender
    async fn cleanup_sender(
        &self,
        sender: &SenderInfo,
        gmail_client: &GmailClient,
        filter_manager: &FilterManager,
        deleter: &MessageDeleter,
        user_id: &str,
    ) -> CleanupResult {
        let plan = planner::plan_cleanup(sender);
        let mut result = CleanupResult {
            sender_email: sender.email.clone(),
            unsubscribed: false,
            blocked: false,
            messages_deleted: 0,
            error: None,
        };

        // Try unsubscribe
        if plan.should_unsubscribe {
            if let Some(url) = &sender.unsubscribe_post_url {
                if HttpClient::is_safe_url(url) {
                    match self.http_client.one_click_unsubscribe(url).await {
                        Ok(success) => {
                            result.unsubscribed = success;
                            if success {
                                println!("  ✓ Unsubscribed");
                            } else {
                                println!("  ✗ Unsubscribe failed, will block instead");
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Unsubscribe failed: {}", e);
                            println!("  ✗ Unsubscribe error: {}", e);
                        }
                    }
                }
            }
        }

        // Block if needed
        if plan.should_block && !result.unsubscribed {
            match filter_manager.create_trash_filter(user_id, &sender.email).await {
                Ok(_) => {
                    result.blocked = true;
                    println!("  ✓ Blocked via filter");
                }
                Err(e) => {
                    tracing::error!("Failed to create filter: {}", e);
                    result.error = Some(format!("Filter creation failed: {}", e));
                }
            }
        }

        // Delete messages
        if plan.should_delete {
            let confirm = dialoguer::Confirm::new()
                .with_prompt(format!("Delete {} messages from {}?", sender.message_count, sender.email))
                .default(true)
                .interact()
                .unwrap_or(false);

            if confirm {
                match deleter.delete_from_sender(user_id, &sender.sample_message_ids).await {
                    Ok(count) => {
                        result.messages_deleted = count;
                        println!("  ✓ Deleted {} messages", count);
                    }
                    Err(e) => {
                        tracing::error!("Failed to delete messages: {}", e);
                        result.error = Some(format!("Deletion failed: {}", e));
                    }
                }
            }
        }

        result
    }

    /// List accounts workflow
    pub async fn list_accounts(&self) -> Result<Vec<EmailAccount>> {
        self.account_store.list_accounts()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_creation() {
        // TODO: Add unit tests with mock dependencies
    }
}
