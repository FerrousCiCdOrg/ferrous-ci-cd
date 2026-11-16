//! Embedded SQL schema migrations for the supported database backends.
//!
//! The SQL files live under `migrations/postgres` and `migrations/sqlite` at the
//! repository root and are embedded into the binary at compile time. sqlx records
//! applied versions in the `_sqlx_migrations` table, so running the migrator again
//! against an up-to-date database is a no-op.

use sqlx::migrate::{MigrateError, Migrator};
use sqlx::{PgPool, SqlitePool};

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
