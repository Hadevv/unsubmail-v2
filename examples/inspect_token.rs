//! Inspect stored OAuth2 token and check if it's valid

use chrono::Utc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::from_filename(".env.local")
        .or_else(|_| dotenvy::dotenv())
        .ok();

    println!("=== Token Inspector ===\n");

    // Get email from user
    println!("Enter the Gmail address to check: ");
    let mut email = String::new();
    std::io::stdin().read_line(&mut email)?;
    let email = email.trim();

    println!("\nChecking token for: {}\n", email);

    // Try to get token from keyring
    match unsubmail::infrastructure::storage::keyring::get_token(email)? {
        Some(token) => {
            println!("✓ Token found in keyring!");
            println!("\n=== Token Details ===");

            // Show partial access token (first/last chars only for security)
            let access_token = &token.access_token;
            if access_token.len() > 20 {
                println!(
                    "Access Token: {}...{}",
                    &access_token[..10],
                    &access_token[access_token.len() - 10..]
                );
            } else {
                println!("Access Token: [too short - invalid?]");
            }

            // Show partial refresh token
            let refresh_token = &token.refresh_token;
            if refresh_token.len() > 20 {
                println!(
                    "Refresh Token: {}...{}",
                    &refresh_token[..10],
                    &refresh_token[refresh_token.len() - 10..]
                );
            } else {
                println!("Refresh Token: [too short - invalid?]");
            }

            // Check expiration
            println!("\nExpires at: {}", token.expires_at);
            println!("Current time: {}", Utc::now());

            if token.is_expired() {
                println!("\n❌ TOKEN IS EXPIRED!");
                println!(
                    "   Time since expiration: {:?}",
                    Utc::now() - token.expires_at
                );
                println!("\n🔧 Action Required:");
                println!("   1. Delete this token from Windows Credential Manager");
                println!("   2. Run the main program to get a new token");
            } else {
                let time_left = token.expires_at - Utc::now();
                println!("\n✓ Token is still valid");
                println!("   Time remaining: {} minutes", time_left.num_minutes());

                if time_left.num_minutes() < 5 {
                    println!("\n⚠️  WARNING: Token expires soon!");
                    println!("   Consider refreshing now to avoid issues");
                }
            }

            // Check token format
            println!("\n=== Token Format Check ===");
            if access_token.starts_with("ya29.") {
                println!("✓ Access token format looks correct (starts with 'ya29.')");
            } else {
                println!("❌ Access token format looks WRONG (should start with 'ya29.')");
                println!("   This token may be invalid!");
            }

            if access_token.len() > 100 {
                println!("✓ Access token length OK ({} chars)", access_token.len());
            } else {
                println!(
                    "⚠️  Access token seems too short ({} chars)",
                    access_token.len()
                );
            }
        }
        None => {
            println!("❌ No token found for this email!");
            println!("\n🔧 Action Required:");
            println!("   Run the main program to authenticate:");
            println!("   cargo run");
        }
    }

    println!("\n=== How to Delete Token ===");
    println!("1. Press Windows + R");
    println!("2. Type: control /name Microsoft.CredentialManager");
    println!("3. Click 'Windows Credentials'");
    println!("4. Look for entries with 'unsubmail' or '{}'", email);
    println!("5. Click the arrow, then 'Remove'");

    Ok(())
}
