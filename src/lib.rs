//! Ferrous CI/CD - A modern CI/CD system built with Rust
//!
//! This library provides the core functionality for the Ferrous CI/CD system,
//! implementing Domain-Driven Design principles for a clean and maintainable architecture.

#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(missing_docs)] // TODO: Add comprehensive documentation

pub mod application;
pub mod config;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod presentation;

// Re-export commonly used types
pub use config::Config;
pub use error::{Error, Result};

/// The current version of Ferrous CI/CD
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the Ferrous CI/CD system
///
/// This function sets up logging, loads configuration, and prepares
/// the system for use.
///
/// # Errors
///
/// Returns an error if initialization fails
pub async fn init() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ferrous_ci_cd=info".parse().unwrap()),
        )
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    tracing::info!("Initializing Ferrous CI/CD v{}", VERSION);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[tokio::test]
    async fn test_init() {
        let result = init().await;
        assert!(result.is_ok());
    }
}
