//! Pipeline repository interface

use crate::domain::entities::pipeline::Pipeline;
use crate::domain::value_objects::{pipeline_id::PipelineId, project_id::ProjectId};
use async_trait::async_trait;

/// Pipeline repository interface
#[async_trait]
pub trait PipelineRepository: Send + Sync {
    /// Save a pipeline
    async fn save(&self, pipeline: &Pipeline) -> crate::Result<()>;

    /// Find a pipeline by ID
    async fn find_by_id(&self, id: &PipelineId) -> crate::Result<Option<Pipeline>>;

    /// Find all pipelines for a project
    async fn find_by_project(&self, project_id: &ProjectId) -> crate::Result<Vec<Pipeline>>;

    /// Find enabled pipelines for a project
    async fn find_enabled_by_project(&self, project_id: &ProjectId)
        -> crate::Result<Vec<Pipeline>>;

    /// Find all pipelines
    async fn find_all(&self) -> crate::Result<Vec<Pipeline>>;

    /// Update a pipeline
    async fn update(&self, pipeline: &Pipeline) -> crate::Result<()>;

    /// Delete a pipeline
    async fn delete(&self, id: &PipelineId) -> crate::Result<()>;

    /// Check if a pipeline exists
    async fn exists(&self, id: &PipelineId) -> crate::Result<bool>;
}
