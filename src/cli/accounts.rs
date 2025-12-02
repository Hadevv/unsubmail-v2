//! Account management CLI commands
//!
//! Handles adding, listing, and removing Gmail accounts via OAuth2.

use anyhow::Result;
use crate::infrastructure::google::auth::GoogleAuthenticator;
use crate::infrastructure::storage::json_store::AccountStore;

/// Add a new Gmail account via OAuth2 flow
pub async fn add_account(_authenticator: &GoogleAuthenticator, _store: &AccountStore) -> Result<String> {
    todo!("Implement OAuth2 flow and save account")
}

/// List all configured accounts
pub async fn list_accounts(_store: &AccountStore) -> Result<Vec<String>> {
    todo!("List all account emails from store")
}

/// Remove an account
pub async fn remove_account(_email: &str, _store: &AccountStore) -> Result<()> {
    todo!("Remove account from store and keyring")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_operations() {
        // TODO: Add unit tests
    }
}
