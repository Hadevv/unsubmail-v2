//! Infrastructure layer - External services and adapters
//!
//! This module provides adapters for external services and systems. It implements
//! the ports defined by the application and domain layers.
//!
//! # Architecture
//!
//! The infrastructure layer handles all I/O operations and external dependencies:
//! - IMAP/email services
//! - Storage (keyring, filesystem)
//! - Network operations (HTTP)
//!
//! All external dependencies should be abstracted through this layer to keep
//! the domain and application layers pure and testable.
//!
//! # Modules
//!
//! - `imap`: IMAP client for Gmail (connection, authentication, message operations)
//! - `storage`: Data persistence (keyring for tokens, JSON for metadata)
//! - `network`: HTTP client for one-click unsubscribe operations
//!
//! # Design Principles
//!
//! - **Adapter Pattern**: Wrap external libraries with domain-friendly interfaces
//! - **Error Handling**: Convert external errors to domain errors
//! - **Async/Await**: All I/O operations are asynchronous
//! - **Testability**: Support mock implementations for testing

pub mod imap;
pub mod network;
pub mod storage;
