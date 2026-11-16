//! Embedded SQL schema migrations for the supported database backends.
//!
//! The SQL files live under `migrations/postgres` and `migrations/sqlite` at the
//! repository root and are embedded into the binary at compile time. sqlx records
//! applied versions in the `_sqlx_migrations` table, so running the migrator again
//! against an up-to-date database is a no-op.
//!
//! [`run_migrations`] is the backend-agnostic entry point used by the
//! `migrate` CLI subcommand and by server startup when
//! `database.auto_migrate` is enabled.

use std::future::Future;
use std::pin::Pin;

use sqlx::error::BoxDynError;
use sqlx::migrate::{Migrate, MigrateError, Migration, MigrationSource, Migrator};
use sqlx::{PgPool, SqlitePool};

use super::DatabasePool;
use crate::error::{Error, Result};

/// Migrations for PostgreSQL, embedded from `migrations/postgres`.
pub static POSTGRES_MIGRATOR: Migrator = sqlx::migrate!("./migrations/postgres");

/// Migrations for SQLite, embedded from `migrations/sqlite`.
pub static SQLITE_MIGRATOR: Migrator = sqlx::migrate!("./migrations/sqlite");

/// Apply all pending PostgreSQL migrations.
///
/// Already-applied migrations are skipped, so calling this repeatedly is safe.
///
/// # Errors
///
/// Returns [`Error::WithContext`] wrapping the underlying [`MigrateError`] when a
/// migration fails or a previously applied migration no longer matches its file.
pub async fn run_postgres_migrations(pool: &PgPool) -> Result<()> {
    POSTGRES_MIGRATOR
        .run(pool)
        .await
        .map_err(|e| migrate_error("PostgreSQL", e))
}

/// Apply all pending SQLite migrations.
///
/// Already-applied migrations are skipped, so calling this repeatedly is safe.
///
/// # Errors
///
/// Returns [`Error::WithContext`] wrapping the underlying [`MigrateError`] when a
/// migration fails or a previously applied migration no longer matches its file.
pub async fn run_sqlite_migrations(pool: &SqlitePool) -> Result<()> {
    SQLITE_MIGRATOR
        .run(pool)
        .await
        .map_err(|e| migrate_error("SQLite", e))
}

/// Outcome of [`run_migrations`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    /// Versions applied by this run, in ascending order. Empty when the
    /// database was already up to date.
    pub applied: Vec<i64>,
    /// Highest successfully applied version after the run, or `None` when the
    /// database has no applied migrations.
    pub current_version: Option<i64>,
}

/// Apply pending migrations for the backend of `pool`.
///
/// With `target` set, only migrations whose version is less than or equal to
/// `target` are applied. Downgrades are not supported: when the database
/// already has a migration newer than `target`, an error is returned and
/// nothing is changed. Without `target`, every pending migration is applied.
///
/// The whole run happens on one dedicated connection that holds the
/// migrator's database lock (a PostgreSQL advisory lock; SQLite needs none)
/// for its entire duration: creating `_sqlx_migrations`, reading the applied
/// versions before and after, and applying migrations. Concurrent callers,
/// such as several server replicas starting with `auto_migrate`, are
/// therefore serialized and see a consistent report. When locking is used,
/// the connection is closed afterwards instead of being returned to the pool,
/// so a lock can never leak to another pool user, even on error.
///
/// # Errors
///
/// Returns [`Error::Validation`] when `target` is not a known migration
/// version or is older than the database's current version, and
/// [`Error::WithContext`] wrapping the underlying [`MigrateError`] when a
/// migration fails.
pub async fn run_migrations(pool: &DatabasePool, target: Option<i64>) -> Result<MigrationReport> {
    match pool {
        DatabasePool::Postgres(pg) => {
            let migrator = migrator_up_to(&POSTGRES_MIGRATOR, target).await?;
            let mut conn = pg.acquire().await.map_err(acquire_error)?;
            if migrator.locking {
                conn.close_on_drop();
            }
            run_locked(&mut *conn, migrator, target, "PostgreSQL").await
        }
        DatabasePool::Sqlite(sqlite) => {
            let migrator = migrator_up_to(&SQLITE_MIGRATOR, target).await?;
            let mut conn = sqlite.acquire().await.map_err(acquire_error)?;
            if migrator.locking {
                conn.close_on_drop();
            }
            run_locked(&mut *conn, migrator, target, "SQLite").await
        }
    }
}

