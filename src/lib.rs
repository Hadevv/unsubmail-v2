//! UnsubMail - Clean your Gmail inbox from newsletters and spam
//!
//! This library provides the core functionality for detecting and removing
//! newsletters and promotional emails from Gmail inboxes.
//!
//! # Overview
//!
//! UnsubMail uses smart heuristics to identify newsletters and promotional emails,
//! offering one-click unsubscribe when available, and helping you bulk clean your
//! Gmail account safely and efficiently.
//!
//! # Architecture
//!
//! UnsubMail follows clean architecture principles with four main layers:
//!
//! 1. **Domain Layer** (`domain`): Pure business logic, no dependencies
//!    - Models and entities
//!    - Newsletter detection heuristics
//!    - Action planning
//!
//! 2. **Application Layer** (`application`): Workflow orchestration
//!    - Coordinates domain logic with infrastructure
//!    - Main application workflows
//!    - Error handling and logging
//!
//! 3. **Infrastructure Layer** (`infrastructure`): External services
//!    - IMAP client (Gmail)
//!    - Storage (keyring, JSON)
//!    - HTTP client (unsubscribe)
//!
//! 4. **CLI Layer** (`cli`): User interface
//!    - Interactive terminal UI
//!    - Command-line interface
//!    - Progress indicators
//!
//! # Features
//!
//! - **Smart Detection**: RFC 2369 List-Unsubscribe headers + pattern matching
//! - **One-Click Unsubscribe**: RFC 8058 automated unsubscribe support
//! - **OAuth2 Authentication**: Secure Gmail access without passwords
//! - **Fast Scanning**: Headers-only, no full message downloads
//! - **Batch Operations**: Process hundreds of messages efficiently
//! - **Safe**: Review before deletion, secure token storage
//!
//! # Example
//!
//! ```no_run
//! use unsubmail::cli::interactive;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Run interactive mode
//!     interactive::run_interactive().await?;
//!     Ok(())
//! }
//! ```
//!
//! # Security
//!
//! - OAuth2 authentication only (no passwords)
//! - Tokens stored in OS keyring (Keychain/Credential Manager/Secret Service)
//! - HTTPS-only unsubscribe links
//! - No message bodies accessed, only headers

pub mod application;
pub mod cli;
pub mod domain;
pub mod infrastructure;
