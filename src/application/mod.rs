//! Application layer - Workflow orchestration
//!
//! This module orchestrates the business logic from the domain layer with
//! external services from the infrastructure layer. It implements the main
//! application workflows for UnsubMail.
//!
//! # Architecture
//!
//! The application layer sits between the CLI/API layer and the domain layer.
//! It coordinates complex operations by:
//! - Calling domain logic for business decisions
//! - Using infrastructure adapters for external I/O
//! - Handling error propagation and logging
//! - Managing transaction boundaries
//!
//! # Modules
//!
//! - `workflow`: Main workflows (add account, scan inbox, clean inbox)
//!
//! # Design Principles
//!
//! - No direct I/O operations (delegated to infrastructure)
//! - No business logic (delegated to domain)
//! - Focus on orchestration and coordination

pub mod workflow;
