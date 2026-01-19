# Domain Model Specification

## Overview

This document provides a detailed description of the domain model for the Ferrous CI/CD system. The design is centered on business logic, following Domain-Driven Design (DDD) principles.

## Aggregates

### Pipeline Aggregate

**Aggregate Root**: `Pipeline`

The pipeline represents a CI/CD pipeline definition as an aggregate root.

#### Entity Structure

```rust
Pipeline {
    id: PipelineId,
    project_id: ProjectId,
    name: String,
    description: Option<String>,
    config: PipelineConfig,
    enabled: bool,
    version: u32,
    tags: Vec<String>,
    environment: HashMap<String, String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    events: Vec<DomainEvent>
}
```

#### Main Methods

- `new(project_id, name, config)`: Creates a new pipeline
- `update_config(config)`: Updates pipeline configuration (version is incremented)
- `enable()`: Enables the pipeline
- `disable()`: Disables the pipeline
- `add_tag(tag)`: Adds a tag
- `remove_tag(tag)`: Removes a tag
- `set_environment_variable(key, value)`: Sets an environment variable
- `validate()`: Validates the pipeline

#### Business Rules

1. Pipeline name must not be empty
2. Pipeline name must be 255 characters or less
3. Pipeline configuration must be valid
4. Version is incremented when configuration is updated

#### Domain Events

- `PipelineCreated`: When pipeline is created
- `PipelineConfigUpdated`: When configuration is updated
- `PipelineEnabled`: When pipeline is enabled
- `PipelineDisabled`: When pipeline is disabled

### Build Aggregate

**Aggregate Root**: `Build`

A build represents an execution instance of a pipeline.

#### Entity Structure

```rust
Build {
    id: BuildId,
    pipeline_id: PipelineId,
    project_id: ProjectId,
    number: u64,
    status: BuildStatus,
    commit_sha: String,
    branch: String,
    commit_message: Option<String>,
    commit_author: Option<String>,
    agent_id: Option<AgentId>,
    parameters: HashMap<String, String>,
    environment: HashMap<String, String>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    trigger: BuildTrigger,
    logs_url: Option<String>,
    artifacts: Vec<String>,
    error_message: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    events: Vec<DomainEvent>
}
```

#### Build Trigger

```rust
enum BuildTrigger {
    Manual { user_id: String },
    Push,
    PullRequest { pr_number: u32 },
    Schedule { cron: String },
    Api { token: String },
    Webhook { source: String },
}
```

#### State Machine

```
Pending → Running → Success
              ↓
           Failed
              ↓
         Cancelled (at any point)
```

#### Business Rules

1. Builds start in Pending state
2. Only Running builds can succeed or fail
3. Completed builds (Success/Failed) cannot be cancelled
4. Build numbers are assigned sequentially per pipeline

#### Domain Events

- `BuildCreated`: When build is created
- `BuildStarted`: When build starts
- `BuildCompleted`: When build completes (success or failure)
- `BuildCancelled`: When build is cancelled

### Project Aggregate

**Aggregate Root**: `Project`

A project is a unit that manages repositories and pipelines.

#### Entity Structure

```rust
Project {
    id: ProjectId,
    name: String,
    description: Option<String>,
    repository_url: String,
    default_branch: String,
    visibility: ProjectVisibility,
    settings: ProjectSettings,
    metadata: HashMap<String, String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    events: Vec<DomainEvent>
}
```

#### Project Visibility

```rust
enum ProjectVisibility {
    Public,    // Public project
    Internal,  // Visible to authenticated users
    Private,   // Private project
}
```

#### Project Settings

```rust
struct ProjectSettings {
    auto_cancel: bool,                    // Auto-cancel redundant builds
    build_timeout: u64,                   // Build timeout (seconds)
    max_concurrent_builds: usize,         // Maximum concurrent builds
    pr_builds_enabled: bool,             // Enable PR builds
    protected_branches: Vec<String>,      // Protected branches
}
```

#### Business Rules

1. Project name must not be empty
2. Repository URL must not be empty
3. Default branch is required

#### Domain Events

- `ProjectCreated`: When project is created

### Agent Aggregate

**Aggregate Root**: `Agent`

An agent is a worker node that executes builds.

#### Entity Structure

```rust
Agent {
    id: AgentId,
    name: String,
    description: Option<String>,
    status: AgentStatus,
    labels: HashMap<String, String>,
    max_concurrent_jobs: usize,
    current_jobs: usize,
    platform: AgentPlatform,
    last_heartbeat: DateTime<Utc>,
    ip_address: Option<String>,
    version: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    events: Vec<DomainEvent>
}
```

#### Agent Status

```rust
enum AgentStatus {
    Online,        // Online and available
    Busy,          // Busy state
    Offline,       // Offline
    Maintenance,   // Maintenance mode
    Disconnected,  // Disconnected
}
```

#### Business Rules

