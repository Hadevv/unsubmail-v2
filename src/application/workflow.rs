//! Main workflow orchestration

use crate::domain::models::*;
use crate::infrastructure::{imap, network, storage};
use anyhow::{Context, Result};
use chrono::Utc;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use url::Url;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GMAIL_SCOPE: &str = "https://mail.google.com/";

/// Add account for specific email (OAuth2 flow with browser)
pub async fn add_account_for_email(email: &str) -> Result<EmailAccount> {
    // Get OAuth2 credentials from environment
    let client_id = env::var("GOOGLE_CLIENT_ID")
        .context("GOOGLE_CLIENT_ID not set")?;
    let client_secret = env::var("GOOGLE_CLIENT_SECRET")
        .context("GOOGLE_CLIENT_SECRET not set")?;
    let redirect_uri = env::var("GOOGLE_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:9090/callback".to_string());

    // Create OAuth2 client
    let client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(GOOGLE_AUTH_URL.to_string())?,
        Some(TokenUrl::new(GOOGLE_TOKEN_URL.to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri.clone())?);

    // Generate PKCE challenge
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate authorization URL
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(GMAIL_SCOPE.to_string()))
        .add_extra_param("access_type", "offline")
        .add_extra_param("prompt", "consent")
        .add_extra_param("login_hint", email)
        .set_pkce_challenge(pkce_challenge)
        .url();

    // Open browser
    println!("\nOpening browser for authentication...");
    println!("If browser doesn't open, visit: {}\n", auth_url);

    if let Err(e) = open::that(auth_url.as_str()) {
        eprintln!("Failed to open browser: {}", e);
    }

    // Start local server to receive callback
    let listener = TcpListener::bind("127.0.0.1:9090")
        .context("Failed to bind to localhost:9090")?;

    println!("Waiting for authorization...\n");

    // Wait for callback
    let (mut stream, _) = listener.accept()
        .context("Failed to accept connection")?;

    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)
        .context("Failed to read request")?;

    // Parse callback URL
    let redirect_url = request_line.split_whitespace().nth(1)
        .context("Invalid request line")?;
    let url = Url::parse(&format!("http://localhost:9090{}", redirect_url))
        .context("Failed to parse callback URL")?;

    // Send success response to browser
    let response = "HTTP/1.1 200 OK\r\n\r\n<html><body><h1>Authentication successful!</h1><p>You can close this window.</p></body></html>";
    stream.write_all(response.as_bytes()).ok();

    // Extract code and state
    let code = url
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| AuthorizationCode::new(value.to_string()))
        .context("Authorization code not found")?;

    let state = url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| CsrfToken::new(value.to_string()))
        .context("State not found")?;

    // Verify CSRF token
    if state.secret() != csrf_token.secret() {
        anyhow::bail!("CSRF token mismatch");
    }

    // Exchange code for token
    let token = client
        .exchange_code(code)
        .set_pkce_verifier(pkce_verifier)
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .context("Failed to exchange authorization code for token")?;

    // Store token for provided email
    let access_token = token.access_token().secret();
    let oauth_token = OAuth2Token {
        access_token: access_token.clone(),
        refresh_token: token
            .refresh_token()
            .context("No refresh token received")?
            .secret()
            .clone(),
        expires_at: Utc::now() + chrono::Duration::seconds(3600),
    };

    storage::keyring::store_token(email, oauth_token)?;

    // Create and save account
    let account = EmailAccount {
        email: email.to_string(),
        added_at: Utc::now(),
    };

    storage::json_store::save_account(&account)?;

    Ok(account)
}

/// Refresh an expired OAuth2 token
pub async fn refresh_token_for_email(email: &str) -> Result<OAuth2Token> {
    tracing::debug!("Refreshing token for {}", email);

    // Get existing token (which should have refresh_token)
    let old_token = storage::keyring::get_token(email)?
        .context("No existing token found for this email")?;

    // Get OAuth2 credentials from environment
    let client_id = env::var("GOOGLE_CLIENT_ID")
        .context("GOOGLE_CLIENT_ID not set")?;
    let client_secret = env::var("GOOGLE_CLIENT_SECRET")
        .context("GOOGLE_CLIENT_SECRET not set")?;

    // Create OAuth2 client
    let client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(GOOGLE_AUTH_URL.to_string())?,
        Some(TokenUrl::new(GOOGLE_TOKEN_URL.to_string())?),
    );

    // Exchange refresh token for new access token
    let refresh_token = oauth2::RefreshToken::new(old_token.refresh_token.clone());

    let token_response = client
        .exchange_refresh_token(&refresh_token)
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .context("Failed to refresh token")?;

    // Store new token
    let new_token = OAuth2Token {
        access_token: token_response.access_token().secret().clone(),
        refresh_token: old_token.refresh_token, // Keep the same refresh token
        expires_at: Utc::now()
            + chrono::Duration::seconds(
                token_response.expires_in().map(|d| d.as_secs() as i64).unwrap_or(3600)
            ),
    };

    storage::keyring::store_token(email, new_token.clone())?;

    tracing::debug!("Token refreshed successfully for {}", email);

    Ok(new_token)
}
