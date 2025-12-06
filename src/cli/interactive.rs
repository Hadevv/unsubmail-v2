//! Interactive CLI - Simplified linear workflow

use crate::application::workflow;
use crate::domain::models::{SenderInfo, UnsubscribeMethod};
use crate::infrastructure::{imap, network, storage};
use anyhow::Result;
use console::{style, Term};
use inquire::{Text, Confirm, MultiSelect};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Main interactive workflow
pub async fn run_interactive() -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    print_header();
    
    // Step 1: Ask for email
    let email = Text::new("Gmail address:")
        .with_help_message("Enter your Gmail email address")
        .prompt()?;
    
    println!();
    
    // Step 2: Get or create OAuth2 token
    let access_token = get_or_create_token(&email).await?;
    
    // Step 3: Scan inbox
    println!();
    println!("{}", style("Scanning inbox...").bold());
    println!();
    
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(
        indicatif::ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    
    let senders = scan_inbox(&email, &access_token, pb).await?;
    
    if senders.is_empty() {
        println!("{}", style("No senders found").yellow());
        press_enter();
        return Ok(());
    }
    
    display_results(&senders);
    
    // Step 4: Select senders
    println!();
    let selected = select_senders(&senders)?;
    
    if selected.is_empty() {
        println!("{}", style("No senders selected").yellow());
        press_enter();
        return Ok(());
    }
    
    // Step 5: Clean
    println!();
    println!("{}", style("Cleaning...").bold());
    println!();
    
    execute_cleanup(&email, &access_token, &selected).await?;
    
    println!();
    println!("{}", style("Done!").green().bold());
    press_enter();
    
    Ok(())
}

fn print_header() {
    println!();
    println!("{}", style("═".repeat(60)).cyan());
    println!(
        "{}{}{}",
        style("  UnsubMail").cyan().bold(),
        style(" v").dim(),
        style(VERSION).dim()
    );
    println!("{}", style("  Clean your Gmail inbox from newsletters and spam").dim());
    println!("{}", style("═".repeat(60)).cyan());
    println!();
}

/// Get existing token or create new one via OAuth2
async fn get_or_create_token(email: &str) -> Result<String> {
    // Check if token exists
    if let Some(token) = storage::keyring::get_token(email)? {
        if !token.is_expired() {
            println!("{}", style("✓ Using existing authentication").dim());
            return Ok(token.access_token);
        } else {
            // Token expired, try to refresh it
            println!("{}", style("Refreshing expired token...").dim());
            match workflow::refresh_token_for_email(email).await {
                Ok(new_token) => {
                    println!("{}", style("✓ Token refreshed successfully").dim());
                    return Ok(new_token.access_token);
                }
                Err(e) => {
                    println!("{}", style(format!("Failed to refresh token: {}", e)).yellow());
                    println!("{}", style("Re-authenticating...").dim());
                }
            }
        }
    }

    // Need to authenticate (first time or refresh failed)
    println!("{}", style("Authenticating with Google...").bold());
    println!();

    let account = workflow::add_account_for_email(email).await?;

    let token = storage::keyring::get_token(&account.email)?
        .ok_or_else(|| anyhow::anyhow!("Token not found after authentication"))?;

    Ok(token.access_token)
}

