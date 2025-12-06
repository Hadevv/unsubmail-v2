//! IMAP connection management

use super::auth::build_xoauth2_string;
use anyhow::{Context, Result};
use async_imap::Session;
use async_native_tls::{TlsConnector, TlsStream};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncReadCompatExt;

const GMAIL_IMAP_HOST: &str = "imap.gmail.com";
const GMAIL_IMAP_PORT: u16 = 993;

/// IMAP session type
pub type ImapSession = Session<TlsStream<tokio_util::compat::Compat<TcpStream>>>;

/// XOAUTH2 Authenticator
struct XOAuth2 {
    auth_str: String,
}

impl async_imap::Authenticator for XOAuth2 {
    type Response = String;

    fn process(&mut self, _challenge: &[u8]) -> Self::Response {
        self.auth_str.clone()
    }
}

/// Connect to Gmail IMAP server with TLS
pub async fn connect(
) -> Result<async_imap::Client<TlsStream<tokio_util::compat::Compat<TcpStream>>>> {
    tracing::info!("Connecting to {}:{}", GMAIL_IMAP_HOST, GMAIL_IMAP_PORT);

    let tcp_stream = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        TcpStream::connect((GMAIL_IMAP_HOST, GMAIL_IMAP_PORT)),
    )
    .await
    .context("Timeout while connecting to Gmail IMAP - Check your network connection")?
    .context("Failed to connect to Gmail IMAP - Verify port 993 is not blocked by firewall")?;

    tracing::info!("✓ TCP connection established, starting TLS handshake");

    // Convert tokio stream to futures-compatible stream
    let compat_stream = tcp_stream.compat();

    let tls = TlsConnector::new();
    let tls_stream = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        tls.connect(GMAIL_IMAP_HOST, compat_stream),
    )
    .await
    .context("Timeout during TLS handshake")?
    .context("Failed to establish TLS connection")?;

    tracing::info!("✓ TLS handshake complete, creating IMAP client");

    let client = async_imap::Client::new(tls_stream);

    tracing::info!("✓ IMAP client created successfully");

    Ok(client)
}

/// Authenticate using XOAUTH2
pub async fn authenticate(
    mut client: async_imap::Client<TlsStream<tokio_util::compat::Compat<TcpStream>>>,
    email: &str,
    access_token: &str,
) -> Result<ImapSession> {
    tracing::info!("Starting XOAUTH2 authentication for {}", email);

    // WORKAROUND for async-imap issue #84:
    // Gmail sends a greeting that must be consumed before authentication
    // See: https://github.com/async-email/async-imap/issues/84
    tracing::info!("Reading server greeting...");
    let greeting = tokio::time::timeout(std::time::Duration::from_secs(10), client.read_response())
        .await
        .context("Timeout while reading server greeting")?
        .context("Failed to read server greeting")?;

    tracing::info!("Server greeting received: {:?}", greeting);

    let auth_str = build_xoauth2_string(email, access_token);
    let authenticator = XOAuth2 { auth_str };

    tracing::info!("Sending AUTHENTICATE XOAUTH2 command...");

    let session = tokio::time::timeout(
        std::time::Duration::from_secs(15),
        client.authenticate("XOAUTH2", authenticator),
    )
    .await
    .context(
        "Timeout during XOAUTH2 authentication - This usually means:\n\
             1. OAuth2 token is invalid or expired\n\
             2. IMAP access is disabled in Gmail settings\n\
             3. Gmail API is not enabled in Google Cloud Console\n\
             4. OAuth2 scope 'https://mail.google.com/' is missing\n\n\
             Please check: https://mail.google.com/mail/u/0/#settings/fwdandpop",
    )?
    .map_err(|(err, _client)| {
        tracing::error!("XOAUTH2 authentication failed: {:?}", err);
        anyhow::anyhow!(
            "XOAUTH2 authentication failed: {:?}\n\n\
             Common causes:\n\
             1. OAuth2 token is invalid or expired (try re-authenticating)\n\
             2. IMAP is not enabled in Gmail settings\n\
             3. OAuth2 client doesn't have correct scopes\n\
             4. Gmail security settings block IMAP access\n\n\
             Enable IMAP: https://mail.google.com/mail/u/0/#settings/fwdandpop\n\
             Check 'IMAP Access' section and enable it",
            err
        )
    })?;

    tracing::info!("✓ XOAUTH2 authentication successful");

    Ok(session)
}

/// Connect and authenticate in one step
pub async fn connect_and_auth(email: &str, access_token: &str) -> Result<ImapSession> {
    let client = connect().await?;
    authenticate(client, email, access_token).await
}
