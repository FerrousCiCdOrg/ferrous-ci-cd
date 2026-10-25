//! Configuration management for Ferrous CI/CD

use anyhow::Result;
use config::{Config as ConfigBuilder, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main configuration structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,

    /// Database configuration
    pub database: DatabaseConfig,

    /// Storage configuration
    pub storage: StorageConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Git configuration
    pub git: GitConfig,

    /// Agent configuration
    pub agents: AgentConfig,

    /// Notification configuration
    #[serde(default)]
    pub notifications: NotificationConfig,

    /// Monitoring configuration
    #[serde(default)]
    pub monitoring: MonitoringConfig,
}

/// Server configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    /// Host to bind to
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to listen on
    #[serde(default = "default_port")]
    pub port: u16,

    /// Number of worker threads
    #[serde(default = "default_workers")]
    pub workers: usize,

    /// Request timeout in seconds
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u64,

    /// Enable TLS
    #[serde(default)]
    pub tls: Option<TlsConfig>,
}

/// TLS configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TlsConfig {
    /// Certificate file path
    pub cert_path: String,

    /// Private key file path
    pub key_path: String,
}

/// Database configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    /// Database type (postgres, sqlite)
    #[serde(rename = "type")]
    pub db_type: DatabaseType,

    /// Database URL
    pub url: String,

    /// Maximum connections in pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,

    /// Enable database migrations
    #[serde(default = "default_true")]
    pub auto_migrate: bool,
}

/// Database type
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    /// PostgreSQL
    Postgres,
    /// SQLite
    Sqlite,
}

/// Storage configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StorageConfig {
    /// Path to store artifacts
    #[serde(default = "default_artifacts_path")]
    pub artifacts_path: String,

    /// Path to workspace directory
    #[serde(default = "default_workspace_path")]
    pub workspace_path: String,

    /// Path to cache directory
    #[serde(default = "default_cache_path")]
    pub cache_path: String,

    /// Maximum artifact size in MB
    #[serde(default = "default_max_artifact_size")]
    pub max_artifact_size: u64,

    /// Artifact retention days
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,

    /// S3 configuration for artifact storage
    #[serde(default)]
    pub s3: Option<S3Config>,
}

/// S3 configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct S3Config {
    /// S3 bucket name
    pub bucket: String,

    /// S3 region
    pub region: String,

    /// S3 endpoint (for S3-compatible services)
    pub endpoint: Option<String>,

    /// Access key ID
    pub access_key_id: String,

    /// Secret access key
    pub secret_access_key: String,
}

/// Security configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecurityConfig {
    /// JWT secret for authentication
    pub jwt_secret: String,

    /// Session timeout in seconds
    #[serde(default = "default_session_timeout")]
    pub session_timeout: u64,

    /// Password hash cost
    #[serde(default = "default_hash_cost")]
    pub password_hash_cost: u32,

    /// Enable API rate limiting
    #[serde(default = "default_true")]
    pub rate_limiting_enabled: bool,

    /// API rate limit per minute
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_minute: u32,

    /// Allowed CORS origins
    #[serde(default)]
    pub cors_origins: Vec<String>,
}

/// Git configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GitConfig {
    /// SSH key path for Git operations
    pub ssh_key_path: Option<String>,

    /// Known hosts file path
    pub known_hosts_path: Option<String>,

    /// Git clone timeout in seconds
    #[serde(default = "default_clone_timeout")]
    pub clone_timeout: u64,

    /// Git fetch timeout in seconds
    #[serde(default = "default_fetch_timeout")]
    pub fetch_timeout: u64,

    /// Maximum repository size in MB
    #[serde(default = "default_max_repo_size")]
    pub max_repo_size: u64,
}

/// Agent configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AgentConfig {
    /// Maximum concurrent builds per agent
    #[serde(default = "default_max_concurrent_builds")]
    pub max_concurrent_builds: usize,

    /// Agent heartbeat interval in seconds
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval: u64,

    /// Agent timeout in seconds (considered dead after this)
    #[serde(default = "default_agent_timeout")]
    pub agent_timeout: u64,

    /// Enable auto-scaling
    #[serde(default)]
    pub auto_scaling: Option<AutoScalingConfig>,
}

/// Auto-scaling configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AutoScalingConfig {
    /// Minimum number of agents
    pub min_agents: usize,

    /// Maximum number of agents
    pub max_agents: usize,

    /// Scale up threshold (percentage)
    pub scale_up_threshold: u8,

    /// Scale down threshold (percentage)
    pub scale_down_threshold: u8,
}

/// Notification configuration
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct NotificationConfig {
    /// Email notification settings
    #[serde(default)]
    pub email: Option<EmailConfig>,

    /// Slack notification settings
    #[serde(default)]
    pub slack: Option<SlackConfig>,

    /// Webhook notification settings
    #[serde(default)]
    pub webhooks: Vec<WebhookConfig>,
}

/// Email configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmailConfig {
    /// SMTP server host
    pub smtp_host: String,

    /// SMTP server port
    pub smtp_port: u16,

    /// SMTP username
    pub smtp_username: String,

    /// SMTP password
    pub smtp_password: String,

    /// From email address
    pub from_address: String,

    /// Use TLS
    #[serde(default = "default_true")]
    pub use_tls: bool,
}

