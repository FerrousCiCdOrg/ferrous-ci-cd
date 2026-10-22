//! Error handling for Ferrous CI/CD

use thiserror::Error;

/// Main error type for Ferrous CI/CD
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Repository error (domain repositories)
    #[error("Repository error: {0}")]
    Repository(String),

    /// Domain error
    #[error("Domain error: {0}")]
    Domain(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Authorization error
    #[error("Authorization error: {0}")]
    Authorization(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// Conflict error
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Build error
    #[error("Build error: {0}")]
    Build(String),

    /// Pipeline error
    #[error("Pipeline error: {0}")]
    Pipeline(String),

    /// Agent error
    #[error("Agent error: {0}")]
    Agent(String),

    /// Git operation error
    #[error("Git error: {0}")]
    Git(String),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// External service error
    #[error("External service error: {0}")]
    ExternalService(String),

    /// Timeout error
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Rate limit error
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error with context
    #[error("{context}: {source}")]
    WithContext {
        context: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl Error {
    /// Create a configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Error::Config(msg.into())
    }

    /// Create a database error
    pub fn database(msg: impl Into<String>) -> Self {
        Error::Database(msg.into())
    }

    /// Create a repository error
    pub fn repository(msg: impl Into<String>) -> Self {
        Error::Repository(msg.into())
    }

    /// Create a domain error
    pub fn domain(msg: impl Into<String>) -> Self {
        Error::Domain(msg.into())
    }

    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Error::Validation(msg.into())
    }

    /// Create an authentication error
    pub fn authentication(msg: impl Into<String>) -> Self {
        Error::Authentication(msg.into())
    }

    /// Create an authorization error
    pub fn authorization(msg: impl Into<String>) -> Self {
        Error::Authorization(msg.into())
    }

    /// Create a not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        Error::NotFound(msg.into())
    }

    /// Create a conflict error
    pub fn conflict(msg: impl Into<String>) -> Self {
        Error::Conflict(msg.into())
    }

    /// Create a build error
    pub fn build(msg: impl Into<String>) -> Self {
        Error::Build(msg.into())
    }

    /// Create a pipeline error
    pub fn pipeline(msg: impl Into<String>) -> Self {
        Error::Pipeline(msg.into())
    }

    /// Create an agent error
    pub fn agent(msg: impl Into<String>) -> Self {
        Error::Agent(msg.into())
    }

    /// Create a git error
    pub fn git(msg: impl Into<String>) -> Self {
        Error::Git(msg.into())
    }

    /// Create a storage error
    pub fn storage(msg: impl Into<String>) -> Self {
        Error::Storage(msg.into())
    }

    /// Create a network error
    pub fn network(msg: impl Into<String>) -> Self {
        Error::Network(msg.into())
    }

    /// Create an external service error
    pub fn external_service(msg: impl Into<String>) -> Self {
        Error::ExternalService(msg.into())
    }

    /// Create a timeout error
    pub fn timeout(msg: impl Into<String>) -> Self {
        Error::Timeout(msg.into())
    }

    /// Create a rate limit error
    pub fn rate_limit(msg: impl Into<String>) -> Self {
        Error::RateLimit(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Error::Internal(msg.into())
    }

    /// Create a serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        Error::Serialization(msg.into())
    }

    /// Add context to an error
    pub fn context(self, context: impl Into<String>) -> Self {
        Error::WithContext {
            context: context.into(),
            source: Box::new(self),
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::Network(_) | Error::Timeout(_) | Error::ExternalService(_)
        )
    }

    /// Get HTTP status code for the error
    pub fn status_code(&self) -> u16 {
        match self {
            Error::Validation(_) => 400,
            Error::Authentication(_) => 401,
            Error::Authorization(_) => 403,
            Error::NotFound(_) => 404,
            Error::Conflict(_) => 409,
            Error::RateLimit(_) => 429,
            Error::Timeout(_) => 408,
            Error::Network(_) | Error::ExternalService(_) => 502,
            _ => 500,
        }
    }
}

/// Result type alias for Ferrous CI/CD
pub type Result<T> = std::result::Result<T, Error>;

/// Extension trait for adding context to results
pub trait ResultExt<T> {
    /// Add context to an error
    fn context(self, context: impl Into<String>) -> Result<T>;

