//! Google OAuth2 authentication
//!
//! Handles OAuth2 flow and token management.

use anyhow::{Context, Result};
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use yup_oauth2::authenticator::Authenticator;
use std::path::PathBuf;

pub type Auth = Authenticator<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>;

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
        use std::fs;

        // Ensure token directory exists
        fs::create_dir_all(&self.token_cache_dir)
            .context("Failed to create token cache directory")?;

        let secret = yup_oauth2::ApplicationSecret {
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
            token_uri: "https://oauth2.googleapis.com/token".to_string(),
            redirect_uris: vec!["http://localhost:9090".to_string()],
            ..Default::default()
        };

        let token_file = self.token_cache_dir.join(format!("{}.json", email.replace(['@', '.'], "_")));

        println!("🔐 Starting OAuth2 flow...");
        println!("📝 Your browser will open automatically for authentication");
        println!("⏳ Please complete the authorization in your browser...");
        println!();

        let auth = InstalledFlowAuthenticator::builder(
            secret,
            InstalledFlowReturnMethod::Interactive,
        )
        .persist_tokens_to_disk(token_file)
        .build()
        .await
        .context("Failed to create authenticator")?;

        // Force token acquisition with required scopes
        let scopes = &[
            "https://www.googleapis.com/auth/gmail.readonly",
            "https://www.googleapis.com/auth/gmail.modify",
            "https://www.googleapis.com/auth/gmail.settings.basic",
        ];

        auth.token(scopes)
            .await
            .context("Failed to obtain OAuth2 token")?;

        println!("✓ Authentication successful!");

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
