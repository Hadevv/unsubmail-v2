//! Account management CLI commands
//!
//! Handles adding, listing, and removing Gmail accounts via OAuth2.

use anyhow::Result;
use crate::infrastructure::google::auth::GoogleAuthenticator;
use crate::infrastructure::storage::json_store::AccountStore;

/// Add a new Gmail account via OAuth2 flow
#[allow(dead_code)]
pub async fn add_account(authenticator: &GoogleAuthenticator, store: &AccountStore) -> Result<String> {
    // TODO: Implement OAuth2 flow
    let _ = (authenticator, store);
    todo!("Implement OAuth2 flow and save account")
}

/// List all configured accounts
#[allow(dead_code)]
pub async fn list_accounts(store: &AccountStore) -> Result<Vec<String>> {
    // TODO: List accounts from store
    let _ = store;
    todo!("List all account emails from store")
}

/// Remove an account
#[allow(dead_code)]
pub async fn remove_account(email: &str, store: &AccountStore) -> Result<()> {
    // TODO: Remove from store and keyring
    let _ = (email, store);
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
