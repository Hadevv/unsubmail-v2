//! Batch cleanup example (non-interactive)
//!
//! This example demonstrates how to:
//! 1. Scan for newsletters with high confidence scores
//! 2. Automatically clean them without user interaction
//! 3. Report results
//!
//! **WARNING**: This example will actually delete emails!
//! Only use with a test Gmail account, not your primary account.
//!
//! Usage:
//!   cargo run --example batch_cleanup <email@gmail.com> [--dry-run]
//!
//! Prerequisites:
//!   - Set GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET environment variables
//!   - Use a TEST Gmail account only!

use anyhow::Result;
use unsubmail::application::workflow;
use unsubmail::domain::{analysis, models::ActionType, planner};
use unsubmail::infrastructure::{imap, network, storage};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::from_filename(".env.local")
        .or_else(|_| dotenvy::dotenv())
        .ok();

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <email@gmail.com> [--dry-run]", args[0]);
        eprintln!("\nWARNING: This will DELETE emails! Use a test account only.");
        std::process::exit(1);
    }

    let email = &args[1];
    let dry_run = args.get(2).map(|s| s == "--dry-run").unwrap_or(false);

    if dry_run {
        println!("DRY RUN MODE - No actual changes will be made\n");
    } else {
        println!("WARNING: Running in REAL mode - emails will be deleted!");
        println!("Press Ctrl+C to cancel or Enter to continue...");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
    }

    println!("UnsubMail - Batch Cleanup Example");
    println!("==================================\n");
    println!("Email: {}\n", email);

    // Get or create token
    let access_token = match storage::keyring::get_token(email)? {
        Some(token) if !token.is_expired() => {
            println!("Using existing OAuth2 token");
            token.access_token
        }
        _ => {
            println!("Authenticating with Google...");
            let account = workflow::add_account_for_email(email).await?;
            let token = storage::keyring::get_token(&account.email)?
                .ok_or_else(|| anyhow::anyhow!("Token not found after auth"))?;
            token.access_token
        }
    };

    // Connect and scan
    println!("\nConnecting to Gmail IMAP...");
    let mut session = imap::connection::connect_and_auth(email, &access_token).await?;

    println!("Fetching message headers (max 200)...");
    let headers = imap::fetch::fetch_all_headers(&mut session, 200).await?;
    println!("Fetched {} messages", headers.len());

    // Group and analyze
    let grouped = imap::fetch::group_by_sender(headers);
    let senders: Vec<_> = grouped
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

    // Filter: Only high-confidence newsletters (score > 1.0 OR one-click available)
    let candidates: Vec<_> = senders
        .into_iter()
        .filter(|s| s.heuristic_score > 1.0 || s.unsubscribe_method.is_one_click())
        .collect();

    println!(
        "\nFound {} high-confidence newsletter senders",
        candidates.len()
    );

    if candidates.is_empty() {
        println!("No newsletters to clean!");
        session.logout().await?;
        return Ok(());
    }

    // Plan actions
    let actions = planner::plan_actions(candidates);

    println!("\nPlanned Actions:");
    for action in &actions {
        let name = action
            .sender
            .display_name
            .as_ref()
            .unwrap_or(&action.sender.email);
        let action_str = match action.action_type {
            ActionType::UnsubscribeAndDelete => "Unsubscribe + Delete",
            ActionType::SpamAndDelete => "Spam + Delete",
            ActionType::DeleteOnly => "Delete Only",
        };
        println!(
            "  - {} ({} msgs): {}",
            name, action.sender.message_count, action_str
        );
    }

    if dry_run {
        println!("\nDRY RUN - Skipping actual execution");
        session.logout().await?;
        return Ok(());
    }

    println!("\nExecuting cleanup...\n");

    let mut total_deleted = 0;
    let mut total_unsubscribed = 0;

    for action in actions {
        let name = action
            .sender
            .display_name
            .as_ref()
            .unwrap_or(&action.sender.email);
        println!("Processing: {}", name);

        // Try to unsubscribe if one-click available
        if action.sender.unsubscribe_method.is_one_click() {
            if let unsubmail::domain::models::UnsubscribeMethod::OneClick { url } =
                &action.sender.unsubscribe_method
            {
                match network::http_client::unsubscribe_one_click(url).await {
                    Ok(true) => {
                        println!("  ✓ Unsubscribed");
                        total_unsubscribed += 1;
                    }
                    Ok(false) => println!("  ✗ Unsubscribe failed"),
                    Err(e) => println!("  ✗ Unsubscribe error: {}", e),
                }
            }
        }

        // Delete messages
        match imap::actions::delete_messages(&mut session, &action.sender.message_uids).await {
            Ok(count) => {
                println!("  ✓ Deleted {} messages", count);
                total_deleted += count;
            }
            Err(e) => {
                println!("  ✗ Delete error: {}", e);
            }
        }
    }

    println!("\n{}", "=".repeat(80));
    println!("Cleanup Complete!");
    println!("  Total unsubscribed: {}", total_unsubscribed);
    println!("  Total messages deleted: {}", total_deleted);

    session.logout().await?;

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
