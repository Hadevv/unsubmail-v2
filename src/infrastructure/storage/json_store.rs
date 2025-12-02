//! JSON-based account metadata storage
//!
//! Stores account metadata in local JSON files.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use crate::domain::models::EmailAccount;

/// JSON file-based account store
pub struct AccountStore {
    store_path: PathBuf,
}

impl AccountStore {
    /// Create new account store
    pub fn new(store_path: PathBuf) -> Result<Self> {
        if !store_path.exists() {
            fs::create_dir_all(&store_path)
                .context("Failed to create store directory")?;
        }
        Ok(Self { store_path })
    }

    /// Save account metadata
    pub fn save_account(&self, account: &EmailAccount) -> Result<()> {
        let file_path = self.account_file_path(&account.email);
        let json = serde_json::to_string_pretty(account)
            .context("Failed to serialize account")?;

        fs::write(&file_path, json)
            .context("Failed to write account file")?;

        Ok(())
    }

    /// Load account metadata
    pub fn load_account(&self, email: &str) -> Result<EmailAccount> {
        let file_path = self.account_file_path(email);
        let json = fs::read_to_string(&file_path)
            .context("Failed to read account file")?;

        serde_json::from_str(&json)
            .context("Failed to deserialize account")
    }

    /// List all accounts
    pub fn list_accounts(&self) -> Result<Vec<EmailAccount>> {
        let mut accounts = Vec::new();

        for entry in fs::read_dir(&self.store_path)
            .context("Failed to read store directory")?
        {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(json) = fs::read_to_string(&path) {
                    if let Ok(account) = serde_json::from_str::<EmailAccount>(&json) {
                        accounts.push(account);
                    }
                }
            }
        }

        Ok(accounts)
    }

    /// Delete account metadata
    pub fn delete_account(&self, email: &str) -> Result<()> {
        let file_path = self.account_file_path(email);
        if file_path.exists() {
            fs::remove_file(&file_path)
                .context("Failed to delete account file")?;
        }
        Ok(())
    }

    /// Check if account exists
    pub fn account_exists(&self, email: &str) -> bool {
        self.account_file_path(email).exists()
    }

    fn account_file_path(&self, email: &str) -> PathBuf {
        let safe_name = email.replace(['@', '.'], "_");
        self.store_path.join(format!("{}.json", safe_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_account_file_path() {
        let store = AccountStore::new(PathBuf::from("/tmp/test")).unwrap();
        let path = store.account_file_path("user@gmail.com");
        assert!(path.to_string_lossy().contains("user_gmail_com.json"));
    }
}
