# Configuration Specification

## Overview

Ferrous CI/CD supports configuration via YAML configuration files and environment variables. Configuration is loaded from the `config.yaml` file and can be overridden by environment variables.

## Configuration File Location

By default, configuration files are searched in the following order:

1. `./config.yaml`
2. `~/.config/ferrous-ci-cd/config.yaml`
3. `/etc/ferrous-ci-cd/config.yaml`

You can specify the configuration file path using the `FERROUS_CONFIG_PATH` environment variable.

## Configuration Structure

### Complete Configuration Example

```yaml
server:
  host: "0.0.0.0"
  port: 8080
  workers: 4
  request_timeout: 60
  tls:
    cert_path: "/path/to/cert.pem"
    key_path: "/path/to/key.pem"

database:
  type: "postgres"  # or "sqlite"
  url: "postgresql://user:password@localhost/ferrous_ci"
  max_connections: 10
  connection_timeout: 30
  auto_migrate: true

storage:
  artifacts_path: "./artifacts"
  workspace_path: "./workspace"
  cache_path: "./cache"
  max_artifact_size: 500  # MB
  retention_days: 30
  s3:
    bucket: "ferrous-artifacts"
    region: "us-east-1"
    endpoint: null
    access_key_id: "AKIAIOSFODNN7EXAMPLE"
    secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"

security:
  jwt_secret: "your-secret-key-change-in-production"
  session_timeout: 3600  # seconds
  password_hash_cost: 12
  rate_limiting_enabled: true
  rate_limit_per_minute: 100
  cors_origins:
    - "https://example.com"
    - "https://app.example.com"

git:
  ssh_key_path: "~/.ssh/id_rsa"
  known_hosts_path: "~/.ssh/known_hosts"
  clone_timeout: 300  # seconds
  fetch_timeout: 60  # seconds
  max_repo_size: 1000  # MB

agents:
  max_concurrent_builds: 5
  heartbeat_interval: 30  # seconds
  agent_timeout: 120  # seconds
  auto_scaling:
    min_agents: 1
    max_agents: 10
    scale_up_threshold: 80  # percentage
    scale_down_threshold: 20  # percentage

notifications:
  email:
    smtp_host: "smtp.example.com"
    smtp_port: 587
    smtp_username: "user@example.com"
    smtp_password: "password"
    from_address: "noreply@example.com"
    use_tls: true
  slack:
    webhook_url: "https://hooks.slack.com/services/..."
    default_channel: "#ci-cd"
    bot_username: "Ferrous CI/CD"
    bot_icon: ":robot_face:"
  webhooks:
    - url: "https://example.com/webhook"
      secret: "webhook-secret"
      events:
        - "build.completed"
        - "build.failed"

monitoring:
  prometheus_enabled: true
  metrics_port: 9090
  jaeger_enabled: false
  jaeger_endpoint: "http://localhost:14268/api/traces"
  sampling_rate: 0.1  # 0.0 - 1.0
```

## Configuration Items

### server

Server configuration.

| Item | Type | Default | Description |
|------|-----|-----------|------|
| `host` | string | `"0.0.0.0"` | Host address to bind |
| `port` | u16 | `8080` | Listening port |
| `workers` | usize | CPU cores | Number of worker threads |
| `request_timeout` | u64 | `60` | Request timeout (seconds) |
| `tls` | object | `null` | TLS configuration (optional) |

#### tls

TLS configuration.

| Item | Type | Description |
|------|-----|------|
| `cert_path` | string | Certificate file path |
| `key_path` | string | Private key file path |

### database

Database configuration.

| Item | Type | Default | Description |
|------|-----|-----------|------|
| `type` | string | `"sqlite"` | Database type (`postgres` or `sqlite`) |
| `url` | string | `"sqlite://ferrous.db"` | Database connection URL |
| `max_connections` | u32 | `10` | Maximum connections |
| `connection_timeout` | u64 | `30` | Connection timeout (seconds) |
| `auto_migrate` | bool | `true` | Auto migration |

**PostgreSQL Connection URL Example**:
```
postgresql://user:password@localhost:5432/ferrous_ci
```

**SQLite Connection URL Example**:
```
sqlite:///path/to/ferrous.db
```

### storage

Storage configuration.

| Item | Type | Default | Description |
|------|-----|-----------|------|
| `artifacts_path` | string | `"./artifacts"` | Artifact storage path |
| `workspace_path` | string | `"./workspace"` | Workspace path |
| `cache_path` | string | `"./cache"` | Cache path |
| `max_artifact_size` | u64 | `500` | Maximum artifact size (MB) |
| `retention_days` | u32 | `30` | Artifact retention period (days) |
| `s3` | object | `null` | S3 configuration (optional) |

#### s3

S3 configuration (optional).

| Item | Type | Description |
|------|-----|------|
| `bucket` | string | S3 bucket name |
| `region` | string | AWS region |
| `endpoint` | string | Custom endpoint (for S3-compatible services) |
| `access_key_id` | string | Access key ID |
| `secret_access_key` | string | Secret access key |

### security

Security configuration.

