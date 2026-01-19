# Events Specification

## Overview

Ferrous CI/CD adopts an event-driven design, notifying system state changes through domain events. This enables a loosely coupled and extensible architecture.

## Event Types

### Build Events

#### BuildCreated

Published when a build is created.

```rust
BuildCreated {
    build_id: BuildId,
    pipeline_id: PipelineId,
    project_id: ProjectId,
    number: u64,
    created_at: DateTime<Utc>,
}
```

**Published When**: A new build is created

**Example Subscribers**:
- Notification services (Slack, email)
- Metrics collection
- Logging

#### BuildStarted

Published when a build starts.

```rust
BuildStarted {
    build_id: BuildId,
    agent_id: AgentId,
    started_at: DateTime<Utc>,
}
```

**Published When**: Build is assigned to an agent and execution starts

**Example Subscribers**:
- Notification services
- Dashboard updates
- Resource monitoring

#### BuildCompleted

Published when a build completes (both success and failure).

```rust
BuildCompleted {
    build_id: BuildId,
    status: BuildStatus,  // Success or Failed
    completed_at: DateTime<Utc>,
}
```

**Published When**: Build completes successfully or fails

**Example Subscribers**:
- Notification services (especially on failure)
- Deployment triggers (on success)
- Metrics collection
- Report generation

#### BuildCancelled

Published when a build is cancelled.

```rust
BuildCancelled {
    build_id: BuildId,
    cancelled_at: DateTime<Utc>,
}
```

**Published When**: Build is cancelled by user or system

**Example Subscribers**:
- Notification services
- Resource release
- Metrics collection

### Pipeline Events

#### PipelineCreated

Published when a pipeline is created.

```rust
PipelineCreated {
    pipeline_id: PipelineId,
    project_id: ProjectId,
    name: String,
    created_at: DateTime<Utc>,
}
```

**Published When**: A new pipeline is created

#### PipelineConfigUpdated

Published when pipeline configuration is updated.

```rust
PipelineConfigUpdated {
    pipeline_id: PipelineId,
    old_version: u32,
    new_version: u32,
    updated_at: DateTime<Utc>,
}
```

**Published When**: Pipeline configuration is changed and version is updated

#### PipelineEnabled

Published when a pipeline is enabled.

```rust
PipelineEnabled {
    pipeline_id: PipelineId,
    enabled_at: DateTime<Utc>,
}
```

**Published When**: A disabled pipeline is enabled

#### PipelineDisabled

Published when a pipeline is disabled.

```rust
PipelineDisabled {
    pipeline_id: PipelineId,
    disabled_at: DateTime<Utc>,
}
```

**Published When**: Pipeline is disabled

### Project Events

#### ProjectCreated

Published when a project is created.

```rust
ProjectCreated {
    project_id: ProjectId,
    name: String,
    repository_url: String,
    created_at: DateTime<Utc>,
}
```

**Published When**: A new project is created

**Example Subscribers**:
- Repository cloning
- Initial configuration application
- Notification services

### Agent Events

#### AgentRegistered

Published when an agent is registered.

```rust
AgentRegistered {
    agent_id: AgentId,
    name: String,
    created_at: DateTime<Utc>,
}
```

**Published When**: A new agent is registered with the system

**Example Subscribers**:
- Resource monitoring
- Metrics collection
- Notification services

#### AgentDisconnected

Published when an agent is disconnected.

```rust
AgentDisconnected {
    agent_id: AgentId,
    disconnected_at: DateTime<Utc>,
}
```

**Published When**: Agent is disconnected

**Example Subscribers**:
- Rescheduling of running builds
- Resource monitoring
- Notification services

### User Events

#### UserCreated

Published when a user is created.

```rust
UserCreated {
    user_id: UserId,
    username: String,
    email: String,
    role: UserRole,
    created_at: DateTime<Utc>,
}
```

**Published When**: A new user is created

#### UserPasswordChanged

Published when a user's password is changed.

```rust
UserPasswordChanged {
    user_id: UserId,
    changed_at: DateTime<Utc>,
}
```

**Published When**: User's password is changed

**Example Subscribers**:
- Security audit logs
- Notification services (security alerts)

#### UserDeactivated

Published when a user is deactivated.

```rust
UserDeactivated {
    user_id: UserId,
    deactivated_at: DateTime<Utc>,
}
```

**Published When**: User account is deactivated

## Event Processing

### Event Publishing

Events are published within entity methods. Entities retrieve events using the `take_events()` method and persist them via the repository's `save()` method.

```rust
// エンティティ内でイベントを追加
self.events.push(DomainEvent::BuildCreated { ... });

// リポジトリでイベントを取得して発行
let events = build.take_events();
for event in events {
    event_publisher.publish(event).await?;
}
```

### Event Handlers

Event handlers implement the `EventHandler` trait.

```rust
#[async_trait]
trait EventHandler: Send + Sync {
    async fn handle(&self, event: &DomainEvent) -> Result<()>;
    fn interested_in(&self) -> Vec<&str>;
}
```

**実装例**:

```rust
struct NotificationHandler {
    slack_client: SlackClient,
    email_client: EmailClient,
}

#[async_trait]
impl EventHandler for NotificationHandler {
    async fn handle(&self, event: &DomainEvent) -> Result<()> {
        match event {
            DomainEvent::BuildCompleted { build_id, status, .. } => {
                if status.is_failed() {
                    self.slack_client.send_alert(build_id).await?;
                    self.email_client.send_failure_notification(build_id).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    fn interested_in(&self) -> Vec<&str> {
        vec!["build.completed", "build.failed"]
    }
}
```

### Event Publishers

Event publishers implement the `EventPublisher` trait.

```rust
#[async_trait]
trait EventPublisher: Send + Sync {
    async fn publish(&self, event: DomainEvent) -> Result<()>;
    async fn publish_batch(&self, events: Vec<DomainEvent>) -> Result<()>;
}
```

**Implementation Examples**:

- **InMemoryEventPublisher**: In-memory implementation for testing
- **DatabaseEventPublisher**: Saves events to database
- **MessageQueueEventPublisher**: Publishes to message queues (RabbitMQ, Redis) (planned)

## Event Persistence

Events are persisted for the following purposes:

1. **Audit Logs**: Recording system state changes
2. **Event Sourcing**: Preparation for future event sourcing implementation
3. **Reprocessing**: Disaster recovery through event reprocessing

## Event Ordering

Events generally guarantee ordering, but complete ordering is not guaranteed in distributed environments. If ordering is important, consider including sequence numbers in events.

## Event Idempotency

Event handlers should be idempotent. They should be implemented so that processing the same event multiple times produces the same result.

## Event Versioning

If event structures change, versioning is necessary. Consider including version fields in events for future extensions.

## Event Filtering

Event handlers specify events of interest using the `interested_in()` method. This avoids processing unnecessary events.

## Event Testing

Tests can use `InMemoryEventPublisher` to verify event publishing.

```rust
let publisher = InMemoryEventPublisher::new();
// ... イベントを発行 ...
let events = publisher.get_events().await;
assert_eq!(events.len(), 1);
assert_eq!(events[0].event_type(), "build.created");
```

## Future Extensions

### Planned Features

- **Message Queue Integration**: RabbitMQ, Redis Streams
- **Event Streaming**: Kafka integration
- **Event Replay**: Reprocessing of past events
- **Event Versioning**: Support for schema evolution
- **Distributed Tracing**: Tracing between events
