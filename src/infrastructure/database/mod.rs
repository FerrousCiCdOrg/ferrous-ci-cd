//! Database connection pool and migrations.
//!
//! [`connect`] builds a [`DatabasePool`] from [`crate::config::DatabaseConfig`],
//! choosing PostgreSQL or SQLite from the configured database type.

pub mod pool;

pub use pool::{connect, redact_url, DatabasePool};
