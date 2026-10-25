//! Agent repository interface

use crate::domain::entities::agent::{Agent, AgentStatus};
use crate::domain::value_objects::agent_id::AgentId;
use async_trait::async_trait;

/// Agent repository interface
#[async_trait]
pub trait AgentRepository: Send + Sync {
    /// Save an agent
    async fn save(&self, agent: &Agent) -> crate::Result<()>;

    /// Find an agent by ID
    async fn find_by_id(&self, id: &AgentId) -> crate::Result<Option<Agent>>;

    /// Find an agent by name
    async fn find_by_name(&self, name: &str) -> crate::Result<Option<Agent>>;

    /// Find all agents
    async fn find_all(&self) -> crate::Result<Vec<Agent>>;

    /// Find agents by status
    async fn find_by_status(&self, status: &AgentStatus) -> crate::Result<Vec<Agent>>;

    /// Find available agents (online and can accept jobs)
    async fn find_available(&self) -> crate::Result<Vec<Agent>>;

    /// Find agents with specific labels
    async fn find_by_labels(&self, labels: &[(String, String)]) -> crate::Result<Vec<Agent>>;

    /// Update an agent
    async fn update(&self, agent: &Agent) -> crate::Result<()>;

    /// Delete an agent
    async fn delete(&self, id: &AgentId) -> crate::Result<()>;

    /// Check if an agent exists
    async fn exists(&self, id: &AgentId) -> crate::Result<bool>;
}
