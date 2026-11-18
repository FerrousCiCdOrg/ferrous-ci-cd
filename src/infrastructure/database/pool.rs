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
/// Error messages contain the URL with credentials redacted, or no URL at
/// all when it cannot be parsed.
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
                .map_err(|e| invalid_url_error("PostgreSQL", &config.url, &e))?;
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
                .map_err(|e| invalid_url_error("SQLite", &config.url, &e))?
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
/// The user name and host are kept to aid diagnosis when that is safe.
/// Input without `://` that has no `@` and no credential query parameter
/// (for example `sqlite::memory:`) is returned unchanged; any other input
/// without `://` is replaced by `<redacted URL>`.
///
/// Secrets are not always percent-encoded in practice (for example a
/// generated secret interpolated into `postgresql://user:$(PASSWORD)@host` or
/// `...?password=$(PASSWORD)`), so they may contain `/`, `?`, `#`, `&`, `:` or
/// `@`. Redaction therefore works conservatively:
///
/// 1. The first credential query parameter (`password`, `sslpassword`,
///    `pass`, `passwd`, `pwd`, `secret`, `token`; matched case-insensitively
///    after `?` or `&`) is masked from its value up to the **end** of the URL,
///    so an unencoded `&`, `#` or `@` in the value never shows.
/// 2. In the part before it, the userinfo is taken to end at the **last**
///    `@`, and everything after the first `:` of the userinfo is masked.
/// 3. When the masked query value itself contains `@`, it is ambiguous
///    whether that `@` ends a userinfo, so everything after the first `:`
///    before the query parameter is masked too (this can hide a port).
///
/// This can over-mask, but it never reveals a password.
#[must_use]
pub fn redact_url(url: &str) -> String {
    const MASK: &str = "***";

    let Some(idx) = url.find("://") else {
        if url.contains('@') || find_secret_param(url).is_some() {
            return "<redacted URL>".to_string();
        }
        return url.to_string();
    };
    let (scheme, rest) = (&url[..idx + 3], &url[idx + 3..]);

    // Step 1: mask the first credential query parameter up to the end.
    let (prefix, secret_key, ambiguous) = match find_secret_param(rest) {
        Some((start, value_start)) => (
            &rest[..start],
            Some(&rest[start..value_start]),
            rest[value_start..].contains('@'),
        ),
        None => (rest, None, false),
    };
    let masked_tail = secret_key.map_or_else(String::new, |key| format!("{key}{MASK}"));

    // Step 3: the userinfo may extend into the masked value.
    if ambiguous {
        let kept = match prefix.find(':') {
            Some(colon) => format!("{}:{MASK}", &prefix[..colon]),
            None => prefix.to_string(),
        };
        return format!("{scheme}{kept}{masked_tail}");
    }

    // Step 2: the userinfo ends at the last `@`; the password is everything
    // after its first `:`.
    let redacted_prefix = match prefix.rfind('@') {
        Some(at) => {
            let (userinfo, host_and_tail) = prefix.split_at(at);
            match userinfo.find(':') {
                Some(colon) => format!("{}:{MASK}{host_and_tail}", &userinfo[..colon]),
                None => prefix.to_string(),
            }
        }
        None => prefix.to_string(),
    };
    format!("{scheme}{redacted_prefix}{masked_tail}")
}

/// Finds the first credential query parameter in `s`.
///
/// Returns the byte index of the `?`/`&` that introduces it and the byte
/// index where its value starts.
fn find_secret_param(s: &str) -> Option<(usize, usize)> {
    const SECRET_PARAMS: [&str; 7] = [
        "password",
        "sslpassword",
        "pass",
        "passwd",
        "pwd",
        "secret",
        "token",
    ];

    let bytes = s.as_bytes();
    bytes.iter().enumerate().find_map(|(i, &b)| {
        if b != b'?' && b != b'&' {
            return None;
        }
        let after = &s[i + 1..];
        SECRET_PARAMS.iter().find_map(|key| {
            let candidate = after.get(..key.len() + 1)?;
            let (name, eq) = candidate.split_at(key.len());
            (eq == "=" && name.eq_ignore_ascii_case(key)).then_some((i, i + 1 + key.len() + 1))
        })
    })
}

