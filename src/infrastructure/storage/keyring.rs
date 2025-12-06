//! Secure token storage using confy

use crate::domain::models::OAuth2Token;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const APP_NAME: &str = "unsubmail";
const CONFIG_NAME: &str = "tokens";

/// Token storage configuration
#[derive(Debug, Default, Serialize, Deserialize)]
struct TokenStore {
    tokens: HashMap<String, OAuth2Token>,
}

/// Store OAuth2 token for an email
pub fn store_token(email: &str, token: OAuth2Token) -> Result<()> {
    let mut store: TokenStore =
        confy::load(APP_NAME, CONFIG_NAME).context("Failed to load token store")?;

    store.tokens.insert(email.to_string(), token);

    confy::store(APP_NAME, CONFIG_NAME, store).context("Failed to save token store")?;

    Ok(())
}

/// Get OAuth2 token for an email
pub fn get_token(email: &str) -> Result<Option<OAuth2Token>> {
    let store: TokenStore =
        confy::load(APP_NAME, CONFIG_NAME).context("Failed to load token store")?;

    Ok(store.tokens.get(email).cloned())
}

/// Delete token for an email
pub fn delete_token(email: &str) -> Result<()> {
    let mut store: TokenStore =
        confy::load(APP_NAME, CONFIG_NAME).context("Failed to load token store")?;

    store.tokens.remove(email);

    confy::store(APP_NAME, CONFIG_NAME, store).context("Failed to save token store")?;

    Ok(())
}

/// List all emails with stored tokens
pub fn list_token_emails() -> Result<Vec<String>> {
    let store: TokenStore =
        confy::load(APP_NAME, CONFIG_NAME).context("Failed to load token store")?;

    Ok(store.tokens.keys().cloned().collect())
}
