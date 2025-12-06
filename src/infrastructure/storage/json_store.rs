//! Account metadata storage

use crate::domain::models::EmailAccount;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

/// Get accounts directory path
fn accounts_dir() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "unsubmail", "unsubmail")
        .context("Failed to get project directories")?;

    let dir = proj_dirs.config_dir().join("accounts");

    fs::create_dir_all(&dir).context("Failed to create accounts directory")?;

    Ok(dir)
}

/// Get account file path
fn account_path(email: &str) -> Result<PathBuf> {
    let dir = accounts_dir()?;
    let filename = format!("{}.json", sanitize_email(email));
    Ok(dir.join(filename))
}

/// Sanitize email for filename
fn sanitize_email(email: &str) -> String {
    email.replace('@', "_at_").replace('.', "_")
}

/// Save account metadata
pub fn save_account(account: &EmailAccount) -> Result<()> {
    let path = account_path(&account.email)?;
    let json = serde_json::to_string_pretty(account).context("Failed to serialize account")?;

    fs::write(&path, json).context("Failed to write account file")?;

    Ok(())
}

/// Load account metadata
pub fn load_account(email: &str) -> Result<Option<EmailAccount>> {
    let path = account_path(email)?;

    if !path.exists() {
        return Ok(None);
    }

    let json = fs::read_to_string(&path).context("Failed to read account file")?;

    let account = serde_json::from_str(&json).context("Failed to deserialize account")?;

    Ok(Some(account))
}

/// List all accounts
pub fn list_accounts() -> Result<Vec<EmailAccount>> {
    let dir = accounts_dir()?;

    let mut accounts = Vec::new();

    for entry in fs::read_dir(&dir).context("Failed to read accounts directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let json = fs::read_to_string(&path)?;
            if let Ok(account) = serde_json::from_str::<EmailAccount>(&json) {
                accounts.push(account);
            }
        }
    }

    Ok(accounts)
}

/// Remove account metadata
pub fn remove_account(email: &str) -> Result<()> {
    let path = account_path(email)?;

    if path.exists() {
        fs::remove_file(&path).context("Failed to remove account file")?;
    }

    Ok(())
}