    /// Add context using a closure (lazy evaluation)
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T> ResultExt<T> for Result<T> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| e.context(context))
    }

    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| e.context(f()))
    }
}

/// Convert from anyhow::Error
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Internal(err.to_string())
    }
}

// Note: Database-specific error conversions are commented out
// Uncomment and enable the corresponding features when needed

// /// Convert from sqlx::Error
// #[cfg(feature = "sqlx")]
// impl From<sqlx::Error> for Error {
//     fn from(err: sqlx::Error) -> Self {
//         match err {
//             sqlx::Error::RowNotFound => Error::NotFound("Database row not found".to_string()),
//             sqlx::Error::Database(db_err) => {
//                 // Check for unique constraint violations
//                 if let Some(constraint) = db_err.constraint() {
//                     Error::Conflict(format!("Constraint violation: {}", constraint))
//                 } else {
//                     Error::Database(db_err.to_string())
//                 }
//             }
//             _ => Error::Database(err.to_string()),
//         }
//     }
// }

// /// Convert from diesel::result::Error
// #[cfg(feature = "diesel")]
// impl From<diesel::result::Error> for Error {
//     fn from(err: diesel::result::Error) -> Self {
//         match err {
//             diesel::result::Error::NotFound => {
//                 Error::NotFound("Database record not found".to_string())
//             }
//             _ => Error::Database(err.to_string())
//         }
//     }
// }

/// Convert from git2::Error
impl From<git2::Error> for Error {
    fn from(err: git2::Error) -> Self {
        Error::Git(err.to_string())
    }
}

/// Convert from serde_json::Error
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

/// Convert from serde_yaml::Error
impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

/// Error response for API
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ErrorResponse {
    /// Error code for programmatic handling
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,

    /// Request ID for tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl From<Error> for ErrorResponse {
    fn from(err: Error) -> Self {
        let code = match &err {
            Error::Validation(_) => "VALIDATION_ERROR",
            Error::Authentication(_) => "AUTHENTICATION_ERROR",
            Error::Authorization(_) => "AUTHORIZATION_ERROR",
            Error::NotFound(_) => "NOT_FOUND",
            Error::Conflict(_) => "CONFLICT",
            Error::RateLimit(_) => "RATE_LIMIT_EXCEEDED",
            Error::Timeout(_) => "TIMEOUT",
            Error::Build(_) => "BUILD_ERROR",
            Error::Pipeline(_) => "PIPELINE_ERROR",
            Error::Agent(_) => "AGENT_ERROR",
            Error::Git(_) => "GIT_ERROR",
            Error::Storage(_) => "STORAGE_ERROR",
            Error::Network(_) => "NETWORK_ERROR",
            Error::ExternalService(_) => "EXTERNAL_SERVICE_ERROR",
            _ => "INTERNAL_ERROR",
        };

        ErrorResponse {
            code: code.to_string(),
            message: err.to_string(),
            details: None,
            request_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::validation("Invalid input");
        assert_eq!(err.to_string(), "Validation error: Invalid input");
        assert_eq!(err.status_code(), 400);

        let err = Error::not_found("Resource not found");
        assert_eq!(err.to_string(), "Not found: Resource not found");
        assert_eq!(err.status_code(), 404);
    }

    #[test]
    fn test_error_context() {
        let err = Error::database("Connection failed").context("While fetching user");

        assert!(err.to_string().contains("While fetching user"));
    }

    #[test]
    fn test_result_ext() {
        fn may_fail() -> Result<i32> {
            Err(Error::internal("Something went wrong"))
        }

        let result = may_fail().context("During operation");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.to_string().contains("During operation"));
    }

    #[test]
    fn test_is_retryable() {
        assert!(Error::network("Network error").is_retryable());
        assert!(Error::timeout("Timeout").is_retryable());
        assert!(Error::external_service("Service unavailable").is_retryable());
        assert!(!Error::validation("Invalid input").is_retryable());
        assert!(!Error::authentication("Invalid credentials").is_retryable());
    }

    #[test]
    fn test_error_response() {
        let err = Error::validation("Invalid email format");
        let response: ErrorResponse = err.into();

        assert_eq!(response.code, "VALIDATION_ERROR");
        assert_eq!(response.message, "Validation error: Invalid email format");
    }
}
