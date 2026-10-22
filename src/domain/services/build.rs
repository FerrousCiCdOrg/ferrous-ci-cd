//! Build domain service

use crate::domain::entities::build::{Build, BuildTrigger};
use crate::domain::events::EventPublisher;
use crate::domain::repositories::build::BuildRepository;
use crate::domain::value_objects::{
    agent_id::AgentId, build_id::BuildId, pipeline_id::PipelineId, project_id::ProjectId,
};
use std::sync::Arc;

/// Build service
pub struct BuildService {
    repository: Arc<dyn BuildRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl BuildService {
    /// Create a new build service
    pub fn new(
        repository: Arc<dyn BuildRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            repository,
            event_publisher,
        }
    }

    /// Create a new build
    pub async fn create_build(
        &self,
        pipeline_id: PipelineId,
        project_id: ProjectId,
        commit_sha: String,
        branch: String,
        trigger: BuildTrigger,
    ) -> crate::Result<Build> {
        // Get next build number
        let number = self.repository.next_build_number(&pipeline_id).await?;

        // Create build
        let mut build = Build::new(pipeline_id, project_id, number, commit_sha, branch, trigger);

        // Save build
        self.repository.save(&build).await?;

        // Publish events
        let events = build.take_events();
        self.event_publisher.publish_batch(events).await?;

        Ok(build)
    }

    /// Start a build
    pub async fn start_build(&self, build_id: &BuildId, agent_id: AgentId) -> crate::Result<()> {
        let mut build = self
            .repository
            .find_by_id(build_id)
            .await?
            .ok_or_else(|| crate::Error::not_found("Build not found"))?;

        build.start(agent_id)?;

        self.repository.update(&build).await?;

        let events = build.take_events();
        self.event_publisher.publish_batch(events).await?;

        Ok(())
    }

    /// Complete a build successfully
    pub async fn complete_build(&self, build_id: &BuildId) -> crate::Result<()> {
        let mut build = self
            .repository
            .find_by_id(build_id)
            .await?
            .ok_or_else(|| crate::Error::not_found("Build not found"))?;

        build.succeed()?;

        self.repository.update(&build).await?;

        let events = build.take_events();
        self.event_publisher.publish_batch(events).await?;

        Ok(())
    }

    /// Fail a build
    pub async fn fail_build(&self, build_id: &BuildId, error_message: String) -> crate::Result<()> {
        let mut build = self
            .repository
            .find_by_id(build_id)
            .await?
            .ok_or_else(|| crate::Error::not_found("Build not found"))?;

        build.fail(error_message)?;

        self.repository.update(&build).await?;

        let events = build.take_events();
        self.event_publisher.publish_batch(events).await?;

        Ok(())
    }

    /// Cancel a build
    pub async fn cancel_build(&self, build_id: &BuildId) -> crate::Result<()> {
        let mut build = self
            .repository
            .find_by_id(build_id)
            .await?
            .ok_or_else(|| crate::Error::not_found("Build not found"))?;

        build.cancel()?;

        self.repository.update(&build).await?;

        let events = build.take_events();
        self.event_publisher.publish_batch(events).await?;

        Ok(())
    }

    /// Get a build by ID
    pub async fn get_build(&self, build_id: &BuildId) -> crate::Result<Build> {
        self.repository
            .find_by_id(build_id)
            .await?
            .ok_or_else(|| crate::Error::not_found("Build not found"))
    }

    /// Get builds for a pipeline
    pub async fn get_pipeline_builds(&self, pipeline_id: &PipelineId) -> crate::Result<Vec<Build>> {
        self.repository.find_by_pipeline(pipeline_id).await
    }

    /// Get running builds
    pub async fn get_running_builds(&self) -> crate::Result<Vec<Build>> {
        self.repository.find_running().await
    }
}
