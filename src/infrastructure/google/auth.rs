//! Google OAuth2 authentication
//!
//! Handles OAuth2 flow and token management.

use anyhow::{Context, Result};
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use yup_oauth2::authenticator::Authenticator;
use yup_oauth2::hyper::client::HttpConnector;
use yup_oauth2::hyper_rustls::HttpsConnector;
use std::path::PathBuf;

pub type Auth = Authenticator<HttpsConnector<HttpConnector>>;

/// Google OAuth2 authenticator
pub struct GoogleAuthenticator {
    client_id: String,
    client_secret: String,
    token_cache_dir: PathBuf,
}

impl GoogleAuthenticator {
    /// Create new authenticator
    pub fn new(client_id: String, client_secret: String, token_cache_dir: PathBuf) -> Self {
        Self {
            client_id,
            client_secret,
            token_cache_dir,
        }
    }

    /// Run OAuth2 flow and return authenticator
    pub async fn authenticate(&self, email: &str) -> Result<Auth> {
        let secret = yup_oauth2::ApplicationSecret {
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
            token_uri: "https://oauth2.googleapis.com/token".to_string(),
            ..Default::default()
        };

        let token_file = self.token_cache_dir.join(format!("{}.json", email));

        let auth = InstalledFlowAuthenticator::builder(
            secret,
            InstalledFlowReturnMethod::HTTPRedirect,
        )
        .persist_tokens_to_disk(token_file)
        .build()
        .await
        .context("Failed to create authenticator")?;

        Ok(auth)
    }

    /// Get existing authenticator from cached tokens
    pub async fn get_authenticator(&self, email: &str) -> Result<Auth> {
        self.authenticate(email).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authenticator_creation() {
        let auth = GoogleAuthenticator::new(
            "client_id".to_string(),
            "client_secret".to_string(),
            PathBuf::from("/tmp"),
        );
        assert_eq!(auth.client_id, "client_id");
    }
}
