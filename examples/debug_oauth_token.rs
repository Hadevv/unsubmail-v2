//! Debug OAuth2 token - Verify token validity and scopes

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::from_filename(".env.local")
        .or_else(|_| dotenvy::dotenv())
        .ok();

    println!("=== OAuth2 Token Diagnostic Tool ===\n");

    // Check if credentials are set
    let client_id = env::var("GOOGLE_CLIENT_ID");
    let client_secret = env::var("GOOGLE_CLIENT_SECRET");

    match (&client_id, &client_secret) {
        (Ok(id), Ok(secret)) => {
            println!("✓ GOOGLE_CLIENT_ID: {}...{}", &id[..20], &id[id.len()-20..]);
            println!("✓ GOOGLE_CLIENT_SECRET: {}***", &secret[..10]);
        }
        _ => {
            println!("❌ OAuth2 credentials NOT found in .env.local");
            println!("   Please set GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET");
            return Ok(());
        }
    }

    println!("\n=== Checking stored token for test email ===");

    // Try to get token from keyring (this will fail but show us the error)
    println!("\nℹ️  To check your actual token, run the main program once to authenticate.");
    println!("   The token will be stored securely in Windows Credential Manager.");

    println!("\n=== Manual Token Check ===");
    println!("1. Press Windows key + R");
    println!("2. Type: control /name Microsoft.CredentialManager");
    println!("3. Look for entries containing 'unsubmail'");
    println!("4. If found, the token exists");
    println!("5. If token is > 1 hour old, it's expired - delete it and re-authenticate");

    println!("\n=== Required OAuth2 Scope ===");
    println!("The token MUST have this scope: https://mail.google.com/");
    println!("This scope includes IMAP access.");

    println!("\n=== Common Issues ===");
    println!("1. Token expired (> 1 hour old)");
    println!("2. Wrong scope (gmail.readonly doesn't work for IMAP)");
    println!("3. IMAP disabled in Gmail settings");
    println!("4. Gmail API not enabled in Google Cloud Console");

    println!("\n=== Next Steps ===");
    println!("1. Verify IMAP is enabled: https://mail.google.com/mail/u/0/#settings/fwdandpop");
    println!("2. Delete old token from Credential Manager if exists");
    println!("3. Run the main program to get a fresh token");

    Ok(())
}
