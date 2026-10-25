//! Pipeline domain service

use crate::domain::entities::pipeline::Pipeline;
use crate::domain::events::EventPublisher;
use crate::domain::repositories::pipeline::PipelineRepository;
use crate::domain::value_objects::{
    pipeline_config::PipelineConfig, pipeline_id::PipelineId, project_id::ProjectId,
};
use std::sync::Arc;

/// Pipeline service
pub struct PipelineService {
    repository: Arc<dyn PipelineRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl PipelineService {
    /// Create a new pipeline service
    pub fn new(
        repository: Arc<dyn PipelineRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            repository,
            event_publisher,
        }
    }

    /// Create a new pipeline
    pub async fn create_pipeline(
        &self,
        project_id: ProjectId,
        name: String,
        config: PipelineConfig,
    ) -> crate::Result<Pipeline> {
        // Validate configuration
        config.validate()?;

        // Create pipeline
        let mut pipeline = Pipeline::new(project_id, name, config);

        // Validate pipeline
        pipeline.validate()?;

        // Save pipeline
        self.repository.save(&pipeline).await?;

        // Publish events
        let events = pipeline.take_events();
        self.event_publisher.publish_batch(events).await?;

        Ok(pipeline)
    }

    /// Update pipeline configuration
    pub async fn update_config(
        &self,
        pipeline_id: &PipelineId,
        config: PipelineConfig,
    ) -> crate::Result<()> {
        // Validate configuration
        config.validate()?;

        // Find pipeline
        let mut pipeline = self
            .repository
            .find_by_id(pipeline_id)
            .await?
            .ok_or_else(|| crate::Error::not_found("Pipeline not found"))?;

        // Update configuration
        pipeline.update_config(config)?;

        // Save pipeline
        self.repository.update(&pipeline).await?;

        // Publish events
        let events = pipeline.take_events();
        self.event_publisher.publish_batch(events).await?;

        Ok(())
    }

    /// Enable a pipeline
    pub async fn enable_pipeline(&self, pipeline_id: &PipelineId) -> crate::Result<()> {
        let mut pipeline = self
            .repository
            .find_by_id(pipeline_id)
            .await?
            .ok_or_else(|| crate::Error::not_found("Pipeline not found"))?;

        pipeline.enable();

        self.repository.update(&pipeline).await?;

        let events = pipeline.take_events();
        self.event_publisher.publish_batch(events).await?;

        Ok(())
    }

    /// Disable a pipeline
    pub async fn disable_pipeline(&self, pipeline_id: &PipelineId) -> crate::Result<()> {
        let mut pipeline = self
            .repository
            .find_by_id(pipeline_id)
            .await?
            .ok_or_else(|| crate::Error::not_found("Pipeline not found"))?;

        pipeline.disable();

        self.repository.update(&pipeline).await?;

        let events = pipeline.take_events();
        self.event_publisher.publish_batch(events).await?;

        Ok(())
    }

    /// Delete a pipeline
    pub async fn delete_pipeline(&self, pipeline_id: &PipelineId) -> crate::Result<()> {
        // Check if pipeline exists
        if !self.repository.exists(pipeline_id).await? {
            return Err(crate::Error::not_found("Pipeline not found"));
        }

        // Delete pipeline
        self.repository.delete(pipeline_id).await?;

        Ok(())
    }

    /// Get a pipeline by ID
    pub async fn get_pipeline(&self, pipeline_id: &PipelineId) -> crate::Result<Pipeline> {
        self.repository
            .find_by_id(pipeline_id)
            .await?
            .ok_or_else(|| crate::Error::not_found("Pipeline not found"))
    }

    /// Get all pipelines for a project
    pub async fn get_project_pipelines(
        &self,
        project_id: &ProjectId,
    ) -> crate::Result<Vec<Pipeline>> {
        self.repository.find_by_project(project_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::events::InMemoryEventPublisher;
    use crate::domain::value_objects::pipeline_config::{Job, Stage, Trigger};
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        pub PipelineRepo {}

        #[async_trait::async_trait]
        impl PipelineRepository for PipelineRepo {
            async fn save(&self, pipeline: &Pipeline) -> crate::Result<()>;
            async fn find_by_id(&self, id: &PipelineId) -> crate::Result<Option<Pipeline>>;
            async fn find_by_project(&self, project_id: &ProjectId) -> crate::Result<Vec<Pipeline>>;
            async fn find_enabled_by_project(&self, project_id: &ProjectId) -> crate::Result<Vec<Pipeline>>;
            async fn find_all(&self) -> crate::Result<Vec<Pipeline>>;
            async fn update(&self, pipeline: &Pipeline) -> crate::Result<()>;
            async fn delete(&self, id: &PipelineId) -> crate::Result<()>;
            async fn exists(&self, id: &PipelineId) -> crate::Result<bool>;
        }
    }

    fn create_test_config() -> PipelineConfig {
        let mut job = Job::new("test".to_string());
        job.add_command("echo 'test'".to_string());

        PipelineConfig::new(
            vec![Stage::new("build".to_string(), vec![job])],
            vec![Trigger::Manual],
        )
    }

    #[tokio::test]
    async fn test_create_pipeline() {
        let mut mock_repo = MockPipelineRepo::new();
        mock_repo.expect_save().times(1).returning(|_| Ok(()));

        let event_publisher = Arc::new(InMemoryEventPublisher::new());
        let service = PipelineService::new(Arc::new(mock_repo), event_publisher.clone());

        let result = service
            .create_pipeline(
                ProjectId::new(),
                "test-pipeline".to_string(),
                create_test_config(),
            )
            .await;

        assert!(result.is_ok());

        let events = event_publisher.get_events().await;
        assert_eq!(events.len(), 1);
    }
}
