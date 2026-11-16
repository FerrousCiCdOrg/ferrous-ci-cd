//! Database schema migration use case.
//!
//! This is the application-layer entry point for applying the embedded schema
//! migrations, used by the `migrate` CLI subcommand and by server startup when
//! `database.auto_migrate` is enabled. Presentation code calls these functions
//! instead of reaching into `infrastructure::database` directly.
//!
//! Like [`crate::application::Application::new`], this module is part of the
//! application's composition root: it wires the infrastructure connection pool
//! and migrator, because schema migration is an operational bootstrap step
//! with no domain repository interface to depend on.

use tracing::info;

use crate::config::DatabaseConfig;
use crate::error::Result;
use crate::infrastructure::database::{self, redact_url};

/// Outcome of a schema migration run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaMigrationReport {
    /// Versions applied by this run, in ascending order. Empty when the
    /// database was already up to date.
    pub applied: Vec<i64>,
    /// Highest successfully applied version after the run, or `None` when the
    /// database has no applied migrations.
    pub current_version: Option<i64>,
}

impl From<database::MigrationReport> for SchemaMigrationReport {
    fn from(report: database::MigrationReport) -> Self {
        Self {
            applied: report.applied,
            current_version: report.current_version,
        }
    }
}

/// Connects to the configured database, applies pending migrations up to
/// `target` (or all of them when `target` is `None`), and closes the pool.
///
/// Running it against an up-to-date database is a no-op. Concurrent runs
/// against the same PostgreSQL database are serialized by the migrator lock.
///
/// # Errors
///
/// Returns an error when the configuration is invalid, the database cannot be
/// reached, `target` is unknown or older than the current schema version, or
/// a migration fails.
pub async fn migrate_database(
    config: &DatabaseConfig,
    target: Option<i64>,
) -> Result<SchemaMigrationReport> {
    info!(
        "Running {:?} database migrations against {}{}",
        config.db_type,
        redact_url(&config.url),
        target.map_or_else(String::new, |t| format!(" (target {t})"))
    );
    let pool = database::connect(config).await?;
    let result = database::run_migrations(&pool, target).await;
    pool.close().await;
    result.map(SchemaMigrationReport::from)
}

/// Applies all pending migrations when `database.auto_migrate` is enabled.
///
/// Returns `Ok(None)` without touching the database when auto migration is
/// disabled.
///
/// # Errors
///
/// Returns the same errors as [`migrate_database`].
pub async fn auto_migrate_database(
    config: &DatabaseConfig,
) -> Result<Option<SchemaMigrationReport>> {
    if !config.auto_migrate {
        info!("Automatic database migration is disabled (database.auto_migrate = false)");
        return Ok(None);
    }
    migrate_database(config, None).await.map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseType;
    use crate::error::Error;
    use crate::infrastructure::database::migrations::SQLITE_MIGRATOR;

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

    fn latest_version() -> i64 {
        SQLITE_MIGRATOR
            .iter()
            .map(|m| m.version)
            .max()
            .expect("at least one migration")
    }

    #[tokio::test]
    async fn migrate_applies_all_then_is_a_no_op() {
        let dir = tempfile::tempdir().unwrap();
        let config = sqlite_config(&dir.path().join("ferrous.db"), false);

        let first = migrate_database(&config, None).await.unwrap();
        assert_eq!(first.applied.len(), SQLITE_MIGRATOR.iter().count());
        assert_eq!(first.current_version, Some(latest_version()));

        let second = migrate_database(&config, None).await.unwrap();
        assert!(second.applied.is_empty());
        assert_eq!(second.current_version, Some(latest_version()));
    }

    #[tokio::test]
    async fn migrate_to_latest_target_succeeds_and_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let config = sqlite_config(&dir.path().join("ferrous.db"), false);
        let latest = latest_version();

        let report = migrate_database(&config, Some(latest)).await.unwrap();
        assert_eq!(report.current_version, Some(latest));
        assert!(report.applied.contains(&latest));

        let again = migrate_database(&config, Some(latest)).await.unwrap();
        assert!(again.applied.is_empty());
    }

    #[tokio::test]
    async fn migrate_rejects_unknown_target() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ferrous.db");
        let err = migrate_database(&sqlite_config(&path, false), Some(1))
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)), "unexpected: {err:?}");
    }

    #[tokio::test]
    async fn migrate_rejects_target_older_than_database() {
        let dir = tempfile::tempdir().unwrap();
        let config = sqlite_config(&dir.path().join("ferrous.db"), false);
        let latest = latest_version();
        migrate_database(&config, None).await.unwrap();

        // Pretend a newer migration was applied by another build.
        let pool = database::connect(&config).await.unwrap();
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, success, checksum, \
             execution_time) VALUES (?1, 'future', 1, x'00', 0)",
        )
        .bind(latest + 1)
        .execute(pool.as_sqlite().unwrap())
        .await
        .unwrap();
        pool.close().await;

        let err = migrate_database(&config, Some(latest)).await.unwrap_err();
        assert!(matches!(err, Error::Validation(_)), "unexpected: {err:?}");
        assert!(err.to_string().contains("downgrades are not supported"));
    }

    #[tokio::test]
    async fn auto_migrate_disabled_does_not_touch_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ferrous.db");
        let report = auto_migrate_database(&sqlite_config(&path, false))
            .await
            .unwrap();
        assert!(report.is_none());
        assert!(!path.exists(), "database must not be created");
    }

    #[tokio::test]
    async fn auto_migrate_enabled_applies_migrations() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ferrous.db");
        let report = auto_migrate_database(&sqlite_config(&path, true))
            .await
            .unwrap()
            .expect("migrations ran");
        assert_eq!(report.current_version, Some(latest_version()));
    }

    #[tokio::test]
    async fn migrate_reports_connection_failure() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing-dir").join("ferrous.db");
        let err = migrate_database(&sqlite_config(&path, false), None)
            .await
            .unwrap_err();
        assert!(
            matches!(err, Error::Database(_) | Error::Timeout(_)),
            "unexpected: {err:?}"
        );
    }
}
