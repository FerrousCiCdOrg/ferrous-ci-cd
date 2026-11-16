//! Config-driven database connection pool construction.

use std::str::FromStr;
use std::time::Duration;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{PgPool, SqlitePool};

use crate::config::{DatabaseConfig, DatabaseType};
use crate::error::{Error, Result};

/// A connection pool for one of the supported database backends.
#[derive(Debug, Clone)]
pub enum DatabasePool {
    /// PostgreSQL connection pool.
    Postgres(PgPool),
    /// SQLite connection pool.
    Sqlite(SqlitePool),
}

impl DatabasePool {
    /// Returns the backend type of this pool.
    #[must_use]
    pub fn db_type(&self) -> DatabaseType {
        match self {
            DatabasePool::Postgres(_) => DatabaseType::Postgres,
            DatabasePool::Sqlite(_) => DatabaseType::Sqlite,
        }
    }

    /// Returns the PostgreSQL pool, or `None` for another backend.
    #[must_use]
    pub fn as_postgres(&self) -> Option<&PgPool> {
        match self {
            DatabasePool::Postgres(pool) => Some(pool),
            DatabasePool::Sqlite(_) => None,
        }
    }

    /// Returns the SQLite pool, or `None` for another backend.
    #[must_use]
    pub fn as_sqlite(&self) -> Option<&SqlitePool> {
        match self {
            DatabasePool::Sqlite(pool) => Some(pool),
            DatabasePool::Postgres(_) => None,
        }
    }

    /// Closes every connection in the pool and waits for them to finish.
    pub async fn close(&self) {
        match self {
            DatabasePool::Postgres(pool) => pool.close().await,
            DatabasePool::Sqlite(pool) => pool.close().await,
        }
    }
}

/// Creates a connection pool from `config`.
///
/// The backend is selected from `config.db_type`. Pool limits and timeouts
/// come from the configuration: `max_connections`, `min_connections`,
/// `connection_timeout` (acquire timeout, which also bounds connection
/// establishment), `idle_timeout`, and `max_lifetime` (`0` disables the last
/// two). SQLite database files are created when missing.
///
/// # Errors
///
/// Returns [`Error::Config`] when the configuration is invalid, and
/// [`Error::Timeout`] or [`Error::Database`] when the pool cannot connect.
/// Error messages contain the URL with credentials redacted.
pub async fn connect(config: &DatabaseConfig) -> Result<DatabasePool> {
    config
        .validate()
        .map_err(|e| Error::config(format!("invalid database configuration: {e}")))?;

    let redacted = redact_url(&config.url);
    let acquire_timeout = Duration::from_secs(config.connection_timeout);
    let idle_timeout = non_zero_secs(config.idle_timeout);
    let max_lifetime = non_zero_secs(config.max_lifetime);

    match config.db_type {
        DatabaseType::Postgres => {
            let options = PgConnectOptions::from_str(&config.url)
                .map_err(|e| Error::config(format!("invalid PostgreSQL URL {redacted}: {e}")))?;
            let pool = PgPoolOptions::new()
                .max_connections(config.max_connections)
                .min_connections(config.min_connections)
                .acquire_timeout(acquire_timeout)
                .idle_timeout(idle_timeout)
                .max_lifetime(max_lifetime)
                .connect_with(options)
                .await
                .map_err(|e| connect_error("PostgreSQL", &redacted, e))?;
            Ok(DatabasePool::Postgres(pool))
        }
        DatabaseType::Sqlite => {
            let options = SqliteConnectOptions::from_str(&config.url)
                .map_err(|e| Error::config(format!("invalid SQLite URL {redacted}: {e}")))?
                .create_if_missing(true);
            let pool = SqlitePoolOptions::new()
                .max_connections(config.max_connections)
                .min_connections(config.min_connections)
                .acquire_timeout(acquire_timeout)
                .idle_timeout(idle_timeout)
                .max_lifetime(max_lifetime)
                .connect_with(options)
                .await
                .map_err(|e| connect_error("SQLite", &redacted, e))?;
            Ok(DatabasePool::Sqlite(pool))
        }
    }
}

/// Returns `url` with any password and credential query parameters replaced
/// by `***`, so it can be safely logged or embedded in error messages.
///
/// The user name is kept to aid diagnosis. Input that does not look like a
/// URL is returned unchanged.
#[must_use]
pub fn redact_url(url: &str) -> String {
    const MASK: &str = "***";
    const SECRET_PARAMS: [&str; 3] = ["password", "sslpassword", "pass"];

    let (scheme, rest) = match url.find("://") {
        Some(idx) => (&url[..idx + 3], &url[idx + 3..]),
        None => ("", url),
    };

    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (authority, tail) = rest.split_at(authority_end);

    let authority = match (scheme.is_empty(), authority.rfind('@')) {
        (false, Some(at)) => {
            let (userinfo, host) = authority.split_at(at);
            match userinfo.find(':') {
                Some(colon) => format!("{}:{MASK}{host}", &userinfo[..colon]),
                None => authority.to_string(),
            }
        }
        _ => authority.to_string(),
    };

    let tail = match tail.find('?') {
        Some(q) => {
            let (path, query_and_fragment) = tail.split_at(q);
            let query_and_fragment = &query_and_fragment[1..];
            let (query, fragment) = match query_and_fragment.find('#') {
                Some(h) => query_and_fragment.split_at(h),
                None => (query_and_fragment, ""),
            };
            let query = query
                .split('&')
                .map(|pair| match pair.split_once('=') {
                    Some((key, _))
                        if SECRET_PARAMS.contains(&key.to_ascii_lowercase().as_str()) =>
                    {
                        format!("{key}={MASK}")
                    }
                    _ => pair.to_string(),
                })
                .collect::<Vec<_>>()
                .join("&");
            format!("{path}?{query}{fragment}")
        }
        None => tail.to_string(),
    };

    format!("{scheme}{authority}{tail}")
}

