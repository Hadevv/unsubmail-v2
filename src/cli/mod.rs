//! CLI layer - User interface and interaction
//!
//! This module provides the command-line interface for UnsubMail, including:
//! - Interactive TUI mode with progress indicators
//! - User prompts and confirmations
//! - Terminal output formatting
//!
//! # Architecture
//!
//! The CLI layer is the outermost layer in the clean architecture model.
//! It handles all user interaction and delegates business logic to the
//! application and domain layers.
//!
//! # Modules
//!
//! - `interactive`: Interactive terminal UI with guided workflows

pub mod interactive;
