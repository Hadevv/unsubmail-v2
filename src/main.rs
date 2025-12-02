mod application;
mod cli;
mod domain;
mod infrastructure;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "unsubmail")]
#[command(about = "Clean your Gmail inbox from newsletters and spam", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a Gmail account via OAuth2
    AddAccount,

    /// Scan inbox for newsletters and spam
    Scan {
        /// Account email to scan (optional, uses default if not specified)
        #[arg(short, long)]
        account: Option<String>,
    },

    /// List configured accounts
    ListAccounts,

    /// Remove an account
    RemoveAccount {
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

    match cli.command {
        Commands::AddAccount => {
            cli::accounts::add_account().await?;
        }
        Commands::Scan { account } => {
            cli::scan::scan_inbox(account).await?;
        }
        Commands::ListAccounts => {
            cli::accounts::list_accounts().await?;
        }
        Commands::RemoveAccount { email } => {
            cli::accounts::remove_account(&email).await?;
        }
    }

    Ok(())
}
