//! CLI handlers
//!
//! Database handlers back the `migrate` subcommand and the automatic
//! migration step that runs before the server starts. They delegate to the
//! application-layer [`migration`] use case and only report the outcome.

use tracing::info;

use crate::application::use_cases::migration::{self, SchemaMigrationReport};
use crate::config::DatabaseConfig;
use crate::error::Result;

/// Handles `ferrous-ci-cd migrate [--target <VERSION>]`.
///
/// Applies pending migrations up to `target` (or all of them when `target` is
/// `None`) through [`migration::migrate_database`] and logs the outcome.
/// Running it against an up-to-date database is a no-op.
///
/// # Errors
///
/// Returns an error when the configuration is invalid, the database cannot be
/// reached, `target` is unknown or older than the current schema version, or
/// a migration fails.
pub async fn migrate(
    config: &DatabaseConfig,
    target: Option<i64>,
) -> Result<SchemaMigrationReport> {
    let report = migration::migrate_database(config, target).await?;
    log_report(&report);
    Ok(report)
}

/// Applies pending migrations before the server starts when
/// `database.auto_migrate` is enabled.
///
/// Returns `Ok(None)` without touching the database when auto migration is
/// disabled.
///
/// # Errors
///
/// Returns the same errors as [`migrate`]; the server must not start on an
/// error.
pub async fn auto_migrate(config: &DatabaseConfig) -> Result<Option<SchemaMigrationReport>> {
    let report = migration::auto_migrate_database(config).await?;
    if let Some(report) = &report {
        log_report(report);
    }
    Ok(report)
}

fn log_report(report: &SchemaMigrationReport) {
    let current = report
        .current_version
        .map_or_else(|| "none".to_string(), |v| v.to_string());
    if report.applied.is_empty() {
        info!("Database schema is up to date (version {current})");
    } else {
        info!(
            "Applied {} migration(s) {:?}; database schema is at version {current}",
            report.applied.len(),
            report.applied
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseType;

    fn sqlite_config(path: &std::path::Path, auto_migrate: bool) -> DatabaseConfig {
        DatabaseConfig {
            db_type: DatabaseType::Sqlite,
            url: format!("sqlite://{}", path.display()),
            max_connections: 2,
            min_connections: 0,
            connection_timeout: 5,
            idle_timeout: 0,
            max_lifetime: 0,
            auto_migrate,
        }
    }

    #[tokio::test]
    async fn migrate_delegates_to_the_use_case() {
        let dir = tempfile::tempdir().unwrap();
        let config = sqlite_config(&dir.path().join("ferrous.db"), false);

        let first = migrate(&config, None).await.unwrap();
        assert!(!first.applied.is_empty());
        let second = migrate(&config, None).await.unwrap();
        assert!(second.applied.is_empty());
        assert_eq!(second.current_version, first.current_version);
    }

    #[tokio::test]
    async fn auto_migrate_respects_the_flag() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ferrous.db");
        assert!(auto_migrate(&sqlite_config(&path, false))
            .await
            .unwrap()
            .is_none());
        assert!(!path.exists());
        assert!(auto_migrate(&sqlite_config(&path, true))
            .await
            .unwrap()
            .is_some());
    }
}