/// Builds the error for a URL the driver could not parse.
///
/// The URL itself is never echoed: if it could not be parsed, redaction
/// cannot be trusted to find the password either. Only the scheme is shown.
fn invalid_url_error(backend: &str, url: &str, err: &sqlx::Error) -> Error {
    let scheme = url
        .split_once(':')
        .map(|(scheme, _)| scheme)
        .filter(|s| {
            !s.is_empty()
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
        })
        .unwrap_or("<unknown>");
    Error::config(format!(
        "invalid {backend} URL (scheme `{scheme}`, <unparseable URL>): {err}"
    ))
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
            "postgres://localhost/db?user=a&password=***"
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

    #[test]
    fn redact_url_masks_passwords_with_unencoded_delimiters() {
        for password in ["pa/ss", "pa?ss", "pa#ss", "pa:ss", "pa@ss", "a/b?c#d:e@f"] {
            let url = format!("postgresql://ferrous:{password}@db.example.com:5432/ferrous");
            let redacted = redact_url(&url);
            assert_eq!(
                redacted, "postgresql://ferrous:***@db.example.com:5432/ferrous",
                "password {password:?}"
            );
            assert!(!redacted.contains(password), "password {password:?} leaked");
        }
    }

    #[test]
    fn redact_url_masks_unencoded_password_and_query_parameter_together() {
        assert_eq!(
            redact_url("postgres://u:p/w@db/x?sslmode=require&password=q#frag"),
            "postgres://u:***@db/x?sslmode=require&password=***"
        );
    }

    #[test]
    fn redact_url_masks_query_password_containing_at_sign() {
        let cases = [
            (
                "postgresql://ferrous@db/ferrous?password=pa@ss",
                "postgresql://ferrous@db/ferrous?password=***",
            ),
            (
                "postgresql://db/x?password=pa@ss&sslmode=require",
                "postgresql://db/x?password=***",
            ),
            (
                "postgresql://ferrous@db/ferrous?password=pa:ss@word",
                "postgresql://ferrous@db/ferrous?password=***",
            ),
            (
                "postgresql://ferrous@db:5432/ferrous?password=pa@ss",
                "postgresql://ferrous@db:***?password=***",
            ),
            (
                "postgresql://db/x?sslmode=require&SSLPassword=k@y",
                "postgresql://db/x?sslmode=require&SSLPassword=***",
            ),
        ];
        for (url, expected) in cases {
            let redacted = redact_url(url);
            assert_eq!(redacted, expected, "url {url:?}");
            assert!(!redacted.contains("ss@"), "secret leaked: {redacted}");
            assert!(!redacted.contains("@ss"), "secret leaked: {redacted}");
            assert!(!redacted.contains("ss@word"), "secret leaked: {redacted}");
        }
    }

    #[test]
    fn redact_url_masks_query_secret_up_to_the_end_of_the_url() {
        assert_eq!(
            redact_url("postgresql://db/x?password=pa&ss&sslmode=require"),
            "postgresql://db/x?password=***"
        );
        assert_eq!(
            redact_url("postgresql://db/x?password=pa#ss"),
            "postgresql://db/x?password=***"
        );
        assert_eq!(
            redact_url("postgresql://db/x?sslmode=require&pass=a&b#c@d"),
            "postgresql://db/x?sslmode=require&pass=***"
        );
    }

    #[test]
    fn redact_url_masks_userinfo_that_spans_a_query_like_password() {
        // The real password is `a@b?password=c`; nothing of it may show.
        let redacted = redact_url("postgresql://u:a@b?password=c@host/db");
        assert_eq!(redacted, "postgresql://u:***?password=***");
        let redacted = redact_url("postgresql://u:x?password=y@host/db");
        assert_eq!(redacted, "postgresql://u:***?password=***");
    }

    #[test]
    fn redact_url_without_scheme_separator_is_conservative() {
        assert_eq!(redact_url("sqlite::memory:"), "sqlite::memory:");
        assert_eq!(redact_url("user:secret@host/db"), "<redacted URL>");
        assert_eq!(redact_url("host/db?password=secret"), "<redacted URL>");
    }

    #[tokio::test]
    async fn connect_timeout_never_echoes_query_password_with_at_sign() {
        let config = DatabaseConfig {
            db_type: DatabaseType::Postgres,
            url: "postgresql://ferrous@127.0.0.1:1/ferrous?password=pa@ss".to_string(),
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
        assert!(!message.contains("pa@ss"), "password leaked: {message}");
        assert!(!message.contains("@ss"), "password leaked: {message}");
        assert!(message.contains("password=***"), "{message}");
    }
    #[tokio::test]
    async fn connect_with_unparseable_url_never_echoes_password() {
        for password in ["pa/ss", "pa?ss", "pa#ss"] {
            let config = DatabaseConfig {
                db_type: DatabaseType::Postgres,
                url: format!("postgres://ferrous:{password}@127.0.0.1:1/ferrous"),
                max_connections: 1,
                min_connections: 0,
                connection_timeout: 1,
                idle_timeout: 0,
                max_lifetime: 0,
                auto_migrate: false,
            };
            let err = connect(&config).await.unwrap_err();
            let message = err.to_string();
            assert!(matches!(err, Error::Config(_)), "unexpected error: {err:?}");
            assert!(
                !message.contains(password),
                "password {password:?} leaked: {message}"
            );
            assert!(
                !message.contains("ss@"),
                "password {password:?} leaked: {message}"
            );
            assert!(message.contains("<unparseable URL>"), "{message}");
            assert!(message.contains("`postgres`"), "{message}");
        }
    }
}
