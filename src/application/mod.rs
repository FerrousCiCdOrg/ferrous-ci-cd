//! Application layer - Use cases and application services

pub mod dto;
pub mod use_cases;

use crate::config::Config;
use crate::domain::events::{EventPublisher, InMemoryEventPublisher};
use crate::domain::repositories::{
    agent::AgentRepository, build::BuildRepository, pipeline::PipelineRepository,
};
use crate::domain::services::{
    agent::AgentService, build::BuildService, pipeline::PipelineService,
};
use std::sync::Arc;

/// Application instance
pub struct Application {
    config: Config,
    #[allow(dead_code)]
    event_publisher: Arc<dyn EventPublisher>,
    pipeline_service: Arc<PipelineService>,
    build_service: Arc<BuildService>,
    agent_service: Arc<AgentService>,
}

impl Application {
    /// Create a new application instance
    pub async fn new(config: Config) -> crate::Result<Self> {
        // Validate configuration
        config.validate()?;

        // Create event publisher (in-memory for now)
        let event_publisher = Arc::new(InMemoryEventPublisher::new());

        // TODO: Initialize database connection
        // TODO: Initialize repository implementations

        // For now, we'll return a placeholder
        // This will be properly implemented in the infrastructure layer

        Ok(Self {
            config,
            event_publisher: event_publisher.clone(),
            pipeline_service: Arc::new(PipelineService::new(
                create_placeholder_pipeline_repo(),
                event_publisher.clone(),
            )),
            build_service: Arc::new(BuildService::new(
                create_placeholder_build_repo(),
                event_publisher.clone(),
            )),
            agent_service: Arc::new(AgentService::new(
                create_placeholder_agent_repo(),
                event_publisher,
            )),
        })
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the pipeline service
    pub fn pipeline_service(&self) -> &PipelineService {
        &self.pipeline_service
    }

    /// Get the build service
    pub fn build_service(&self) -> &BuildService {
        &self.build_service
    }

    /// Get the agent service
    pub fn agent_service(&self) -> &AgentService {
        &self.agent_service
    }
}

// Placeholder functions - will be replaced with actual implementations
fn create_placeholder_pipeline_repo() -> Arc<dyn PipelineRepository> {
    use crate::infrastructure::repositories::in_memory::InMemoryPipelineRepository;
    Arc::new(InMemoryPipelineRepository::new())
}

fn create_placeholder_build_repo() -> Arc<dyn BuildRepository> {
    use crate::infrastructure::repositories::in_memory::InMemoryBuildRepository;
    Arc::new(InMemoryBuildRepository::new())
}

fn create_placeholder_agent_repo() -> Arc<dyn AgentRepository> {
    use crate::infrastructure::repositories::in_memory::InMemoryAgentRepository;
    Arc::new(InMemoryAgentRepository::new())
}
