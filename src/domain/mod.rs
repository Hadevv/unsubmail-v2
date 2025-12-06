//! Domain layer - Pure business logic
//!
//! This module contains the pure business logic for UnsubMail. All code in this
//! layer is independent of external dependencies and frameworks.
//!
//! # Architecture
//!
//! The domain layer is the heart of the application. It contains:
//! - Business models and entities
//! - Business rules and validation
//! - Domain services and algorithms
//!
//! **Important**: This layer has NO dependencies on infrastructure or application
//! layers. It uses only standard library types and domain-specific abstractions.
//!
//! # Modules
//!
//! - `models`: Core data structures (EmailAccount, SenderInfo, etc.)
//! - `analysis`: Newsletter detection and email analysis heuristics
//! - `planner`: Cleanup action planning and strategy selection
//!
//! # Design Principles
//!
//! - **No I/O**: All functions are pure or use dependency injection
//! - **Testable**: Easy to test without mocks or external services
//! - **Type-safe**: Leverage Rust's type system for correctness
//! - **Single Responsibility**: Each module has one clear purpose

pub mod models;
pub mod analysis;
pub mod planner;
