use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt};
use unsubmail::cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env.local or .env
    dotenvy::from_filename(".env.local")
        .or_else(|_| dotenvy::dotenv())
        .ok(); // Ignore if no .env file exists

    // Initialize logging
    fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("unsubmail=info".parse()?))
        .init();

    // Always run interactive mode
    cli::interactive::run_interactive().await
}