/// Runs `migrator` on `conn` while holding the migrator's lock, reading the
/// applied versions before and after the run under the same lock.
async fn run_locked<C: Migrate>(
    conn: &mut C,
    mut migrator: Migrator,
    target: Option<i64>,
    backend: &str,
) -> Result<MigrationReport> {
    let locking = migrator.locking;
    // The lock is taken here, so the migrator must not take it again.
    migrator.set_locking(false);

    if locking {
        conn.lock().await.map_err(|e| migrate_error(backend, e))?;
    }

    let result = run_under_lock(conn, &migrator, target, backend).await;

    if locking {
        if let Err(e) = conn.unlock().await {
            let unlock_err = migrate_error(backend, e);
            if result.is_ok() {
                return Err(unlock_err);
            }
            // Keep the original error; the connection is closed on drop,
            // which releases the lock anyway.
            tracing::warn!("failed to release migration lock: {unlock_err}");
        }
    }
    result
}

async fn run_under_lock<C: Migrate>(
    conn: &mut C,
    migrator: &Migrator,
    target: Option<i64>,
    backend: &str,
) -> Result<MigrationReport> {
    let before = applied_versions(conn)
        .await
        .map_err(|e| migrate_error(backend, e))?;
    check_not_ahead(&before, target)?;
    migrator
        .run_direct(conn)
        .await
        .map_err(|e| migrate_error(backend, e))?;
    let after = applied_versions(conn)
        .await
        .map_err(|e| migrate_error(backend, e))?;
    Ok(report(&before, after))
}

/// A [`MigrationSource`] backed by an in-memory list of migrations.
#[derive(Debug)]
struct StaticMigrations(Vec<Migration>);

impl MigrationSource<'static> for StaticMigrations {
    fn resolve(
        self,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<Vec<Migration>, BoxDynError>> + Send>>
    {
        Box::pin(async move { Ok(self.0) })
    }
}

/// Builds a migrator containing the migrations of `base` up to `target`.
async fn migrator_up_to(base: &Migrator, target: Option<i64>) -> Result<Migrator> {
    if let Some(target) = target {
        if !base.version_exists(target) {
            let known: Vec<String> = base.iter().map(|m| m.version.to_string()).collect();
            return Err(Error::validation(format!(
                "unknown migration version {target}; known versions: {}",
                known.join(", ")
            )));
        }
    }
    let migrations = base
        .iter()
        .filter(|m| target.is_none_or(|t| m.version <= t))
        .cloned()
        .collect();
    let mut migrator = Migrator::new(StaticMigrations(migrations))
        .await
        .map_err(|e| Error::WithContext {
            context: "failed to build the migration set for the requested target".to_string(),
            source: Box::new(e),
        })?;
    migrator.set_locking(base.locking);
    Ok(migrator)
}

/// Returns the successfully applied versions in ascending order.
///
/// Creates `_sqlx_migrations` when missing, so it must be called while the
/// migration lock is held.
async fn applied_versions<C: Migrate + ?Sized>(
    conn: &mut C,
) -> std::result::Result<Vec<i64>, MigrateError> {
    conn.ensure_migrations_table().await?;
    let mut versions: Vec<i64> = conn
        .list_applied_migrations()
        .await?
        .into_iter()
        .map(|m| m.version)
        .collect();
    versions.sort_unstable();
    Ok(versions)
}

fn check_not_ahead(applied: &[i64], target: Option<i64>) -> Result<()> {
    match (target, applied.last()) {
        (Some(target), Some(&current)) if current > target => Err(Error::validation(format!(
            "database is at migration version {current}, newer than target {target}; \
             downgrades are not supported"
        ))),
        _ => Ok(()),
    }
}

fn report(before: &[i64], after: Vec<i64>) -> MigrationReport {
    let current_version = after.last().copied();
    let applied = after
        .into_iter()
        .filter(|v| before.binary_search(v).is_err())
        .collect();
    MigrationReport {
        applied,
        current_version,
    }
}

fn acquire_error(err: sqlx::Error) -> Error {
    Error::WithContext {
        context: "failed to acquire a database connection for migrations".to_string(),
        source: Box::new(err),
    }
}

