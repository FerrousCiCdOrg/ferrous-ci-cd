//! Agent domain service

use crate::domain::entities::agent::{Agent, AgentPlatform, AgentStatus};
use crate::domain::events::EventPublisher;
use crate::domain::repositories::agent::AgentRepository;
use crate::domain::value_objects::agent_id::AgentId;
use std::sync::Arc;

/// Agent service
pub struct AgentService {
    repository: Arc<dyn AgentRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl AgentService {
    /// Create a new agent service
    pub fn new(
        repository: Arc<dyn AgentRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            repository,
            event_publisher,
        }
    }

    /// Register a new agent
    pub async fn register_agent(
        &self,
        name: String,
        max_concurrent_jobs: usize,
        platform: AgentPlatform,
        version: String,
        ip_address: String,
    ) -> crate::Result<Agent> {
        // Check if agent name already exists
        if let Some(_) = self.repository.find_by_name(&name).await? {
            return Err(crate::Error::conflict("Agent name already exists"));
        }

        // Create agent
        let mut agent = Agent::new(name, max_concurrent_jobs, platform, version);
        agent.register(ip_address)?;

        // Save agent
        self.repository.save(&agent).await?;

        // Publish events
        let events = agent.take_events();
        self.event_publisher.publish_batch(events).await?;

        Ok(agent)
    }

    /// Update agent heartbeat
    pub async fn heartbeat(&self, agent_id: &AgentId) -> crate::Result<()> {
        let mut agent = self
            .repository
            .find_by_id(agent_id)
            .await?
            .ok_or_else(|| crate::Error::not_found("Agent not found"))?;

        agent.heartbeat()?;

        self.repository.update(&agent).await?;

        Ok(())
    }

    /// Disconnect an agent
    pub async fn disconnect_agent(&self, agent_id: &AgentId) -> crate::Result<()> {
        let mut agent = self
            .repository
            .find_by_id(agent_id)
            .await?
            .ok_or_else(|| crate::Error::not_found("Agent not found"))?;

        agent.disconnect();

        self.repository.update(&agent).await?;

        let events = agent.take_events();
        self.event_publisher.publish_batch(events).await?;

        Ok(())
    }

    /// Find available agents for a job
    pub async fn find_available_agents(&self) -> crate::Result<Vec<Agent>> {
        self.repository.find_available().await
    }

    /// Assign a job to an agent
    pub async fn assign_job(&self, agent_id: &AgentId) -> crate::Result<()> {
        let mut agent = self
            .repository
            .find_by_id(agent_id)
            .await?
            .ok_or_else(|| crate::Error::not_found("Agent not found"))?;

        agent.assign_job()?;

        self.repository.update(&agent).await?;

        Ok(())
    }

    /// Release a job from an agent
    pub async fn release_job(&self, agent_id: &AgentId) -> crate::Result<()> {
        let mut agent = self
            .repository
            .find_by_id(agent_id)
            .await?
            .ok_or_else(|| crate::Error::not_found("Agent not found"))?;

        agent.release_job()?;

        self.repository.update(&agent).await?;

        Ok(())
    }

    /// Find dead agents and mark them as disconnected
    pub async fn cleanup_dead_agents(&self, timeout_seconds: i64) -> crate::Result<usize> {
        let all_agents = self.repository.find_all().await?;
        let mut cleaned = 0;

        for mut agent in all_agents {
            if agent.status() != &AgentStatus::Offline && agent.is_dead(timeout_seconds) {
                agent.disconnect();
                self.repository.update(&agent).await?;

                let events = agent.take_events();
                self.event_publisher.publish_batch(events).await?;

                cleaned += 1;
            }
        }

        Ok(cleaned)
    }
}
