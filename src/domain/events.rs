//! Domain events for the CI/CD system

use crate::domain::entities::user::UserRole;
use crate::domain::value_objects::{
    agent_id::AgentId, build_id::BuildId, build_status::BuildStatus, pipeline_id::PipelineId,
    project_id::ProjectId, user_id::UserId,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Domain event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum DomainEvent {
    // Build events
    BuildCreated {
        build_id: BuildId,
        pipeline_id: PipelineId,
        project_id: ProjectId,
        number: u64,
        created_at: DateTime<Utc>,
    },
    BuildStarted {
        build_id: BuildId,
        agent_id: AgentId,
        started_at: DateTime<Utc>,
    },
    BuildCompleted {
        build_id: BuildId,
        status: BuildStatus,
        completed_at: DateTime<Utc>,
    },
    BuildCancelled {
        build_id: BuildId,
        cancelled_at: DateTime<Utc>,
    },

    // Pipeline events
    PipelineCreated {
        pipeline_id: PipelineId,
        project_id: ProjectId,
        name: String,
        created_at: DateTime<Utc>,
    },
    PipelineConfigUpdated {
        pipeline_id: PipelineId,
        old_version: u32,
        new_version: u32,
        updated_at: DateTime<Utc>,
    },
    PipelineEnabled {
        pipeline_id: PipelineId,
        enabled_at: DateTime<Utc>,
    },
    PipelineDisabled {
        pipeline_id: PipelineId,
        disabled_at: DateTime<Utc>,
    },

    // Project events
    ProjectCreated {
        project_id: ProjectId,
        name: String,
        repository_url: String,
        created_at: DateTime<Utc>,
    },

    // Agent events
    AgentRegistered {
        agent_id: AgentId,
        name: String,
        created_at: DateTime<Utc>,
    },
    AgentDisconnected {
        agent_id: AgentId,
        disconnected_at: DateTime<Utc>,
    },

    // User events
    UserCreated {
        user_id: UserId,
        username: String,
        email: String,
        role: UserRole,
        created_at: DateTime<Utc>,
    },
    UserPasswordChanged {
        user_id: UserId,
        changed_at: DateTime<Utc>,
    },
    UserDeactivated {
        user_id: UserId,
        deactivated_at: DateTime<Utc>,
    },
}

impl DomainEvent {
    /// Get the event type as a string
    pub fn event_type(&self) -> &str {
        match self {
            DomainEvent::BuildCreated { .. } => "build.created",
            DomainEvent::BuildStarted { .. } => "build.started",
            DomainEvent::BuildCompleted { .. } => "build.completed",
            DomainEvent::BuildCancelled { .. } => "build.cancelled",
            DomainEvent::PipelineCreated { .. } => "pipeline.created",
            DomainEvent::PipelineConfigUpdated { .. } => "pipeline.config_updated",
            DomainEvent::PipelineEnabled { .. } => "pipeline.enabled",
            DomainEvent::PipelineDisabled { .. } => "pipeline.disabled",
            DomainEvent::ProjectCreated { .. } => "project.created",
            DomainEvent::AgentRegistered { .. } => "agent.registered",
            DomainEvent::AgentDisconnected { .. } => "agent.disconnected",
            DomainEvent::UserCreated { .. } => "user.created",
            DomainEvent::UserPasswordChanged { .. } => "user.password_changed",
            DomainEvent::UserDeactivated { .. } => "user.deactivated",
        }
    }

    /// Get the timestamp of the event
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            DomainEvent::BuildCreated { created_at, .. }
            | DomainEvent::PipelineCreated { created_at, .. }
            | DomainEvent::ProjectCreated { created_at, .. }
            | DomainEvent::AgentRegistered { created_at, .. }
            | DomainEvent::UserCreated { created_at, .. } => *created_at,
            DomainEvent::BuildStarted { started_at, .. } => *started_at,
            DomainEvent::BuildCompleted { completed_at, .. } => *completed_at,
            DomainEvent::BuildCancelled { cancelled_at, .. } => *cancelled_at,
            DomainEvent::PipelineConfigUpdated { updated_at, .. } => *updated_at,
            DomainEvent::PipelineEnabled { enabled_at, .. } => *enabled_at,
            DomainEvent::PipelineDisabled { disabled_at, .. } => *disabled_at,
            DomainEvent::AgentDisconnected {
                disconnected_at, ..
            } => *disconnected_at,
            DomainEvent::UserPasswordChanged { changed_at, .. } => *changed_at,
            DomainEvent::UserDeactivated { deactivated_at, .. } => *deactivated_at,
        }
    }
}

/// Event publisher trait for publishing domain events
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a domain event
    async fn publish(&self, event: DomainEvent) -> crate::Result<()>;

    /// Publish multiple domain events
    async fn publish_batch(&self, events: Vec<DomainEvent>) -> crate::Result<()>;
}

/// Event handler trait for handling domain events
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle a domain event
    async fn handle(&self, event: &DomainEvent) -> crate::Result<()>;

    /// Get the events this handler is interested in
    fn interested_in(&self) -> Vec<&str>;
}

/// In-memory event publisher for testing
pub struct InMemoryEventPublisher {
    events: Arc<tokio::sync::Mutex<Vec<DomainEvent>>>,
}

impl InMemoryEventPublisher {
    /// Create a new in-memory event publisher
    pub fn new() -> Self {
        Self {
            events: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    /// Get all published events
    pub async fn get_events(&self) -> Vec<DomainEvent> {
        self.events.lock().await.clone()
    }

    /// Clear all events
    pub async fn clear(&self) {
        self.events.lock().await.clear();
    }
}

impl Default for InMemoryEventPublisher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventPublisher for InMemoryEventPublisher {
    async fn publish(&self, event: DomainEvent) -> crate::Result<()> {
        self.events.lock().await.push(event);
        Ok(())
    }

    async fn publish_batch(&self, events: Vec<DomainEvent>) -> crate::Result<()> {
        self.events.lock().await.extend(events);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type() {
        let event = DomainEvent::BuildCreated {
            build_id: BuildId::new(),
            pipeline_id: PipelineId::new(),
            project_id: ProjectId::new(),
            number: 1,
            created_at: Utc::now(),
        };

        assert_eq!(event.event_type(), "build.created");
    }

    #[test]
    fn test_event_timestamp() {
        let now = Utc::now();
        let event = DomainEvent::BuildStarted {
            build_id: BuildId::new(),
            agent_id: AgentId::new(),
            started_at: now,
        };

        assert_eq!(event.timestamp(), now);
    }

    #[tokio::test]
    async fn test_in_memory_publisher() {
        let publisher = InMemoryEventPublisher::new();

        let event = DomainEvent::BuildCreated {
            build_id: BuildId::new(),
            pipeline_id: PipelineId::new(),
            project_id: ProjectId::new(),
            number: 1,
            created_at: Utc::now(),
        };

        publisher.publish(event.clone()).await.unwrap();

        let events = publisher.get_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type(), "build.created");
    }

    #[tokio::test]
    async fn test_batch_publish() {
        let publisher = InMemoryEventPublisher::new();

        let events = vec![
            DomainEvent::BuildCreated {
                build_id: BuildId::new(),
                pipeline_id: PipelineId::new(),
                project_id: ProjectId::new(),
                number: 1,
                created_at: Utc::now(),
            },
            DomainEvent::BuildStarted {
                build_id: BuildId::new(),
                agent_id: AgentId::new(),
                started_at: Utc::now(),
            },
        ];

        publisher.publish_batch(events).await.unwrap();

        let stored_events = publisher.get_events().await;
        assert_eq!(stored_events.len(), 2);
    }
}