/// Slack configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SlackConfig {
    /// Slack webhook URL
    pub webhook_url: String,

    /// Default channel
    pub default_channel: Option<String>,

    /// Bot username
    pub bot_username: Option<String>,

    /// Bot icon emoji
    pub bot_icon: Option<String>,
}

/// Webhook configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WebhookConfig {
    /// Webhook URL
    pub url: String,

    /// Secret for webhook signature
    pub secret: Option<String>,

    /// Events to trigger webhook
    pub events: Vec<String>,
}

/// Monitoring configuration
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MonitoringConfig {
    /// Enable Prometheus metrics
    #[serde(default)]
    pub prometheus_enabled: bool,

    /// Prometheus metrics port
    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,

    /// Enable Jaeger tracing
    #[serde(default)]
    pub jaeger_enabled: bool,

    /// Jaeger endpoint
    pub jaeger_endpoint: Option<String>,

    /// Sampling rate for tracing (0.0 - 1.0)
    #[serde(default = "default_sampling_rate")]
    pub sampling_rate: f64,
}

// Default value functions
fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_workers() -> usize {
    num_cpus::get()
}

fn default_request_timeout() -> u64 {
    60
}

fn default_max_connections() -> u32 {
    10
}

fn default_connection_timeout() -> u64 {
    30
}

fn default_artifacts_path() -> String {
    "./artifacts".to_string()
}

fn default_workspace_path() -> String {
    "./workspace".to_string()
}

fn default_cache_path() -> String {
    "./cache".to_string()
}

fn default_max_artifact_size() -> u64 {
    500 // MB
}

fn default_retention_days() -> u32 {
    30
}

fn default_session_timeout() -> u64 {
    3600
}

fn default_hash_cost() -> u32 {
    12
}

fn default_rate_limit() -> u32 {
    100
}

fn default_clone_timeout() -> u64 {
    300
}

fn default_fetch_timeout() -> u64 {
    60
}

fn default_max_repo_size() -> u64 {
    1000 // MB
}

fn default_max_concurrent_builds() -> usize {
    5
}

fn default_heartbeat_interval() -> u64 {
    30
}

fn default_agent_timeout() -> u64 {
    120
}

fn default_metrics_port() -> u16 {
    9090
}

fn default_sampling_rate() -> f64 {
    0.1
}

fn default_true() -> bool {
    true
}

impl Config {
    /// Load configuration from file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let settings = ConfigBuilder::builder()
            // Add the config file
            .add_source(File::from(path.as_ref()).required(false))
            // Add environment variables with prefix FERROUS_
            .add_source(Environment::with_prefix("FERROUS").separator("__"))
            .build()?;

        settings.try_deserialize().map_err(|e| anyhow::anyhow!(e))
    }

    /// Create default configuration
    pub fn default() -> Self {
        Self {
            server: ServerConfig {
                host: default_host(),
                port: default_port(),
                workers: default_workers(),
                request_timeout: default_request_timeout(),
                tls: None,
            },
            database: DatabaseConfig {
                db_type: DatabaseType::Sqlite,
                url: "sqlite://ferrous.db".to_string(),
                max_connections: default_max_connections(),
                connection_timeout: default_connection_timeout(),
                auto_migrate: true,
            },
            storage: StorageConfig {
                artifacts_path: default_artifacts_path(),
                workspace_path: default_workspace_path(),
                cache_path: default_cache_path(),
                max_artifact_size: default_max_artifact_size(),
                retention_days: default_retention_days(),
                s3: None,
            },
            security: SecurityConfig {
                jwt_secret: "change-me-in-production".to_string(),
                session_timeout: default_session_timeout(),
                password_hash_cost: default_hash_cost(),
                rate_limiting_enabled: true,
                rate_limit_per_minute: default_rate_limit(),
                cors_origins: vec![],
            },
            git: GitConfig {
                ssh_key_path: None,
                known_hosts_path: None,
                clone_timeout: default_clone_timeout(),
                fetch_timeout: default_fetch_timeout(),
                max_repo_size: default_max_repo_size(),
            },
            agents: AgentConfig {
                max_concurrent_builds: default_max_concurrent_builds(),
                heartbeat_interval: default_heartbeat_interval(),
                agent_timeout: default_agent_timeout(),
                auto_scaling: None,
            },
            notifications: NotificationConfig::default(),
            monitoring: MonitoringConfig::default(),
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate server config
        if self.server.port == 0 {
            return Err(anyhow::anyhow!("Invalid port number"));
        }

        // Validate database URL
        if self.database.url.is_empty() {
            return Err(anyhow::anyhow!("Database URL cannot be empty"));
        }

        // Validate security config
        if self.security.jwt_secret == "change-me-in-production" {
            eprintln!("WARNING: Using default JWT secret, please change in production!");
        }

        // Validate storage paths
        std::fs::create_dir_all(&self.storage.artifacts_path)?;
        std::fs::create_dir_all(&self.storage.workspace_path)?;
        std::fs::create_dir_all(&self.storage.cache_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.database.max_connections, 10);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Invalid port should fail
        config.server.port = 0;
        assert!(config.validate().is_err());
        config.server.port = 8080;

        // Empty database URL should fail
        config.database.url = "".to_string();
        assert!(config.validate().is_err());
    }
}
