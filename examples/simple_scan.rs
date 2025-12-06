//! Simple inbox scan example
//!
//! This example demonstrates how to:
//! 1. Authenticate with Gmail via OAuth2
//! 2. Scan an inbox for newsletters
//! 3. Display results without taking any action
//!
//! Usage:
//!   cargo run --example simple_scan
//!
//! Prerequisites:
//!   - Set GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET environment variables
//!   - Have a Gmail account to test with (use a test account!)

use anyhow::Result;
use unsubmail::application::workflow;
use unsubmail::domain::analysis;
use unsubmail::infrastructure::{imap, storage};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::from_filename(".env.local")
        .or_else(|_| dotenvy::dotenv())
        .ok();

    println!("UnsubMail - Simple Scan Example");
    println!("================================\n");

    // Get email from command line or prompt
    let email = std::env::args().nth(1).unwrap_or_else(|| {
        println!("Usage: cargo run --example simple_scan <email@gmail.com>");
        std::process::exit(1);
    });

    println!("Email: {}\n", email);

    // Check if token exists
    let access_token = match storage::keyring::get_token(&email)? {
        Some(token) if !token.is_expired() => {
            println!("Using existing OAuth2 token");
            token.access_token
        }
        _ => {
            println!("Authenticating with Google...");
            let account = workflow::add_account_for_email(&email).await?;
            let token = storage::keyring::get_token(&account.email)?
                .ok_or_else(|| anyhow::anyhow!("Token not found after auth"))?;
            token.access_token
        }
    };

    // Connect to IMAP
    println!("\nConnecting to Gmail IMAP...");
    let mut session = imap::connection::connect_and_auth(&email, &access_token).await?;

    // Fetch message headers (limit to 100 for this example)
    println!("Fetching message headers (max 100)...");
    let headers = imap::fetch::fetch_all_headers(&mut session, 100).await?;
    println!("Fetched {} messages\n", headers.len());

    // Group by sender
    let grouped = imap::fetch::group_by_sender(headers);
    println!("Found {} unique senders\n", grouped.len());

    // Analyze each sender
    let mut senders: Vec<_> = grouped
        .into_iter()
        .map(|(email, messages)| {
            let message_count = messages.len();
            let message_uids: Vec<u32> = messages.iter().map(|m| m.uid).collect();
            let first = &messages[0];
            let display_name = extract_display_name(&first.from);
            let sample_subjects: Vec<String> =
                messages.iter().take(3).map(|m| m.subject.clone()).collect();

            analysis::analyze_sender(
                email,
                display_name,
                message_count,
                message_uids,
                first.list_unsubscribe.clone(),
                first.list_unsubscribe_post.clone(),
                sample_subjects,
            )
        })
        .collect();

    // Sort by heuristic score (highest first)
    senders.sort_by(|a, b| {
        b.heuristic_score
            .partial_cmp(&a.heuristic_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Display top newsletter candidates
    println!("Top Newsletter Candidates:");
    println!("{}", "=".repeat(80));

    for (i, sender) in senders.iter().take(20).enumerate() {
        let name = sender.display_name.as_ref().unwrap_or(&sender.email);
        let method_str = match &sender.unsubscribe_method {
            unsubmail::domain::models::UnsubscribeMethod::OneClick { .. } => "✓ One-Click",
            unsubmail::domain::models::UnsubscribeMethod::HttpLink { .. } => "⚠ HTTP Link",
            unsubmail::domain::models::UnsubscribeMethod::Mailto { .. } => "✉ Mailto",
            unsubmail::domain::models::UnsubscribeMethod::None => "✗ None",
        };

        println!(
            "{}. {} ({} msgs) - Score: {:.2} - Unsubscribe: {}",
            i + 1,
            name,
            sender.message_count,
            sender.heuristic_score,
            method_str
        );

        if !sender.sample_subjects.is_empty() {
            println!("   Sample: {}", sender.sample_subjects[0]);
        }
    }

    // Disconnect
    session.logout().await?;

    println!("\n{}", "=".repeat(80));
    println!("Scan complete!");
    println!("\nNote: This example only scans and displays results.");
    println!("Use the interactive mode to actually clean your inbox:");
    println!("  cargo run");

    Ok(())
}

fn extract_display_name(from: &str) -> Option<String> {
    if let Some(pos) = from.find('<') {
        let name = from[..pos].trim().trim_matches('"');
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}
