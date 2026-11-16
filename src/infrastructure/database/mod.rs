//! Database connection pool and migrations.
//!
//! [`connect`] builds a [`DatabasePool`] from [`crate::config::DatabaseConfig`],
//! choosing PostgreSQL or SQLite from the configured database type.
//! [`migrations`] applies the embedded schema migrations for either backend.

pub mod migrations;
pub mod pool;

pub use pool::{connect, redact_url, DatabasePool};
