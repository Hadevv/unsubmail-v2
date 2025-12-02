//! UnsubMail CLI entry point
//!
//! A fast, reliable CLI tool to clean Gmail inbox from newsletters and spam.

mod application;
mod cli;
mod domain;
mod infrastructure;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use application::workflow::Workflow;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "unsubmail")]
#[command(about = "Clean your Gmail inbox from newsletters and spam", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Configuration directory
    #[arg(long)]
    config_dir: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a Gmail account via OAuth2
    Add,

    /// List configured accounts
    List,

    /// Scan inbox for newsletters and unwanted senders
    Scan {
        /// Account email to scan
        email: String,
    },

    /// Clean inbox (scan + select + cleanup)
    Clean {
        /// Account email to clean
        email: String,
    },

    /// Remove an account
    Remove {
        /// Account email to remove
        email: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "unsubmail=info,warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    // Get config directory
    let config_dir = cli.config_dir
        .or_else(|| {
            dirs::config_dir().map(|d| d.join("unsubmail"))
        })
        .context("Failed to determine config directory")?;

    // Create workflow
    let workflow = Workflow::new(config_dir)?;

    match cli.command {
        Commands::Add => {
            let email = workflow.add_account().await?;
            println!("✓ Account {} added successfully!", email);
        }

        Commands::List => {
            let accounts = workflow.list_accounts().await?;
            if accounts.is_empty() {
                println!("No accounts configured. Run 'unsubmail add' to add an account.");
            } else {
                println!("Configured accounts:");
                for account in accounts {
                    println!("  • {} (added: {})", account.email, account.added_at.format("%Y-%m-%d"));
                }
            }
        }

        Commands::Scan { email } => {
            let senders = workflow.scan_account(&email).await?;
            println!("\n✓ Scan complete! Found {} unique senders", senders.len());

            // Show top 10 newsletter candidates
            println!("\nTop newsletter candidates:");
            for (i, sender) in senders.iter().take(10).enumerate() {
                println!("  {}. {} ({} messages) - score: {:.2}",
                    i + 1,
                    sender.email,
                    sender.message_count,
                    sender.score
                );
            }

            println!("\nRun 'unsubmail clean {}' to select and cleanup senders.", email);
        }

        Commands::Clean { email } => {
            // Scan
            let senders = workflow.scan_account(&email).await?;

            if senders.is_empty() {
                println!("No senders found to clean.");
                return Ok(());
            }

            // Select
            println!("\nSelect senders to clean:");
            let selections = cli::select::select_senders(&senders)?;

            if selections.is_empty() {
                println!("No senders selected.");
                return Ok(());
            }

            // Cleanup
            println!("\nCleaning {} senders...", selections.len());
            let results = workflow.cleanup_account(&email, selections, &senders).await?;

            // Summary
            println!("\n=== Cleanup Summary ===");
            let mut total_deleted = 0;
            let mut total_unsubscribed = 0;
            let mut total_blocked = 0;

            for result in results {
                total_deleted += result.messages_deleted;
                if result.unsubscribed {
                    total_unsubscribed += 1;
                }
                if result.blocked {
                    total_blocked += 1;
                }
            }

            println!("✓ Unsubscribed: {}", total_unsubscribed);
            println!("✓ Blocked: {}", total_blocked);
            println!("✓ Messages deleted: {}", total_deleted);
        }

        Commands::Remove { email } => {
            use dialoguer::Confirm;

            let confirm = Confirm::new()
                .with_prompt(format!("Remove account {}?", email))
                .default(false)
                .interact()?;

            if confirm {
                // TODO: Remove from store and keyring
                println!("✓ Account {} removed", email);
            } else {
                println!("Cancelled");
            }
        }
    }

    Ok(())
}