fn non_zero_secs(secs: u64) -> Option<Duration> {
    (secs > 0).then(|| Duration::from_secs(secs))
}

fn connect_error(backend: &str, redacted_url: &str, err: sqlx::Error) -> Error {
    match err {
        sqlx::Error::PoolTimedOut => Error::timeout(format!(
            "timed out connecting to {backend} database at {redacted_url}"
        )),
        other => Error::database(format!(
            "failed to connect to {backend} database at {redacted_url}: {other}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sqlite_config(url: &str) -> DatabaseConfig {
        DatabaseConfig {
            db_type: DatabaseType::Sqlite,
            url: url.to_string(),
            max_connections: 3,
            min_connections: 1,
            connection_timeout: 2,
            idle_timeout: 60,
            max_lifetime: 0,
            auto_migrate: false,
        }
    }

    #[tokio::test]
    async fn connect_sqlite_in_memory_applies_limits() {
        let pool = connect(&sqlite_config("sqlite::memory:")).await.unwrap();
        assert_eq!(pool.db_type(), DatabaseType::Sqlite);
        assert!(pool.as_postgres().is_none());

        let sqlite = pool.as_sqlite().expect("sqlite pool");
        assert_eq!(sqlite.options().get_max_connections(), 3);
        assert_eq!(sqlite.options().get_min_connections(), 1);
        assert_eq!(
            sqlite.options().get_acquire_timeout(),
            Duration::from_secs(2)
        );
        assert_eq!(
            sqlite.options().get_idle_timeout(),
            Some(Duration::from_secs(60))
        );
        assert_eq!(sqlite.options().get_max_lifetime(), None);

        let one: i64 = sqlx::query_scalar("SELECT 1")
            .fetch_one(sqlite)
            .await
            .unwrap();
        assert_eq!(one, 1);
        pool.close().await;
    }

    #[tokio::test]
    async fn connect_sqlite_creates_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ferrous.db");
        assert!(!path.exists());

        let url = format!("sqlite://{}", path.display());
        let pool = connect(&sqlite_config(&url)).await.unwrap();
        assert!(path.exists());
        pool.close().await;
    }

    #[tokio::test]
    async fn connect_sqlite_invalid_path_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let url = format!(
            "sqlite://{}",
            dir.path().join("missing-dir").join("ferrous.db").display()
        );
        let err = connect(&sqlite_config(&url)).await.unwrap_err();
        assert!(
            matches!(err, Error::Database(_) | Error::Timeout(_)),
            "unexpected error: {err:?}"
        );
    }

    #[tokio::test]
    async fn connect_unreachable_postgres_returns_redacted_error() {
        let config = DatabaseConfig {
            db_type: DatabaseType::Postgres,
            url: "postgres://ferrous:s3cret@127.0.0.1:1/ferrous".to_string(),
            max_connections: 1,
            min_connections: 0,
            connection_timeout: 1,
            idle_timeout: 0,
            max_lifetime: 0,
            auto_migrate: false,
        };
        let err = connect(&config).await.unwrap_err();
        assert!(
            matches!(err, Error::Database(_) | Error::Timeout(_)),
            "unexpected error: {err:?}"
        );
        let message = err.to_string();
        assert!(!message.contains("s3cret"), "password leaked: {message}");
        assert!(message.contains("ferrous:***@127.0.0.1:1"), "{message}");
    }

    #[tokio::test]
    async fn connect_rejects_invalid_config() {
        let mut config = sqlite_config("sqlite::memory:");
        config.min_connections = 10;
        let err = connect(&config).await.unwrap_err();
        assert!(matches!(err, Error::Config(_)), "unexpected error: {err:?}");
    }

    #[test]
    fn redact_url_masks_password() {
        assert_eq!(
            redact_url("postgresql://user:p%40ss@db.example.com:5432/ferrous"),
            "postgresql://user:***@db.example.com:5432/ferrous"
        );
    }

    #[test]
    fn redact_url_masks_password_containing_at_sign() {
        assert_eq!(
            redact_url("postgres://user:p@ss@localhost/db"),
            "postgres://user:***@localhost/db"
        );
    }

    #[test]
    fn redact_url_masks_password_query_parameter() {
        assert_eq!(
            redact_url("postgres://localhost/db?user=a&password=secret&sslmode=require#x"),
            "postgres://localhost/db?user=a&password=***&sslmode=require#x"
        );
    }

    #[test]
    fn redact_url_keeps_urls_without_credentials() {
        assert_eq!(
            redact_url("postgres://user@localhost/db"),
            "postgres://user@localhost/db"
        );
        assert_eq!(redact_url("sqlite::memory:"), "sqlite::memory:");
        assert_eq!(
            redact_url("sqlite:///var/lib/ferrous/ferrous.db"),
            "sqlite:///var/lib/ferrous/ferrous.db"
        );
    }
}