| Item | Type | Default | Description |
|------|-----|-----------|------|
| `jwt_secret` | string | `"change-me-in-production"` | JWT signing secret |
| `session_timeout` | u64 | `3600` | Session timeout (seconds) |
| `password_hash_cost` | u32 | `12` | Password hash cost (bcrypt) |
| `rate_limiting_enabled` | bool | `true` | Enable rate limiting |
| `rate_limit_per_minute` | u32 | `100` | Requests per minute limit |
| `cors_origins` | array | `[]` | CORS allowed origins |

### git

Git configuration.

| Item | Type | Default | Description |
|------|-----|-----------|------|
| `ssh_key_path` | string | `null` | SSH private key path |
| `known_hosts_path` | string | `null` | known_hosts file path |
| `clone_timeout` | u64 | `300` | Clone timeout (seconds) |
| `fetch_timeout` | u64 | `60` | Fetch timeout (seconds) |
| `max_repo_size` | u64 | `1000` | Maximum repository size (MB) |

### agents

Agent configuration.

| Item | Type | Default | Description |
|------|-----|-----------|------|
| `max_concurrent_builds` | usize | `5` | Maximum concurrent builds per agent |
| `heartbeat_interval` | u64 | `30` | Heartbeat interval (seconds) |
| `agent_timeout` | u64 | `120` | Agent timeout (seconds) |
| `auto_scaling` | object | `null` | Auto-scaling configuration (optional) |

#### auto_scaling

Auto-scaling configuration (optional).

| Item | Type | Description |
|------|-----|------|
| `min_agents` | usize | Minimum number of agents |
| `max_agents` | usize | Maximum number of agents |
| `scale_up_threshold` | u8 | Scale-up threshold (%) |
| `scale_down_threshold` | u8 | Scale-down threshold (%) |

### notifications

Notification configuration.

#### email

Email notification configuration.

| Item | Type | Description |
|------|-----|------|
| `smtp_host` | string | SMTP server host |
| `smtp_port` | u16 | SMTP port |
| `smtp_username` | string | SMTP username |
| `smtp_password` | string | SMTP password |
| `from_address` | string | From email address |
| `use_tls` | bool | Use TLS |

#### slack

Slack notification configuration.

| Item | Type | Description |
|------|-----|------|
| `webhook_url` | string | Slack Webhook URL |
| `default_channel` | string | Default channel |
| `bot_username` | string | Bot username |
| `bot_icon` | string | Bot icon (emoji) |

#### webhooks

Webhook notification configuration.

| Item | Type | Description |
|------|-----|------|
| `url` | string | Webhook URL |
| `secret` | string | Webhook signing secret |
| `events` | array | List of events to notify |

### monitoring

Monitoring configuration.

| Item | Type | Default | Description |
|------|-----|-----------|------|
| `prometheus_enabled` | bool | `false` | Enable Prometheus metrics |
| `metrics_port` | u16 | `9090` | Metrics endpoint port |
| `jaeger_enabled` | bool | `false` | Enable Jaeger tracing |
| `jaeger_endpoint` | string | `null` | Jaeger endpoint |
| `sampling_rate` | f64 | `0.1` | Tracing sampling rate (0.0-1.0) |

## Configuration via Environment Variables

You can override configuration using environment variables. Environment variable names use the `FERROUS__` prefix, and nested settings are separated by `__` (double underscore).

**Example**:
```bash
# Set server port
export FERROUS__SERVER__PORT=9000

# Set database URL
export FERROUS__DATABASE__URL="postgresql://user:pass@localhost/db"

# Set JWT secret
export FERROUS__SECURITY__JWT_SECRET="my-secret-key"
```

## Configuration Validation

Configuration validation is performed at startup. The following checks are executed:

1. Whether server port is within valid range
2. Whether database URL is not empty
3. Whether JWT secret is not default value (warning)
4. Whether storage paths exist (created if they don't exist)

## Configuration Priority

1. Environment variables (highest priority)
2. Configuration file
3. Default values (lowest priority)

## Security Best Practices

1. **JWT Secret**: Always use a strong secret in production
2. **Database Credentials**: Recommended to set via environment variables
3. **SSH Keys**: Protect with appropriate permissions (600)
4. **TLS**: Always enable TLS in production
5. **CORS**: Only allow necessary origins

## Configuration Examples

### Development Environment

```yaml
server:
  port: 8080

database:
  type: "sqlite"
  url: "sqlite://./dev.db"

storage:
  artifacts_path: "./artifacts"
  workspace_path: "./workspace"
```

### Production Environment

```yaml
server:
  host: "0.0.0.0"
  port: 443
  tls:
    cert_path: "/etc/ssl/certs/ferrous.pem"
    key_path: "/etc/ssl/private/ferrous.key"

database:
  type: "postgres"
  url: "${DATABASE_URL}"  # 環境変数から読み込み
  max_connections: 50

storage:
  artifacts_path: "/var/lib/ferrous/artifacts"
  workspace_path: "/var/lib/ferrous/workspace"
  s3:
    bucket: "ferrous-artifacts"
    region: "us-east-1"
    access_key_id: "${AWS_ACCESS_KEY_ID}"
    secret_access_key: "${AWS_SECRET_ACCESS_KEY}"

security:
  jwt_secret: "${JWT_SECRET}"  # 環境変数から読み込み
  rate_limiting_enabled: true
  rate_limit_per_minute: 200
```