fn migrate_error(backend: &str, err: MigrateError) -> Error {
    Error::WithContext {
        context: format!("failed to run {backend} database migrations"),
        source: Box::new(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    /// Tables the initial schema must create.
    const EXPECTED_TABLES: &[&str] = &[
        "agents",
        "artifacts",
        "builds",
        "jobs",
        "pipelines",
        "projects",
        "stages",
        "users",
    ];

    async fn sqlite_pool(path: &std::path::Path) -> SqlitePool {
        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
            .expect("valid sqlite url")
            .create_if_missing(true)
            .foreign_keys(true);
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("connect to sqlite")
    }

    async fn sqlite_tables(pool: &SqlitePool) -> Vec<String> {
        sqlx::query_scalar::<_, String>(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' \
             AND name <> '_sqlx_migrations' ORDER BY name",
        )
        .fetch_all(pool)
        .await
        .expect("list tables")
    }

    #[tokio::test]
    async fn sqlite_migrations_apply_idempotently() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("ferrous.db");

        let pool = sqlite_pool(&path).await;
        run_sqlite_migrations(&pool)
            .await
            .expect("first migration run");
        run_sqlite_migrations(&pool)
            .await
            .expect("second migration run on the same pool");
        pool.close().await;

        // A fresh pool against the same file must also see an up-to-date schema.
        let pool = sqlite_pool(&path).await;
        run_sqlite_migrations(&pool)
            .await
            .expect("migration run on a reopened database");

        assert_eq!(sqlite_tables(&pool).await, EXPECTED_TABLES);

        let applied: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .expect("count applied migrations");
        assert_eq!(
            usize::try_from(applied).ok(),
            Some(SQLITE_MIGRATOR.iter().count())
        );
    }

    #[tokio::test]
    async fn sqlite_schema_enforces_build_number_uniqueness_and_cascades() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                SqliteConnectOptions::from_str("sqlite::memory:")
                    .expect("valid sqlite url")
                    .foreign_keys(true),
            )
            .await
            .expect("connect to in-memory sqlite");
        run_sqlite_migrations(&pool).await.expect("migrate");

        let now = "2026-11-16T00:00:00Z";
        sqlx::query(
            "INSERT INTO projects (id, name, repository_url, default_branch, visibility, settings, \
             created_at, updated_at) VALUES ('p1', 'demo', 'https://example.com/r.git', 'main', \
             'Private', '{}', ?1, ?1)",
        )
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert project");
        sqlx::query(
            "INSERT INTO pipelines (id, project_id, name, config, created_at, updated_at) \
             VALUES ('pl1', 'p1', 'ci', '{}', ?1, ?1)",
        )
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert pipeline");

        let insert_build = |id: &'static str| {
            sqlx::query(
                "INSERT INTO builds (id, pipeline_id, project_id, number, status, commit_sha, \
                 branch, trigger, created_at, updated_at) \
                 VALUES (?1, 'pl1', 'p1', 1, 'Pending', 'abc', 'main', '{\"type\":\"Push\"}', ?2, ?2)",
            )
            .bind(id)
            .bind(now)
        };
        insert_build("b1")
            .execute(&pool)
            .await
            .expect("first build");
        assert!(
            insert_build("b2").execute(&pool).await.is_err(),
            "duplicate build number per pipeline must be rejected"
        );

        sqlx::query("DELETE FROM projects WHERE id = 'p1'")
            .execute(&pool)
            .await
            .expect("delete project");
        let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM builds")
            .fetch_one(&pool)
            .await
            .expect("count builds");
        assert_eq!(remaining, 0, "deleting a project cascades to its builds");
    }

    /// A [`Migrate`] implementation that records the calls made on it.
    #[derive(Default)]
    struct RecordingConn {
        events: Vec<String>,
        applied: Vec<sqlx::migrate::AppliedMigration>,
        fail_apply: bool,
    }

    type Fut<'a, T> =
        Pin<Box<dyn Future<Output = std::result::Result<T, MigrateError>> + Send + 'a>>;

    impl Migrate for RecordingConn {
        fn ensure_migrations_table(&mut self) -> Fut<'_, ()> {
            self.events.push("ensure".into());
            Box::pin(async { Ok(()) })
        }
        fn dirty_version(&mut self) -> Fut<'_, Option<i64>> {
            self.events.push("dirty".into());
            Box::pin(async { Ok(None) })
        }
        fn list_applied_migrations(&mut self) -> Fut<'_, Vec<sqlx::migrate::AppliedMigration>> {
            self.events.push("list".into());
            let applied = self.applied.clone();
            Box::pin(async move { Ok(applied) })
        }
        fn lock(&mut self) -> Fut<'_, ()> {
            self.events.push("lock".into());
            Box::pin(async { Ok(()) })
        }
        fn unlock(&mut self) -> Fut<'_, ()> {
            self.events.push("unlock".into());
            Box::pin(async { Ok(()) })
        }
        fn apply<'e: 'm, 'm>(
            &'e mut self,
            migration: &'m Migration,
        ) -> Fut<'m, std::time::Duration> {
            self.events.push(format!("apply:{}", migration.version));
            if self.fail_apply {
                return Box::pin(async { Err(MigrateError::Execute(sqlx::Error::PoolClosed)) });
            }
            self.applied.push(sqlx::migrate::AppliedMigration {
                version: migration.version,
                checksum: migration.checksum.clone(),
            });
            Box::pin(async { Ok(std::time::Duration::ZERO) })
        }
        fn revert<'e: 'm, 'm>(
            &'e mut self,
            _migration: &'m Migration,
        ) -> Fut<'m, std::time::Duration> {
            unreachable!("migrations are never reverted")
        }
    }

    async fn locking_migrator() -> Migrator {
        let migrator = migrator_up_to(&SQLITE_MIGRATOR, None).await.unwrap();
        assert!(migrator.locking);
        migrator
    }

    #[tokio::test]
    async fn run_locked_reads_and_creates_table_only_under_the_lock() {
        let mut conn = RecordingConn::default();
        let report = run_locked(&mut conn, locking_migrator().await, None, "test")
            .await
            .unwrap();

        let events = &conn.events;
        assert_eq!(
            events.first().map(String::as_str),
            Some("lock"),
            "{events:?}"
        );
        assert_eq!(
            events.last().map(String::as_str),
            Some("unlock"),
            "{events:?}"
        );
        assert_eq!(
            events.iter().filter(|e| *e == "lock").count(),
            1,
            "{events:?}"
        );
        assert_eq!(
            events.iter().filter(|e| *e == "unlock").count(),
            1,
            "{events:?}"
        );
        // Table creation and both reads happen between lock and unlock.
        assert!(
            events.iter().filter(|e| *e == "ensure").count() >= 2,
            "{events:?}"
        );

        let all: Vec<i64> = SQLITE_MIGRATOR.iter().map(|m| m.version).collect();
        assert_eq!(report.applied, all);
        assert_eq!(report.current_version, all.last().copied());
    }

    #[tokio::test]
    async fn run_locked_releases_the_lock_when_a_migration_fails() {
        let mut conn = RecordingConn {
            fail_apply: true,
            ..RecordingConn::default()
        };
        let err = run_locked(&mut conn, locking_migrator().await, None, "test")
            .await
            .unwrap_err();
        assert!(matches!(err, Error::WithContext { .. }), "{err:?}");
        assert_eq!(conn.events.last().map(String::as_str), Some("unlock"));
    }

    #[tokio::test]
    async fn run_locked_releases_the_lock_when_target_is_rejected() {
        let latest = SQLITE_MIGRATOR.iter().map(|m| m.version).max().unwrap();
        let mut conn = RecordingConn::default();
        conn.applied.push(sqlx::migrate::AppliedMigration {
            version: latest + 1,
            checksum: Vec::new().into(),
        });
        let migrator = migrator_up_to(&SQLITE_MIGRATOR, Some(latest))
            .await
            .unwrap();
        let err = run_locked(&mut conn, migrator, Some(latest), "test")
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)), "{err:?}");
        assert!(!conn.events.iter().any(|e| e.starts_with("apply")));
        assert_eq!(conn.events.last().map(String::as_str), Some("unlock"));
    }

    #[tokio::test]
    async fn run_locked_without_locking_never_locks() {
        let mut migrator = locking_migrator().await;
        migrator.set_locking(false);
        let mut conn = RecordingConn::default();
        run_locked(&mut conn, migrator, None, "test").await.unwrap();
        assert!(!conn.events.iter().any(|e| e == "lock" || e == "unlock"));
    }

    #[tokio::test]
    #[ignore = "requires a PostgreSQL server; set DATABASE_URL to run"]
    async fn postgres_concurrent_migrations_are_serialized() {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = DatabasePool::Postgres(PgPool::connect(&url).await.expect("connect"));
        let runs = (0..3).map(|_| run_migrations(&pool, None));
        let reports = futures::future::join_all(runs).await;
        for report in &reports {
            assert!(report.is_ok(), "{report:?}");
        }
    }

    #[tokio::test]
    #[ignore = "requires a PostgreSQL server; set DATABASE_URL to run"]
    async fn postgres_migrations_apply_idempotently() {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&url).await.expect("connect to postgres");

        run_postgres_migrations(&pool).await.expect("first run");
        run_postgres_migrations(&pool).await.expect("second run");

        let tables: Vec<String> = sqlx::query_scalar(
            "SELECT table_name::text FROM information_schema.tables \
             WHERE table_schema = current_schema() AND table_name <> '_sqlx_migrations' \
             ORDER BY table_name",
        )
        .fetch_all(&pool)
        .await
        .expect("list tables");
        for table in EXPECTED_TABLES {
            assert!(tables.iter().any(|t| t == table), "missing table {table}");
        }
    }
}
