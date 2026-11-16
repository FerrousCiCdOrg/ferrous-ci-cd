//! CLI handlers
//!
//! Database handlers back the `migrate` subcommand and the automatic
//! migration step that runs before the server starts.

use tracing::info;

use crate::config::DatabaseConfig;
use crate::error::Result;
use crate::infrastructure::database::{self, redact_url, MigrationReport};

/// Handles `ferrous-ci-cd migrate [--target <VERSION>]`.
///
/// Connects to the configured database, applies pending migrations up to
/// `target` (or all of them when `target` is `None`), and closes the pool.
/// Running it against an up-to-date database is a no-op.
///
/// # Errors
///
/// Returns an error when the configuration is invalid, the database cannot be
/// reached, `target` is unknown or older than the current schema version, or
/// a migration fails.
pub async fn migrate(config: &DatabaseConfig, target: Option<i64>) -> Result<MigrationReport> {
    info!(
        "Running {:?} database migrations against {}{}",
        config.db_type,
        redact_url(&config.url),
        target.map_or_else(String::new, |t| format!(" (target {t})"))
    );
    let pool = database::connect(config).await?;
    let result = database::run_migrations(&pool, target).await;
    pool.close().await;
    let report = result?;
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
pub async fn auto_migrate(config: &DatabaseConfig) -> Result<Option<MigrationReport>> {
    if !config.auto_migrate {
        info!("Automatic database migration is disabled (database.auto_migrate = false)");
        return Ok(None);
    }
    migrate(config, None).await.map(Some)
}

fn log_report(report: &MigrationReport) {
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

        let first = migrate(&config, None).await.unwrap();
        assert_eq!(first.applied.len(), SQLITE_MIGRATOR.iter().count());
        assert_eq!(first.current_version, Some(latest_version()));

        let second = migrate(&config, None).await.unwrap();
        assert!(second.applied.is_empty());
        assert_eq!(second.current_version, Some(latest_version()));
    }

    #[tokio::test]
    async fn migrate_to_latest_target_succeeds_and_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let config = sqlite_config(&dir.path().join("ferrous.db"), false);
        let latest = latest_version();

        let report = migrate(&config, Some(latest)).await.unwrap();
        assert_eq!(report.current_version, Some(latest));
        assert!(report.applied.contains(&latest));

        let again = migrate(&config, Some(latest)).await.unwrap();
        assert!(again.applied.is_empty());
    }

    #[tokio::test]
    async fn migrate_rejects_unknown_target() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ferrous.db");
        let err = migrate(&sqlite_config(&path, false), Some(1))
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)), "unexpected: {err:?}");
    }

    #[tokio::test]
    async fn migrate_rejects_target_older_than_database() {
        let dir = tempfile::tempdir().unwrap();
        let config = sqlite_config(&dir.path().join("ferrous.db"), false);
        let latest = latest_version();
        migrate(&config, None).await.unwrap();

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

        let err = migrate(&config, Some(latest)).await.unwrap_err();
        assert!(matches!(err, Error::Validation(_)), "unexpected: {err:?}");
        assert!(err.to_string().contains("downgrades are not supported"));
    }

    #[tokio::test]
    async fn auto_migrate_disabled_does_not_touch_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ferrous.db");
        let report = auto_migrate(&sqlite_config(&path, false)).await.unwrap();
        assert!(report.is_none());
        assert!(!path.exists(), "database must not be created");
    }

    #[tokio::test]
    async fn auto_migrate_enabled_applies_migrations() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ferrous.db");
        let report = auto_migrate(&sqlite_config(&path, true))
            .await
            .unwrap()
            .expect("migrations ran");
        assert_eq!(report.current_version, Some(latest_version()));
    }

    #[tokio::test]
    async fn migrate_reports_connection_failure() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing-dir").join("ferrous.db");
        let err = migrate(&sqlite_config(&path, false), None)
            .await
            .unwrap_err();
        assert!(
            matches!(err, Error::Database(_) | Error::Timeout(_)),
            "unexpected: {err:?}"
        );
    }
}
