//! Secure token storage using system keyring
//!
//! Stores OAuth2 tokens securely in the OS keyring.

use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE_NAME: &str = "unsubmail";

/// Keyring-based token storage
pub struct KeyringStore;

impl KeyringStore {
    /// Save token for an account
    pub fn save_token(email: &str, token: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, email)
            .context("Failed to create keyring entry")?;

        entry.set_password(token)
            .context("Failed to save token to keyring")?;

        Ok(())
    }

    /// Get token for an account
    pub fn get_token(email: &str) -> Result<String> {
        let entry = Entry::new(SERVICE_NAME, email)
            .context("Failed to create keyring entry")?;

        entry.get_password()
            .context("Failed to get token from keyring")
    }

    /// Delete token for an account
    pub fn delete_token(email: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, email)
            .context("Failed to create keyring entry")?;

        entry.delete_credential()
            .context("Failed to delete token from keyring")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_name() {
        assert_eq!(SERVICE_NAME, "unsubmail");
    }
}