1. Agents can accept jobs up to max_concurrent_jobs
2. Jobs can only be accepted when Online and current_jobs < max_concurrent_jobs
3. Agents are considered unresponsive if heartbeat is missing for a certain period

#### Domain Events

- `AgentRegistered`: When agent is registered
- `AgentDisconnected`: When agent is disconnected

### User Aggregate

**Aggregate Root**: `User`

A user represents a system user.

#### User Roles

```rust
enum UserRole {
    Admin,      // Administrator (full access)
    Developer,  // Developer (build and deploy permissions)
    Viewer,     // Viewer (read-only access)
    Service,    // Service account (for API access)
}
```

#### Domain Events

- `UserCreated`: When user is created
- `UserPasswordChanged`: When password is changed
- `UserDeactivated`: When user is deactivated

## Value Objects

### ID Value Objects

All IDs are implemented as value objects, providing type safety.

- `PipelineId`: Pipeline ID (UUID)
- `BuildId`: Build ID (UUID)
- `ProjectId`: Project ID (UUID)
- `AgentId`: Agent ID (UUID)
- `UserId`: User ID (UUID)

### BuildStatus

```rust
enum BuildStatus {
    Pending,    // Waiting to start
    Running,    // Running
    Success,    // Success
    Failed,     // Failed
    Cancelled,  // Cancelled
}
```

### PipelineConfig

A value object representing pipeline execution configuration.

#### Structure

```rust
struct PipelineConfig {
    version: String,
    stages: Vec<Stage>,
    triggers: Vec<Trigger>,
    environment: HashMap<String, String>,
    notifications: Option<NotificationConfig>,
}
```

#### Stage

```rust
struct Stage {
    name: String,
    jobs: Vec<Job>,
    parallel: bool,              // Whether to run jobs in parallel
    when: Option<WhenCondition>, // Execution conditions
}
```

#### Job

```rust
struct Job {
    name: String,
    image: Option<String>,                    // Docker image
    commands: Vec<String>,                     // Commands to execute
    environment: HashMap<String, String>,      // Environment variables
    working_directory: Option<String>,        // Working directory
    timeout: Option<u64>,                     // Timeout (seconds)
    retry: Option<u32>,                       // Retry count
    artifacts: Option<ArtifactConfig>,       // Artifact configuration
    cache: Option<CacheConfig>,               // Cache configuration
    needs: Vec<String>,                       // Dependent jobs
    when: Option<WhenCondition>,              // Execution conditions
}
```

#### Trigger

```rust
enum Trigger {
    Push { branches: Vec<String> },
    PullRequest { branches: Vec<String> },
    Schedule { cron: String },
    Manual,
    Tag { patterns: Vec<String> },
}
```

## Repository Interfaces

### PipelineRepository

```rust
#[async_trait]
trait PipelineRepository: Send + Sync {
    async fn save(&self, pipeline: &mut Pipeline) -> Result<()>;
    async fn find_by_id(&self, id: &PipelineId) -> Result<Option<Pipeline>>;
    async fn find_by_project_id(&self, project_id: &ProjectId) -> Result<Vec<Pipeline>>;
    async fn delete(&self, id: &PipelineId) -> Result<()>;
}
```

### BuildRepository

```rust
#[async_trait]
trait BuildRepository: Send + Sync {
    async fn save(&self, build: &mut Build) -> Result<()>;
    async fn find_by_id(&self, id: &BuildId) -> Result<Option<Build>>;
    async fn find_by_pipeline_id(&self, pipeline_id: &PipelineId) -> Result<Vec<Build>>;
    async fn find_running_builds(&self) -> Result<Vec<Build>>;
    async fn get_next_build_number(&self, pipeline_id: &PipelineId) -> Result<u64>;
}
```

### ProjectRepository

```rust
#[async_trait]
trait ProjectRepository: Send + Sync {
    async fn save(&self, project: &mut Project) -> Result<()>;
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Project>>;
    async fn find_all(&self) -> Result<Vec<Project>>;
    async fn delete(&self, id: &ProjectId) -> Result<()>;
}
```

### AgentRepository

```rust
#[async_trait]
trait AgentRepository: Send + Sync {
    async fn save(&self, agent: &mut Agent) -> Result<()>;
    async fn find_by_id(&self, id: &AgentId) -> Result<Option<Agent>>;
    async fn find_available_agents(&self) -> Result<Vec<Agent>>;
    async fn find_by_label(&self, key: &str, value: &str) -> Result<Vec<Agent>>;
    async fn delete(&self, id: &AgentId) -> Result<()>;
}
```

## Domain Services

### PipelineService

Provides business logic related to pipelines.

- Pipeline configuration validation
- Pipeline executability checks
- Pipeline configuration merging

### BuildService

Provides business logic related to builds.

- Build scheduling
- Agent assignment
- Build execution management

### AgentService

Provides business logic related to agents.

- Agent availability checks
- Label matching
- Agent health checks