/// Scan inbox
async fn scan_inbox(
    email: &str,
    access_token: &str,
    pb: indicatif::ProgressBar,
) -> Result<Vec<SenderInfo>> {
    pb.set_message("Connecting to IMAP...");

    let mut session = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        imap::connection::connect_and_auth(email, access_token)
    )
    .await
    .map_err(|_| {
        anyhow::anyhow!(
            "Connection timed out after 30 seconds.\n\
            This usually means the OAuth2 token is invalid or network issues.\n\
            Try re-running the program to refresh your authentication."
        )
    })??;

    pb.set_message("Fetching messages...");
    let headers = imap::fetch::fetch_all_headers(&mut session, 200).await?;
    
    pb.set_message("Analyzing senders...");
    let grouped = imap::fetch::group_by_sender(headers);
    
    let senders: Vec<SenderInfo> = grouped
        .into_iter()
        .map(|(email, messages)| {
            let message_count = messages.len();
            let message_uids: Vec<u32> = messages.iter().map(|m| m.uid).collect();
            let first = &messages[0];
            let display_name = extract_display_name(&first.from);
            let sample_subjects: Vec<String> = messages
                .iter()
                .take(3)
                .map(|m| m.subject.clone())
                .collect();
            
            crate::domain::analysis::analyze_sender(
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
    
    session.logout().await?;
    pb.finish_and_clear();
    
    Ok(senders)
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

fn display_results(senders: &[SenderInfo]) {
    println!();
    println!("{}", style("Scan Results").bold().underlined());
    println!();
    println!("  {} unique senders found", senders.len());
    
    let with_unsub = senders.iter().filter(|s| s.unsubscribe_method.is_available()).count();
    let with_one_click = senders.iter().filter(|s| s.unsubscribe_method.is_one_click()).count();
    
    println!("  {} with unsubscribe option", with_unsub);
    println!("  {} with one-click unsubscribe", with_one_click);
    println!();
}

fn select_senders(senders: &[SenderInfo]) -> Result<Vec<SenderInfo>> {
    let mut sorted = senders.to_vec();
    sorted.sort_by(|a, b| {
        b.heuristic_score
            .partial_cmp(&a.heuristic_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    
    let options: Vec<String> = sorted
        .iter()
        .map(|s| {
            let name = s.display_name.as_ref().unwrap_or(&s.email);
            let method = if s.unsubscribe_method.is_one_click() {
                "✓ One-Click"
            } else if s.unsubscribe_method.is_available() {
                "⚠ Manual"
            } else {
                "✗ No unsub"
            };
            format!("{} ({} msgs) {}", name, s.message_count, method)
        })
        .collect();
    
    let selected_strs = MultiSelect::new("Select senders to clean:", options)
        .with_help_message("Use Space to select, Enter to confirm")
        .prompt()?;
    
    let selected: Vec<SenderInfo> = selected_strs
        .iter()
        .filter_map(|s| {
            sorted.iter().find(|sender| {
                let name = sender.display_name.as_ref().unwrap_or(&sender.email);
                s.starts_with(name)
            }).cloned()
        })
        .collect();
    
    Ok(selected)
}

async fn execute_cleanup(
    email: &str,
    access_token: &str,
    senders: &[SenderInfo],
) -> Result<()> {
    let mut session = imap::connection::connect_and_auth(email, access_token).await?;
    
    for (idx, sender) in senders.iter().enumerate() {
        println!();
        println!(
            "{} {} ({} messages)",
            style(format!("[{}/{}]", idx + 1, senders.len())).dim(),
            style(&sender.email).cyan().bold(),
            sender.message_count
        );
        
        let has_one_click = sender.unsubscribe_method.is_one_click();
        
        if has_one_click {
            println!("  {} One-click unsubscribe available", style("✓").green());
            
            let unsub = Confirm::new("Unsubscribe from this sender?")
                .with_default(true)
                .prompt()?;
            
            if unsub {
                if let UnsubscribeMethod::OneClick { url } = &sender.unsubscribe_method {
                    match network::http_client::unsubscribe_one_click(url).await {
                        Ok(true) => println!("  {} Unsubscribed successfully", style("✓").green()),
                        Ok(false) => println!("  {} Unsubscribe failed", style("✗").red()),
                        Err(e) => println!("  {} Error: {}", style("✗").red(), e),
                    }
                }
            }
        } else {
            println!("  {} No one-click unsubscribe", style("!").yellow());
            
            let block = Confirm::new("Block this sender (move to spam)?")
                .with_default(true)
                .prompt()?;
            
            if block {
                match imap::actions::move_to_spam(&mut session, &sender.message_uids).await {
                    Ok(count) => {
                        println!("  {} Moved {} messages to spam", style("✓").green(), count);
                        continue;
                    }
                    Err(e) => println!("  {} Error: {}", style("✗").red(), e),
                }
            }
        }
        
        let delete = Confirm::new(&format!(
            "Delete all {} messages from this sender?",
            sender.message_count
        ))
        .with_default(false)
        .prompt()?;
        
        if delete {
            match imap::actions::delete_messages(&mut session, &sender.message_uids).await {
                Ok(count) => println!("  {} Deleted {} messages", style("✓").green(), count),
                Err(e) => println!("  {} Error: {}", style("✗").red(), e),
            }
        }
    }
    
    session.logout().await?;
    
    Ok(())
}

fn press_enter() {
    println!();
    println!("{}", style("Press Enter to exit...").dim());
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
}
